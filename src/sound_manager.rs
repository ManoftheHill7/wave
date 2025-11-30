use crate::config;
use rand::Rng;
use raylib::ffi;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;

pub struct SoundManager {
    /// Sounds organized by category (e.g., "pickaxe", "footstep", "jumping")
    sounds: HashMap<String, Vec<ffi::Sound>>,
    /// Mute state
    muted: bool,
    /// Volume multiplier
    volume: f32,
}

impl SoundManager {
    pub fn new() -> Result<Self, String> {
        let mut sounds: HashMap<String, Vec<ffi::Sound>> = HashMap::new();
        let effects_dir = "assets/sound/effects";

        // Read all subdirectories in effects folder
        let entries = fs::read_dir(effects_dir)
            .map_err(|e| format!("Failed to read effects directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                let category = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| "Invalid directory name".to_string())?
                    .to_string();

                let mut category_sounds = Vec::new();

                // Read all .wav files in this subdirectory
                let files = fs::read_dir(&path)
                    .map_err(|e| format!("Failed to read category {}: {}", category, e))?;

                for file in files {
                    let file = file.map_err(|e| format!("Failed to read file: {}", e))?;
                    let file_path = file.path();

                    if file_path.extension().and_then(|e| e.to_str()) == Some("wav") {
                        let sound = load_sound(file_path.to_str().unwrap())?;
                        category_sounds.push(sound);
                    }
                }

                if !category_sounds.is_empty() {
                    println!(
                        "Loaded {} sounds for category '{}'",
                        category_sounds.len(),
                        category
                    );
                    sounds.insert(category, category_sounds);
                }
            }
        }

        // Load mute state from config (same as music)
        let cfg = config::load_config();

        Ok(SoundManager {
            sounds,
            muted: cfg.audio.muted,
            volume: 1.0,
        })
    }

    /// Play a random sound from the given category
    pub fn play_random(&self, category: &str) {
        if self.muted {
            return;
        }

        if let Some(category_sounds) = self.sounds.get(category) {
            if !category_sounds.is_empty() {
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..category_sounds.len());
                let sound = category_sounds[index];
                unsafe {
                    ffi::SetSoundVolume(sound, self.volume);
                    ffi::PlaySound(sound);
                }
            }
        }
    }

    /// Play a specific sound from the given category by index
    pub fn play(&self, category: &str, index: usize) {
        if self.muted {
            return;
        }

        if let Some(category_sounds) = self.sounds.get(category) {
            if let Some(&sound) = category_sounds.get(index) {
                unsafe {
                    ffi::SetSoundVolume(sound, self.volume);
                    ffi::PlaySound(sound);
                }
            }
        }
    }

    /// Play a specific sound file by name within a category
    /// Note: This requires sounds to be loaded with their names preserved
    pub fn play_named(&self, category: &str, name: &str) {
        if self.muted {
            return;
        }

        // For now, we'll use a naming convention where sounds are indexed
        // e.g., "jumping_7" would be index 6 (0-based)
        // This is a simple approach; we could store names separately if needed
        if let Some(category_sounds) = self.sounds.get(category) {
            // Try to extract number from name (e.g., "jumping_7" -> 7)
            if let Some(num_str) = name.split('_').last() {
                if let Ok(num) = num_str.parse::<usize>() {
                    // Convert to 0-based index
                    let index = num.saturating_sub(1);
                    if let Some(&sound) = category_sounds.get(index) {
                        unsafe {
                            ffi::SetSoundVolume(sound, self.volume);
                            ffi::PlaySound(sound);
                        }
                        return;
                    }
                }
            }
        }
    }

    /// Set mute state
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Toggle mute state
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }
}

impl Drop for SoundManager {
    fn drop(&mut self) {
        for sounds in self.sounds.values() {
            for &sound in sounds {
                unsafe {
                    ffi::UnloadSound(sound);
                }
            }
        }
    }
}

/// Load a sound file using raw FFI
fn load_sound(path: &str) -> Result<ffi::Sound, String> {
    let c_path = CString::new(path).map_err(|e| format!("Invalid path: {}", e))?;

    let sound = unsafe { ffi::LoadSound(c_path.as_ptr()) };

    // Check if loaded successfully
    if sound.stream.buffer.is_null() {
        return Err(format!("Failed to load sound: {}", path));
    }

    Ok(sound)
}
