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
    PlaceBlock(Block),
}

impl ToolType {
    pub fn get_texture<'a>(&self, textures: &'a TextureManager) -> Option<&'a Texture2D> {
        match self {
            ToolType::Dash => Some(&textures.tools.emerald_amulet),
            ToolType::Pickaxe => Some(&textures.tools.steel_pickaxe),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPickaxe {
    pub durability: f32,
    pub max_durability: f32,
    pub speed: f32,
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
    }
}
