#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    Dash,
    Pickaxe,
}

pub struct ToolDash {
    pub durability: f32,
    pub max_durability: f32,
    pub max_dashes: i32,
    pub dash_time: f32,
    pub dash_extended_time: f32,
    pub dash_control_modifier: f32,
}

pub struct ToolPickaxe {}

pub fn load_dash(dt: &str) -> ToolDash {
    let toml_str = include_str!("../assets/data/tools.toml");
    let table: toml::Table = toml::from_str(toml_str).expect("Failed to parse tools.toml");

    let dash = table
        .get("dash")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get(dt))
        .and_then(|v| v.as_table())
        .expect((String::new() + "Missing [dash." + dt + "]").as_str());

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
