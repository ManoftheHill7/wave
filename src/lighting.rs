use crate::terrain::Terrain;
use raylib::prelude::*;
use std::collections::HashMap;

const MAX_DARKNESS: f32 = 0.99;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    Lamp,
    CoalTorch,
    LumostoneTorch,
    LumositeOre,
}

#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub position: Vector2,
    pub radius: f32,
    pub intensity: f32,
    pub color: Color,
    pub light_type: LightType,
}

impl Light {
    pub fn new(position: Vector2, light_type: LightType) -> Self {
        let (radius, color) = match light_type {
            LightType::Lamp => (24.0, Color::new(255, 200, 150, 255)), // Warm orange
            LightType::CoalTorch => (12.0, Color::new(255, 200, 150, 255)), // Warm orange
            LightType::LumostoneTorch => (20.0, Color::new(200, 220, 255, 255)), // Cool blue-white
            LightType::LumositeOre => (6.0, Color::new(200, 220, 255, 255)), // Cool blue-white
        };

        Light {
            position,
            radius,
            intensity: radius / 4.0,
            color,
            light_type,
        }
    }

    pub fn with_flicker(&self, time: f32) -> Light {
        let mut light = *self;
        match self.light_type {
            LightType::CoalTorch | LightType::LumostoneTorch => {
                // Subtle flicker for torches
                let flicker = (time * 5.0).sin() * 0.1 + (time * 9.0).sin() * 0.05;
                light.radius = self.radius * (1.0 + flicker * 0.3);
                light.intensity = self.intensity * (1.0 + flicker * 0.3);
            },
            LightType::LumositeOre => {
                // Subtle flicker for ore
                let flicker = (time * 0.9).sin() * 0.05 + (time * 1.4).sin() * 0.025;
                light.radius = self.radius * (1.0 + flicker * 0.6);
                light.intensity = self.intensity * (0.2 + flicker * 0.4);
            },
            _ => {}
        }
        light
    }
}

/// Shadow map stores which tiles are lit (true) or shadowed (false)
#[derive(Debug)]
pub struct ShadowMap {
    data: HashMap<(i32, i32), bool>,
}

impl ShadowMap {
    pub fn new() -> Self {
        ShadowMap {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, x: i32, y: i32, is_lit: bool) {
        self.data.insert((x, y), is_lit);
    }

    pub fn get(&self, x: i32, y: i32) -> bool {
        *self.data.get(&(x, y)).unwrap_or(&false)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

pub struct LightingSystem {
    lights: Vec<Light>,
    shadow_map: ShadowMap,
    opaque_light_cache: HashMap<(i32, i32), f32>,
    air_light_cache: HashMap<(i32, i32), f32>,
    pub ambient_darkness: f32,
}

impl LightingSystem {
    pub fn new() -> Self {
        LightingSystem {
            lights: Vec::new(),
            shadow_map: ShadowMap::new(),
            opaque_light_cache: HashMap::new(),
            air_light_cache: HashMap::new(),
            ambient_darkness: 0.0,
        }
    }

    pub fn clear_lights(&mut self) {
        self.lights.clear();
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn lights(&self) -> &[Light] {
        &self.lights
    }

    /// Update ambient darkness based on player depth
    /// Darkness gradually increases from y=0 to y=100
    pub fn update_ambient_darkness(&mut self, player_y: f32) {
        // At y=0 or above (surface): 0% darkness
        // At y=100 or below (deep): 95% darkness
        self.ambient_darkness = (player_y / 100.0).clamp(0.0, MAX_DARKNESS);
    }

    /// Calculate shadows for all lights using raycasting
    pub fn calculate_shadows(
        &mut self,
        terrain: &Terrain,
        camera_x: i32,
        camera_y: i32,
        render_distance: i32,
    ) {
        self.shadow_map.clear();

        for light in &self.lights {
            let light_radius = light.radius as i32;
            let lx = light.position.x as i32;
            let ly = light.position.y as i32;

            // Only calculate shadows for tiles near the camera
            let min_x = (camera_x - render_distance).max(lx - light_radius);
            let max_x = (camera_x + render_distance).min(lx + light_radius);
            let min_y = (camera_y - render_distance).max(ly - light_radius);
            let max_y = (camera_y + render_distance).min(ly + light_radius);

            for tile_y in min_y..=max_y {
                for tile_x in min_x..=max_x {
                    // Skip if outside light radius
                    let dx = tile_x - lx;
                    let dy = tile_y - ly;
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    if dist_sq > light.radius * light.radius {
                        continue;
                    }

                    // Check if already marked as lit by another light
                    if self.shadow_map.get(tile_x, tile_y) {
                        continue;
                    }

                    // Raycast from light center to tile center
                    let tile_center = Vector2::new(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
                    let is_lit = !self.raycast_hits_terrain(light.position, tile_center, terrain);

                    if is_lit {
                        self.shadow_map.set(tile_x, tile_y, true);
                    }
                }
            }
        }
    }

    /// Raycast from start to end, return true if hits opaque terrain
    fn raycast_hits_terrain(&self, start: Vector2, end: Vector2, terrain: &Terrain) -> bool {
        // DDA algorithm for grid traversal
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 0.01 {
            return false;
        }

        let step_x = dx / distance;
        let step_y = dy / distance;
        let steps = distance.ceil() as i32 * 2; // Sample at least twice per tile

        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let x = start.x + step_x * t * distance;
            let y = start.y + step_y * t * distance;
            if terrain.opaque_terrain_at(x as i32, y as i32) {
                return true;
            }
        }

        false
    }

    /// Check if a specific tile is lit (for rendering)
    pub fn is_tile_lit(&self, x: i32, y: i32) -> bool {
        self.shadow_map.get(x, y)
    }

    /// Get light intensity at a specific world position
    /// This is used for calculating light in AIR tiles
    pub fn get_light_at(&self, x: f32, y: f32) -> f32 {
        let mut total_light = 0.0;

        for light in &self.lights {
            let dx = light.position.x - x;
            let dy = light.position.y - y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < light.radius * light.radius {
                // Check if shadowed
                if !self.shadow_map.get(x as i32, y as i32) {
                    continue;
                }

                // Inverse square falloff
                let attenuation = light.intensity / (1.0 + dist_sq * 0.2);
                total_light += attenuation;
            }
        }

        total_light.min(1.0)
    }

    /// Calculate lighting for opaque blocks based on nearby air tiles
    /// This should be called after calculate_shadows() during update, not render
    pub fn calculate_opaque_lighting(
        &mut self,
        terrain: &Terrain,
        center_x: i32,
        center_y: i32,
        range: i32,
    ) {
        self.air_light_cache.clear();
        self.opaque_light_cache.clear();

        // First pass: Cache air tile brightness
        for ty in (center_y - range - 2)..(center_y + range + 2) {
            for tx in (center_x - range - 2)..(center_x + range + 2) {
                // Skip tiles far from all lights
                let mut near_light = false;
                for light in &self.lights {
                    let dx = tx as f32 - light.position.x;
                    let dy = ty as f32 - light.position.y;
                    let dist_sq = dx * dx + dy * dy;
                    let check_radius = (light.radius + 3.0) * (light.radius + 3.0);
                    if dist_sq <= check_radius {
                        near_light = true;
                        break;
                    }
                }

                if !near_light {
                    continue;
                }

                if !terrain.opaque_terrain_at(tx, ty) {
                    let tile_x = tx as f32 + 0.5;
                    let tile_y = ty as f32 + 0.5;
                    let light = self.get_light_at(tile_x, tile_y);
                    if light > 0.01 {
                        self.air_light_cache.insert((tx, ty), light);
                    }
                }
            }
        }

        // Second pass: Calculate opaque block lighting from nearby air
        for ty in (center_y - range)..(center_y + range) {
            for tx in (center_x - range)..(center_x + range) {
                if !terrain.opaque_terrain_at(tx, ty) {
                    continue;
                }

                let mut max_nearby_light: f32 = 0.0;

                // Check immediate neighbors (distance 1)
                if let Some(&light) = self.air_light_cache.get(&(tx - 1, ty)) {
                    max_nearby_light = max_nearby_light.max(light);
                }
                if let Some(&light) = self.air_light_cache.get(&(tx + 1, ty)) {
                    max_nearby_light = max_nearby_light.max(light);
                }
                if let Some(&light) = self.air_light_cache.get(&(tx, ty - 1)) {
                    max_nearby_light = max_nearby_light.max(light);
                }
                if let Some(&light) = self.air_light_cache.get(&(tx, ty + 1)) {
                    max_nearby_light = max_nearby_light.max(light);
                }

                // Early exit if we found bright light
                if max_nearby_light < 0.95 {
                    // Check diagonal neighbors (distance ~1.4)
                    // Only allow diagonal light if at least one adjacent cardinal is also lit
                    if let Some(&light) = self.air_light_cache.get(&(tx - 1, ty - 1)) {
                        if self.air_light_cache.contains_key(&(tx - 1, ty))
                            || self.air_light_cache.contains_key(&(tx, ty - 1))
                        {
                            max_nearby_light = max_nearby_light.max(light);
                        }
                    }
                    if let Some(&light) = self.air_light_cache.get(&(tx + 1, ty - 1)) {
                        if self.air_light_cache.contains_key(&(tx + 1, ty))
                            || self.air_light_cache.contains_key(&(tx, ty - 1))
                        {
                            max_nearby_light = max_nearby_light.max(light);
                        }
                    }
                    if let Some(&light) = self.air_light_cache.get(&(tx - 1, ty + 1)) {
                        if self.air_light_cache.contains_key(&(tx - 1, ty))
                            || self.air_light_cache.contains_key(&(tx, ty + 1))
                        {
                            max_nearby_light = max_nearby_light.max(light);
                        }
                    }
                    if let Some(&light) = self.air_light_cache.get(&(tx + 1, ty + 1)) {
                        if self.air_light_cache.contains_key(&(tx + 1, ty))
                            || self.air_light_cache.contains_key(&(tx, ty + 1))
                        {
                            max_nearby_light = max_nearby_light.max(light);
                        }
                    }

                    // Only check 2-tile distance if still not well lit
                    if max_nearby_light < 0.7 {
                        // Check 2-tile cardinal directions with 50% falloff
                        if let Some(&light) = self.air_light_cache.get(&(tx - 2, ty)) {
                            max_nearby_light = max_nearby_light.max(light * 0.5);
                        }
                        if let Some(&light) = self.air_light_cache.get(&(tx + 2, ty)) {
                            max_nearby_light = max_nearby_light.max(light * 0.5);
                        }
                        if let Some(&light) = self.air_light_cache.get(&(tx, ty - 2)) {
                            max_nearby_light = max_nearby_light.max(light * 0.5);
                        }
                        if let Some(&light) = self.air_light_cache.get(&(tx, ty + 2)) {
                            max_nearby_light = max_nearby_light.max(light * 0.5);
                        }
                    }
                }

                if max_nearby_light > 0.01 {
                    self.opaque_light_cache.insert((tx, ty), max_nearby_light);
                }
            }
        }
    }

    /// Get cached light value for any tile (air or opaque)
    pub fn get_cached_light(&self, x: i32, y: i32, is_opaque: bool) -> f32 {
        if is_opaque {
            self.opaque_light_cache.get(&(x, y)).copied().unwrap_or(0.0)
        } else {
            self.air_light_cache.get(&(x, y)).copied().unwrap_or(0.0)
        }
    }

    /// Create a dual-channel lighting texture for GPU rendering
    /// Returns RG pixel data: R=brightness (0-255), G=is_opaque flag (0 or 255)
    /// Uses 1 texel per tile; shader does sub-tile interpolation
    pub fn create_lighting_texture(
        &self,
        terrain: &Terrain,
        center_x: i32,
        center_y: i32,
        range: i32,
    ) -> Vec<u8> {
        let size = range * 2;
        let mut pixels = vec![0u8; (size * size * 2) as usize]; // 2 channels: RG

        for y in 0..size {
            for x in 0..size {
                let wx = center_x - range + x;
                let wy = center_y - range + y;
                let is_opaque = terrain.opaque_terrain_at(wx, wy);

                // Use cached lighting values (from CPU calculations)
                let brightness = self.get_cached_light(wx, wy, is_opaque);

                let idx = ((y * size + x) * 2) as usize;
                pixels[idx] = (brightness * 255.0) as u8; // R channel = brightness
                pixels[idx + 1] = if is_opaque { 255 } else { 0 }; // G channel = is_opaque flag
            }
        }

        pixels
    }

    /// Get the size of the lighting texture
    pub fn get_texture_size(range: i32) -> i32 {
        range * 2
    }
}
