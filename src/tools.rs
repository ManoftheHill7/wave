use raylib::prelude::Texture2D;
use serde::{Deserialize, Serialize};
use toml::map::Map;
use toml::Value;

use crate::terrain::Block;
use crate::TextureManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolType {
    Dash,
    Pickaxe,
    Lamp,
    Glider,
    PlaceBlock(Block),
}

impl ToolType {
    pub fn get_texture<'a>(&self, textures: &'a TextureManager) -> Option<&'a Texture2D> {
        match self {
            ToolType::Dash => Some(&textures.tools.white_pearl_amulet),
            ToolType::Pickaxe => Some(&textures.tools.stone_pickaxe),
            ToolType::Lamp => Some(&textures.tools.lamp_coal1),
            ToolType::Glider => Some(&textures.tools.glider),
            ToolType::PlaceBlock(blk) => blk.to_item_type().map(|item| item.get_texture(textures)),
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
            // All glider tiers use the same texture for now until new art is added
            "linen_glider" | "pearl_glider" | "amethyst_glider" | "emerald_glider"
            | "topaz_glider" | "ruby_glider" | "diamond_glider" => &textures.tools.glider,
            _ => &textures.tools.glider,
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
