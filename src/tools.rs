use raylib::prelude::Texture2D;
use serde::{Deserialize, Serialize};
use toml::map::Map;
use toml::Value;

use crate::inventory::ItemType;
use crate::terrain::Block;
use crate::TextureManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolType {
    Dash,
    Pickaxe,
    Lamp,
    Glider,
    TideClock,
    PlaceBlock(Block),
    //ConsumeItem(ItemType),
}

impl ToolType {
    pub fn get_texture<'a>(&self, textures: &'a TextureManager) -> Option<&'a Texture2D> {
        match self {
            ToolType::Dash => Some(&textures.tools.white_pearl_amulet),
            ToolType::Pickaxe => Some(&textures.tools.stone_pickaxe),
            ToolType::Lamp => Some(&textures.tools.lamp_coal1),
            ToolType::Glider => Some(&textures.tools.linen_glider),
            ToolType::TideClock => Some(&textures.items.tidalcave_clock),
            ToolType::PlaceBlock(blk) => blk.to_item_type().map(|item| item.get_texture(textures)),
            //ToolType::ConsumeItem(itm) => itm.consume_item(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDash {
    pub durability: f32,
    pub max_durability: f32,
    pub max_dashes: i32,
    pub dash_time: f32,
    pub dash_extended_time: f32,
    pub dash_control_modifier: f32,
    pub level: String,
}

impl ToolDash {
    pub fn texture_for_level<'a>(level: &str, textures: &'a TextureManager) -> &'a Texture2D {
        match level {
            "white_pearl_amulet" => &textures.tools.white_pearl_amulet,
            "amethyst_amulet" => &textures.tools.amethyst_amulet,
            "jasper_amulet" => &textures.tools.jasper_amulet,
            "emerald_amulet" => &textures.tools.emerald_amulet,
            "black_pearl_amulet" => &textures.tools.black_pearl_amulet,
            "topaz_amulet" => &textures.tools.topaz_amulet,
            "ruby_amulet" => &textures.tools.ruby_amulet,
            "diamond_amulet" => &textures.tools.diamond_amulet,
            _ => &textures.tools.white_pearl_amulet,
        }
    }

    pub fn get_texture<'a>(&self, textures: &'a TextureManager) -> &'a Texture2D {
        Self::texture_for_level(&self.level, textures)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPickaxe {
    pub durability: f32,
    pub max_durability: f32,
    pub speed: f32,
    pub level: String,
}

impl ToolPickaxe {
    pub fn texture_for_level<'a>(level: &str, textures: &'a TextureManager) -> &'a Texture2D {
        match level {
            "stone_pickaxe" => &textures.tools.stone_pickaxe,
            "copper_pickaxe" => &textures.tools.copper_pickaxe,
            "bronze_pickaxe" => &textures.tools.bronze_pickaxe,
            "iron_pickaxe" => &textures.tools.iron_pickaxe,
            "steel_pickaxe" => &textures.tools.steel_pickaxe,
            "platinum_pickaxe" => &textures.tools.platinum_pickaxe,
            "mithril_pickaxe" => &textures.tools.mithril_pickaxe,
            "ozathinum_pickaxe" => &textures.tools.ozathinum_pickaxe,
            "torzite_pickaxe" => &textures.tools.torzite_pickaxe,
            "etherealite_pickaxe" => &textures.tools.etherealite_pickaxe,
            _ => &textures.tools.stone_pickaxe,
        }
    }

    pub fn get_texture<'a>(&self, textures: &'a TextureManager) -> &'a Texture2D {
        Self::texture_for_level(&self.level, textures)
    }
}

pub fn load_leveled_tool(tool: &str, level: &str) -> Map<String, Value> {
    let toml_str = include_str!("../assets/data/tools.toml");
    let table: toml::Table = toml::from_str(toml_str).expect("Failed to parse tools.toml");
    table
        .get(tool)
        .and_then(|v| v.as_table())
        .and_then(|t| t.get(level))
        .and_then(|v| v.as_table())
        .expect((String::new() + "Missing [" + tool + "." + level + "]").as_str())
        .clone()
}

pub fn load_pick(level: &str) -> ToolPickaxe {
    let tool = load_leveled_tool("pick", level);
    ToolPickaxe {
        durability: tool.get("durability").unwrap().as_float().unwrap() as f32,
        max_durability: tool.get("durability").unwrap().as_float().unwrap() as f32,
        speed: tool.get("speed").unwrap().as_float().unwrap() as f32,
        level: level.to_string(),
    }
}

pub fn load_dash(level: &str) -> ToolDash {
    let dash = load_leveled_tool("dash", level);
    ToolDash {
        durability: dash.get("durability").unwrap().as_float().unwrap() as f32,
        max_durability: dash.get("durability").unwrap().as_float().unwrap() as f32,
        max_dashes: dash.get("max_dashes").unwrap().as_integer().unwrap() as i32,
        dash_time: dash.get("dash_time").unwrap().as_float().unwrap() as f32,
        dash_extended_time: dash.get("dash_extended_time").unwrap().as_float().unwrap() as f32,
        dash_control_modifier: dash
            .get("dash_control_modifier")
            .unwrap()
            .as_float()
            .unwrap() as f32,
        level: level.to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolGlider {
    pub durability: f32,
    pub max_durability: f32,
    pub max_fall_speed: f32,
    pub level: String,
}

impl ToolGlider {
    pub fn texture_for_level<'a>(level: &str, textures: &'a TextureManager) -> &'a Texture2D {
        match level {
            "broken" => &textures.tools.broken_glider,
            "linen_glider" => &textures.tools.linen_glider,
            "white_pearl_glider" => &textures.tools.white_pearl_glider,
            "amethyst_glider" => &textures.tools.amethyst_glider,
            "jasper_glider" => &textures.tools.jasper_glider,
            "emerald_glider" => &textures.tools.emerald_glider,
            "topaz_glider" => &textures.tools.topaz_glider,
            "ruby_glider" => &textures.tools.ruby_glider,
            "diamond_glider" => &textures.tools.diamond_glider,
            _ => &textures.tools.linen_glider,
        }
    }

    pub fn get_texture<'a>(&self, textures: &'a TextureManager) -> &'a Texture2D {
        if self.durability <= 0.0 {
            &textures.tools.broken_glider
        } else {
            Self::texture_for_level(&self.level, textures)
        }
    }
}

pub fn load_glider(level: &str) -> ToolGlider {
    let glider = load_leveled_tool("glider", level);
    ToolGlider {
        durability: glider.get("durability").unwrap().as_float().unwrap() as f32,
        max_durability: glider.get("durability").unwrap().as_float().unwrap() as f32,
        max_fall_speed: glider.get("max_fall_speed").unwrap().as_float().unwrap() as f32,
        level: level.to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTideClock {
    // TideClock has no durability or levels - it just exists
}

impl ToolTideClock {
    pub fn new() -> Self {
        ToolTideClock {}
    }

    /// Get the tideclock frame texture based on tide percentage (0.0 to 1.0)
    /// Frame 0 = low tide, last frame = high tide
    pub fn get_frame_texture<'a>(tide_percent: f32, textures: &'a TextureManager) -> &'a Texture2D {
        // Clamp to 0.0-1.0 range
        let percent = 1.0 - tide_percent.clamp(0.0, 1.0);
        // Get the number of frames available
        let num_frames = textures.ui.tideclock.len();
        // Map percentage to frame index
        let frame = ((percent * (num_frames - 1) as f32).round() as usize).min(num_frames - 1);
        &textures.ui.tideclock[frame]
    }
}
