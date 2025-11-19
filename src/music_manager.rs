use crate::config;
use raylib::ffi;
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::CString;

#[derive(Debug, Deserialize)]
pub struct MusicConfig {
    pub static_music: HashMap<String, StaticMusicConfig>,
    pub game_layers: Vec<GameLayerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct StaticMusicConfig {
    pub file: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GameLayerConfig {
    pub name: String,
    pub file: String,
    pub depth_curve: [DepthPoint; 4], // Trapezoid: 4 points
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct DepthPoint {
    pub depth: f32,
    pub volume: f32,
}

impl GameLayerConfig {
    /// Calculate volume for this layer at a given depth using trapezoid curve
    pub fn volume_at_depth(&self, depth: f32) -> f32 {
        let [p0, p1, p2, p3] = self.depth_curve;

        // Trapezoid shape:
        // p0 = start fade in (volume 0)
        // p1 = full volume start (volume 1)
        // p2 = full volume end (volume 1)
        // p3 = end fade out (volume 0)

        if depth <= p0.depth {
            // Before fade-in (above the trapezoid)
            p0.volume
        } else if depth <= p1.depth {
            // Fade-in region (linear interpolation)
            let range = p0.depth - p1.depth;
            if range.abs() < 0.01 {
                // Points too close, just use p1 volume
                p1.volume
            } else {
                let t = (p0.depth - depth) / range;
                lerp(p0.volume, p1.volume, t)
            }
        } else if depth <= p2.depth {
            // Full volume plateau
            p2.volume
        } else if depth <= p3.depth {
            // Fade-out region (linear interpolation)
            let range = p2.depth - p3.depth;
            if range.abs() < 0.01 {
                // Points too close, just use p3 volume
                p3.volume
            } else {
                let t = (p2.depth - depth) / range;
                lerp(p2.volume, p3.volume, t)
            }
        } else {
            // After fade-out (below the trapezoid)
            p3.volume
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

struct GameLayer {
    music: ffi::Music,
    config: GameLayerConfig,
    current_volume: f32,
    target_volume: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveMusic {
    Static,
    GameLayers,
    None,
}

pub struct MusicManager {
    // Static music
    static_music: HashMap<String, ffi::Music>,

    // Game layers
    game_layers: Vec<GameLayer>,

    // State tracking
    active_music: ActiveMusic,
    current_static_track: Option<String>,

    // Volume smoothing (avoids pops/clicks)
    volume_smooth_time: f32,

    // Global volume multiplier (for pause, etc.)
    volume_multiplier: f32,

    // Mute state
    muted: bool,
}

impl MusicManager {
    pub fn new() -> Result<Self, String> {
        // Load config
        let config_str = std::fs::read_to_string("assets/data/music_layers.toml")
            .map_err(|e| format!("Failed to read music config: {}", e))?;
        let config: MusicConfig = toml::from_str(&config_str)
            .map_err(|e| format!("Failed to parse music config: {}", e))?;

        // Load static music
        let mut static_music = HashMap::new();
        for (name, cfg) in &config.static_music {
            let music = load_music(&cfg.file)?;
            static_music.insert(name.clone(), music);
        }

        // Load game layers
        let mut game_layers = Vec::new();
        for layer_config in config.game_layers {
            let music = load_music(&layer_config.file)?;
            game_layers.push(GameLayer {
                music,
                config: layer_config,
                current_volume: 0.0,
                target_volume: 0.0,
            });
        }

        // Load config to get saved mute state
        let config = config::load_config();

        let mut manager = MusicManager {
            static_music,
            game_layers,
            active_music: ActiveMusic::None,
            current_static_track: None,
            volume_smooth_time: 0.15, // 150ms smooth time for volume changes
            volume_multiplier: 1.0,
            muted: config.audio.muted,
        };

        // Apply initial volume state (in case we're starting muted)
        manager.apply_volume();

        Ok(manager)
    }

    /// Play a static music track (menu, crafting, etc.)
    /// Stops any currently playing music and starts the requested track
    pub fn play_static(&mut self, name: &str) {
        // Check if already playing this track
        if self.active_music == ActiveMusic::Static {
            if let Some(ref current) = self.current_static_track {
                if current == name {
                    return; // Already playing this track
                }
            }
        }

        // Stop current music
        self.stop_all();

        // Start requested track
        if let Some(music) = self.static_music.get(name) {
            unsafe {
                ffi::PlayMusicStream(*music);
                // Apply mute state immediately
                let volume = if self.muted {
                    0.0
                } else {
                    self.volume_multiplier
                };
                ffi::SetMusicVolume(*music, volume);
            }
            self.active_music = ActiveMusic::Static;
            self.current_static_track = Some(name.to_string());
        } else {
            eprintln!("Warning: Static music '{}' not found", name);
        }
    }

    /// Start the game layer system
    /// All layers start playing at zero volume (or muted if mute is enabled)
    pub fn play_game_layers(&mut self) {
        if self.active_music == ActiveMusic::GameLayers {
            return; // Already playing
        }

        // Stop current music
        self.stop_all();

        // Start all game layers at zero volume
        // Note: They start at 0 regardless of mute state, volume will be updated by update_game_depth
        for layer in &mut self.game_layers {
            unsafe {
                ffi::PlayMusicStream(layer.music);
                ffi::SetMusicVolume(layer.music, 0.0);
            }
            layer.current_volume = 0.0;
            layer.target_volume = 0.0;
        }

        self.active_music = ActiveMusic::GameLayers;
    }

    /// Update game layers based on player depth
    /// Call this every frame when game layers are active
    pub fn update_game_depth(&mut self, dt: f32, depth: f32) {
        if self.active_music != ActiveMusic::GameLayers {
            return;
        }

        for layer in &mut self.game_layers {
            // Calculate target volume from trapezoid curve
            layer.target_volume = layer.config.volume_at_depth(depth);

            // Smooth volume changes to avoid pops/clicks
            let volume_change_rate = 1.0 / self.volume_smooth_time;
            let max_change = volume_change_rate * dt;

            let diff = layer.target_volume - layer.current_volume;
            if diff.abs() < max_change {
                layer.current_volume = layer.target_volume;
            } else {
                layer.current_volume += diff.signum() * max_change;
            }

            // Apply volume to music stream (multiplied by global volume multiplier and mute)
            let final_volume = if self.muted {
                0.0
            } else {
                layer.current_volume * self.volume_multiplier
            };
            unsafe {
                ffi::SetMusicVolume(layer.music, final_volume);
            }
        }
    }

    /// Update all active music streams
    /// Must be called every frame to keep music playing
    pub fn update_streams(&mut self) {
        // Update static music
        for music in self.static_music.values() {
            unsafe {
                ffi::UpdateMusicStream(*music);
            }
        }

        // Update game layers
        for layer in &self.game_layers {
            unsafe {
                ffi::UpdateMusicStream(layer.music);
            }
        }
    }

    /// Stop all music
    pub fn stop_all(&mut self) {
        // Stop static music
        for music in self.static_music.values() {
            unsafe {
                ffi::StopMusicStream(*music);
            }
        }

        // Stop game layers
        for layer in &self.game_layers {
            unsafe {
                ffi::StopMusicStream(layer.music);
            }
        }

        self.active_music = ActiveMusic::None;
        self.current_static_track = None;
    }

    /// Check if any music is currently playing
    pub fn is_playing(&self) -> bool {
        self.active_music != ActiveMusic::None
    }

    /// Set global volume multiplier (e.g., 0.5 for half volume when paused)
    pub fn set_volume_multiplier(&mut self, multiplier: f32) {
        self.volume_multiplier = multiplier.clamp(0.0, 1.0);

        // Immediately apply to game layers if active
        if self.active_music == ActiveMusic::GameLayers {
            for layer in &self.game_layers {
                let final_volume = if self.muted {
                    0.0
                } else {
                    layer.current_volume * self.volume_multiplier
                };
                unsafe {
                    ffi::SetMusicVolume(layer.music, final_volume);
                }
            }
        }

        // Apply to static music if active
        if self.active_music == ActiveMusic::Static {
            if let Some(ref track_name) = self.current_static_track {
                if let Some(music) = self.static_music.get(track_name) {
                    let final_volume = if self.muted {
                        0.0
                    } else {
                        self.volume_multiplier
                    };
                    unsafe {
                        ffi::SetMusicVolume(*music, final_volume);
                    }
                }
            }
        }
    }

    /// Toggle mute state
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        self.apply_volume();

        // Save mute state to config
        let mut config = config::load_config();
        config.audio.muted = self.muted;
        if let Err(e) = config::save_config(&config) {
            eprintln!("Failed to save config: {}", e);
        }
    }

    /// Check if music is muted
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Apply current volume settings to all active music
    fn apply_volume(&mut self) {
        if self.active_music == ActiveMusic::GameLayers {
            for layer in &self.game_layers {
                let final_volume = if self.muted {
                    0.0
                } else {
                    layer.current_volume * self.volume_multiplier
                };
                unsafe {
                    ffi::SetMusicVolume(layer.music, final_volume);
                }
            }
        }

        if self.active_music == ActiveMusic::Static {
            if let Some(ref track_name) = self.current_static_track {
                if let Some(music) = self.static_music.get(track_name) {
                    let final_volume = if self.muted {
                        0.0
                    } else {
                        self.volume_multiplier
                    };
                    unsafe {
                        ffi::SetMusicVolume(*music, final_volume);
                    }
                }
            }
        }
    }
}

impl Drop for MusicManager {
    fn drop(&mut self) {
        // Stop all music before unloading
        self.stop_all();

        // Unload static music
        for music in self.static_music.values() {
            unsafe {
                ffi::UnloadMusicStream(*music);
            }
        }

        // Unload game layers
        for layer in &self.game_layers {
            unsafe {
                ffi::UnloadMusicStream(layer.music);
            }
        }
    }
}

/// Load a music file using raw FFI
fn load_music(filename: &str) -> Result<ffi::Music, String> {
    let path = format!("assets/sound/music/{}", filename);
    let c_path = CString::new(path.as_str()).map_err(|e| format!("Invalid path: {}", e))?;

    let music = unsafe { ffi::LoadMusicStream(c_path.as_ptr()) };

    // Check if loaded successfully
    if music.stream.buffer.is_null() {
        return Err(format!("Failed to load music: {}", path));
    }

    // Set to loop by modifying the struct field directly
    let mut music_looping = music;
    music_looping.looping = true;

    Ok(music_looping)
}
