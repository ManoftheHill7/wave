use crate::terrain::Terrain;
use raylib::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    Lamp,
    CoalTorch,
    LumostoneTorch,
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
            LightType::Lamp => (8.0, Color::new(255, 200, 150, 255)), // Warm orange
            LightType::CoalTorch => (6.0, Color::new(255, 200, 150, 255)), // Warm orange
            LightType::LumostoneTorch => (10.0, Color::new(200, 220, 255, 255)), // Cool blue-white
        };

        Light {
            position,
            radius,
            intensity: 1.0,
            color,
            light_type,
        }
    }

    pub fn with_flicker(&self, time: f32) -> Light {
        let mut light = *self;
        match self.light_type {
            LightType::CoalTorch | LightType::LumostoneTorch => {
                // Subtle flicker for torches
                let flicker = (time * 10.0).sin() * 0.1 + (time * 23.0).sin() * 0.05;
                light.radius = self.radius * (1.0 + flicker * 0.1);
                light.intensity = self.intensity * (1.0 + flicker * 0.2);
            }
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
    pub ambient_darkness: f32,
}

impl LightingSystem {
    pub fn new() -> Self {
        LightingSystem {
            lights: Vec::new(),
            shadow_map: ShadowMap::new(),
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
        self.ambient_darkness = (player_y / 100.0).clamp(0.0, 0.95);
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

    /// Raycast from start to end, return true if hits solid terrain
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

            if terrain.solid_terrain_at(x as i32, y as i32) {
                return true;
            }
        }

        false
    }

    /// Check if a specific tile is lit (for rendering)
    pub fn is_tile_lit(&self, x: i32, y: i32) -> bool {
        self.shadow_map.get(x, y)
    }

    /// Get light intensity at a specific world position (for shader-less rendering)
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
}
