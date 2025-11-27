use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/data");

    let out_dir = env::var("OUT_DIR").unwrap();

    // Load all TOML data from assets/data/
    let game_data = load_game_data();

    // Generate items
    generate_items(&out_dir, &game_data);

    // Generate blocks
    generate_blocks(&out_dir, &game_data);

    // Generate block-item mappings
    generate_block_mappings(&out_dir, &game_data);

    // Generate recipes
    generate_recipes(&out_dir, &game_data);
}

/// Loads and merges all TOML files from assets/data/ directory
fn load_game_data() -> toml::Table {
    let data_dir = Path::new("assets/data");
    let mut merged_table = toml::Table::new();

    // Read all .toml files in the directory
    let entries = fs::read_dir(data_dir)
        .expect("Failed to read assets/data directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("toml"))
        .collect::<Vec<_>>();

    // Sort by filename for deterministic order
    let mut entries = entries;
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();

        println!("cargo:rerun-if-changed={}", path.display());

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", file_name, e));

        let table: toml::Table = toml::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", file_name, e));

        // Merge tables
        for (key, value) in table {
            if merged_table.contains_key(&key) {
                // If the key already exists and both are tables, merge them
                if let (Some(existing_table), toml::Value::Table(new_table)) = (
                    merged_table.get_mut(&key).and_then(|v| v.as_table_mut()),
                    &value,
                ) {
                    for (sub_key, sub_value) in new_table {
                        existing_table.insert(sub_key.clone(), sub_value.clone());
                    }
                } else {
                    panic!("Conflicting key '{}' in TOML files", key);
                }
            } else {
                merged_table.insert(key, value);
            }
        }
    }

    merged_table
}

fn generate_items(out_dir: &str, game_data: &toml::Table) {
    let dest_path = Path::new(&out_dir).join("generated_items.rs");

    let items = game_data
        .get("items")
        .and_then(|v| v.as_table())
        .expect("Missing [items] table in game data");

    let mut enum_variants = Vec::new();
    let mut name_match_arms = Vec::new();
    let mut weight_match_arms = Vec::new();
    let mut from_str_match_arms = Vec::new();
    let mut all_items = Vec::new();
    let mut texture_match_arms = Vec::new();

    for (key, value) in items.iter() {
        let table = value.as_table().expect("Item must be a table");

        let variant_name = to_pascal_case(key);
        let item_name = table.get("name").and_then(|v| v.as_str()).unwrap_or(key);
        let weight = table
            .get("weight")
            .and_then(|v| {
                if let Some(f) = v.as_float() {
                    Some(f)
                } else if let Some(i) = v.as_integer() {
                    Some(i as f64)
                } else {
                    None
                }
            })
            .unwrap_or(1.0);

        // Get optional image field, default to items.{itemname}
        let texture_path = if let Some(image) = table.get("image").and_then(|v| v.as_str()) {
            // Parse path like "tools.steel_pickaxe" or "items.stone"
            format!("textures.{}", image)
        } else {
            // Default to textures.items.{key}
            format!("textures.items.{}", key)
        };

        enum_variants.push(format!("    {},", variant_name));
        name_match_arms.push(format!(
            "            ItemType::{} => \"{}\",",
            variant_name, item_name
        ));
        weight_match_arms.push(format!(
            "            ItemType::{} => {:.1},",
            variant_name, weight
        ));
        from_str_match_arms.push(format!(
            "            \"{}\" => Some(ItemType::{}),",
            key.to_lowercase(),
            variant_name
        ));
        all_items.push(format!("            ItemType::{},", variant_name));
        texture_match_arms.push(format!(
            "            ItemType::{} => &{},",
            variant_name, texture_path
        ));
    }

    let generated_code = format!(
        r#"#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ItemType {{
{}
}}

impl ItemType {{
    pub fn name(&self) -> &'static str {{
        match self {{
{}
        }}
    }}

    pub fn weight(&self) -> f32 {{
        match self {{
{}
        }}
    }}

    pub fn from_str(s: &str) -> Option<Self> {{
        match s.to_lowercase().as_str() {{
{}
            _ => None,
        }}
    }}

    pub fn all() -> &'static [ItemType] {{
        &[
{}
        ]
    }}

    /// Returns the texture for this item from the TextureManager.
    /// The texture path can be customized in items.toml with the 'image' field.
    /// By default, uses textures.items.{{itemname}}
    pub fn get_texture<'a>(&self, textures: &'a crate::TextureManager) -> &'a raylib::prelude::Texture2D {{
        match self {{
{}
        }}
    }}
}}
"#,
        enum_variants.join("\n"),
        name_match_arms.join("\n"),
        weight_match_arms.join("\n"),
        from_str_match_arms.join("\n"),
        all_items.join("\n"),
        texture_match_arms.join("\n")
    );

    fs::write(&dest_path, generated_code).expect("Failed to write generated items");
}

fn generate_blocks(out_dir: &str, game_data: &toml::Table) {
    let dest_path = Path::new(&out_dir).join("generated_blocks.rs");

    let blocks = game_data
        .get("blocks")
        .and_then(|v| v.as_table())
        .expect("Missing [blocks] table in game data");

    let ores = game_data.get("ore").and_then(|v| v.as_table());

    let mut enum_variants = Vec::new();
    let mut name_match_arms = Vec::new();
    let mut durability_match_arms = Vec::new();
    let mut drops_match_arms = Vec::new();
    let mut key_match_arms = Vec::new();
    let mut from_str_match_arms = Vec::new();
    let mut is_solid_match_arms = Vec::new();
    let mut is_liquid_match_arms = Vec::new();
    let mut is_spike_match_arms = Vec::new();
    let mut all_blocks = Vec::new();
    let mut texture_match_arms = Vec::new();
    let mut ore_spawn_match_arms = Vec::new();
    let mut width_match_arms = Vec::new();
    let mut height_match_arms = Vec::new();
    let mut is_multi_tile_match_arms = Vec::new();

    for (key, value) in blocks.iter() {
        let table = value.as_table().expect("Block must be a table");

        let variant_name = to_pascal_case(key);
        let block_name = table.get("name").and_then(|v| v.as_str()).unwrap_or(key);
        let block_type = table
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("solid");

        let durability = table
            .get("durability")
            .and_then(|v| {
                if let Some(f) = v.as_float() {
                    Some(f)
                } else if let Some(i) = v.as_integer() {
                    Some(i as f64)
                } else {
                    None
                }
            })
            .unwrap_or(1.0);

        enum_variants.push(format!("    {},", variant_name));
        name_match_arms.push(format!(
            "            Block::{} => \"{}\",",
            variant_name, block_name
        ));
        durability_match_arms.push(format!(
            "            Block::{} => {:.1},",
            variant_name, durability
        ));
        key_match_arms.push(format!(
            "            Block::{} => Some(\"{}\"),",
            variant_name, key
        ));
        from_str_match_arms.push(format!(
            "            \"{}\" => Some(Block::{}),",
            key.to_lowercase(),
            variant_name
        ));

        // Generate drops match arm
        if let Some(drops_str) = table.get("drops").and_then(|v| v.as_str()) {
            let amount = table
                .get("amount")
                .and_then(|v| v.as_integer())
                .unwrap_or(1);
            drops_match_arms.push(format!(
                "            Block::{} => Some((crate::inventory::ItemType::from_str(\"{}\")?, {})),",
                variant_name, drops_str, amount
            ));
        } else {
            drops_match_arms.push(format!("            Block::{} => None,", variant_name));
        }

        // Generate type check match arms based on block_type
        match block_type {
            "solid" => {
                is_solid_match_arms.push(format!("            Block::{} => true,", variant_name));
                is_liquid_match_arms.push(format!("            Block::{} => false,", variant_name));
                is_spike_match_arms.push(format!("            Block::{} => false,", variant_name));
            }
            "liquid" => {
                is_solid_match_arms.push(format!("            Block::{} => false,", variant_name));
                is_liquid_match_arms.push(format!("            Block::{} => true,", variant_name));
                is_spike_match_arms.push(format!("            Block::{} => false,", variant_name));
            }
            "spike" => {
                is_solid_match_arms.push(format!("            Block::{} => false,", variant_name));
                is_liquid_match_arms.push(format!("            Block::{} => false,", variant_name));
                is_spike_match_arms.push(format!("            Block::{} => true,", variant_name));
            }
            "air" => {
                is_solid_match_arms.push(format!("            Block::{} => false,", variant_name));
                is_liquid_match_arms.push(format!("            Block::{} => false,", variant_name));
                is_spike_match_arms.push(format!("            Block::{} => false,", variant_name));
            }
            _ => panic!("Unknown block type '{}' for block '{}'", block_type, key),
        }

        all_blocks.push(format!("            Block::{},", variant_name));

        // Parse width and height (default to 1)
        let width = table.get("width").and_then(|v| v.as_integer()).unwrap_or(1) as usize;
        let height = table
            .get("height")
            .and_then(|v| v.as_integer())
            .unwrap_or(1) as usize;

        width_match_arms.push(format!("            Block::{} => {},", variant_name, width));
        height_match_arms.push(format!(
            "            Block::{} => {},",
            variant_name, height
        ));

        let is_multi = width > 1 || height > 1;
        is_multi_tile_match_arms.push(format!(
            "            Block::{} => {},",
            variant_name, is_multi
        ));

        // Check if this block has ore spawn data
        let has_ore_data = ores.as_ref().and_then(|o| o.get(key)).is_some();
        if has_ore_data {
            let ore_table = ores.as_ref().unwrap().get(key).unwrap().as_table().unwrap();
            let spawns_from = ore_table
                .get("spawns_from")
                .and_then(|v| v.as_integer())
                .unwrap_or(0);
            let spawns_peak = ore_table
                .get("spawns_peak")
                .and_then(|v| v.as_integer())
                .unwrap_or(100);
            let spawns_to = ore_table
                .get("spawns_to")
                .and_then(|v| v.as_integer())
                .unwrap_or(200);
            let spawns_pap = ore_table
                .get("spawns_pap")
                .and_then(|v| {
                    if let Some(f) = v.as_float() {
                        Some(f)
                    } else if let Some(i) = v.as_integer() {
                        Some(i as f64)
                    } else {
                        None
                    }
                })
                .unwrap_or(1.0);
            let min_vein_size = ore_table
                .get("min_vein_size")
                .and_then(|v| v.as_integer())
                .unwrap_or(2);
            let max_vein_size = ore_table
                .get("max_vein_size")
                .and_then(|v| v.as_integer())
                .unwrap_or(6);

            ore_spawn_match_arms.push(format!(
                "            Block::{} => Some(OreSpawnData {{ spawns_from: {}, spawns_peak: {}, spawns_to: {}, spawns_pap: {:.1}, min_vein_size: {}, max_vein_size: {} }}),",
                variant_name, spawns_from, spawns_peak, spawns_to, spawns_pap, min_vein_size, max_vein_size
            ));
        } else {
            ore_spawn_match_arms.push(format!("            Block::{} => None,", variant_name));
        }

        // Generate texture match arm
        if let Some(image_str) = table.get("image").and_then(|v| v.as_str()) {
            // Parse path like "tiles.dirt" or "items.stone"
            let texture_path = format!("textures.{}", image_str);
            texture_match_arms.push(format!(
                "            Block::{} => &{},",
                variant_name, texture_path
            ));
        } else {
            // Default to fallback texture
            texture_match_arms.push(format!(
                "            Block::{} => &textures.tiles.{},",
                variant_name, key
            ));
        }
    }

    let generated_code = format!(
        r#"// Generated block definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Block {{
{}
}}

/// Ore spawn configuration data
#[derive(Debug, Clone, Copy)]
pub struct OreSpawnData {{
    pub spawns_from: i32,
    pub spawns_peak: i32,
    pub spawns_to: i32,
    pub spawns_pap: f32,
    pub min_vein_size: i32,
    pub max_vein_size: i32,
}}

impl Block {{
    pub fn name(self) -> &'static str {{
        match self {{
{}
        }}
    }}

    pub fn durability(self) -> f32 {{
        match self {{
{}
        }}
    }}

    /// Returns the lowercase key used in blocks.toml for this block.
    pub fn key(self) -> Option<&'static str> {{
        match self {{
{}
        }}
    }}

    pub fn from_str(s: &str) -> Option<Self> {{
        match s.to_lowercase().as_str() {{
{}
            _ => None,
        }}
    }}

    /// Returns (item_type, amount) that this block drops when mined
    pub fn get_drops(self) -> Option<(crate::inventory::ItemType, u32)> {{
        match self {{
{}
        }}
    }}

    pub fn is_solid(self) -> bool {{
        match self {{
{}
        }}
    }}

    pub fn is_liquid(self) -> bool {{
        match self {{
{}
        }}
    }}

    pub fn is_spike(self) -> bool {{
        match self {{
{}
        }}
    }}

    pub fn all() -> &'static [Block] {{
        &[
{}
        ]
    }}

    /// Returns the texture for this block from the TextureManager.
    /// The texture path can be customized in blocks.toml with the 'image' field.
    /// By default, uses the fallback texture.
    pub fn get_texture<'a>(self, textures: &'a crate::TextureManager) -> &'a raylib::prelude::Texture2D {{
        match self {{
{}
        }}
    }}

    /// Returns the ore spawn data for this block, if it's an ore.
    /// Configured in blocks.toml under [ore.blockname] sections.
    pub fn get_ore_spawn_data(self) -> Option<OreSpawnData> {{
        match self {{
{}
        }}
    }}

    /// Returns the width of this block in tiles (default 1)
    pub fn width(self) -> usize {{
        match self {{
{}
        }}
    }}

    /// Returns the height of this block in tiles (default 1)
    pub fn height(self) -> usize {{
        match self {{
{}
        }}
    }}

    /// Returns true if this block occupies multiple tiles
    pub fn is_multi_tile(self) -> bool {{
        match self {{
{}
        }}
    }}
}}
"#,
        enum_variants.join("\n"),
        name_match_arms.join("\n"),
        durability_match_arms.join("\n"),
        key_match_arms.join("\n"),
        from_str_match_arms.join("\n"),
        drops_match_arms.join("\n"),
        is_solid_match_arms.join("\n"),
        is_liquid_match_arms.join("\n"),
        is_spike_match_arms.join("\n"),
        all_blocks.join("\n"),
        texture_match_arms.join("\n"),
        ore_spawn_match_arms.join("\n"),
        width_match_arms.join("\n"),
        height_match_arms.join("\n"),
        is_multi_tile_match_arms.join("\n")
    );

    fs::write(&dest_path, generated_code).expect("Failed to write generated blocks");
}

fn generate_block_mappings(out_dir: &str, game_data: &toml::Table) {
    let items = game_data
        .get("items")
        .and_then(|v| v.as_table())
        .expect("Missing [items] table in game data");

    let blocks = game_data
        .get("blocks")
        .and_then(|v| v.as_table())
        .expect("Missing [blocks] table in game data");

    // Get all block-type items
    let mut block_items: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for (key, value) in items.iter() {
        let table = value.as_table().expect("Item must be a table");
        if let Some(item_type) = table.get("type").and_then(|v| v.as_str()) {
            if item_type == "block" {
                block_items.insert(key.clone(), to_pascal_case(key));
            }
        }
    }

    // Generate Block::to_item_type() match arms (only for blocks that have items)
    let mut block_to_item_arms = Vec::new();
    for block_key in blocks.keys() {
        let variant = to_pascal_case(block_key);
        if block_items.contains_key(block_key) {
            block_to_item_arms.push(format!(
                "            Block::{} => Some(ItemType::{}),",
                variant, variant
            ));
        } else {
            block_to_item_arms.push(format!("            Block::{} => None,", variant));
        }
    }

    // Generate ItemType::to_block() match arms
    let mut item_to_block_arms = Vec::new();
    for variant in block_items.values() {
        item_to_block_arms.push(format!(
            "            ItemType::{} => Some(Block::{}),",
            variant, variant
        ));
    }

    // Generate the block mappings file
    let block_mappings_code = format!(
        r#"// Generated block-item mappings
impl Block {{
    /// Converts a block to its corresponding item type, if it has one.
    /// Some blocks (like Air, Water, Lava) don't have corresponding items.
    pub fn to_item_type(self) -> Option<ItemType> {{
        match self {{
{}
        }}
    }}

    pub fn from_item_type(item: ItemType) -> Option<Block> {{
        item.to_block()
    }}
}}

impl ItemType {{
    /// Converts an item to its corresponding block, if it's a block-type item.
    #[allow(unreachable_patterns)]
    pub fn to_block(self) -> Option<Block> {{
        match self {{
{}
            _ => None,
        }}
    }}
}}
"#,
        block_to_item_arms.join("\n"),
        item_to_block_arms.join("\n")
    );

    let block_mappings_path = Path::new(&out_dir).join("generated_block_mappings.rs");
    fs::write(&block_mappings_path, block_mappings_code)
        .expect("Failed to write generated block mappings");
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn process_recipe(
    key: &str,
    value: &toml::Value,
    available_items: &std::collections::HashSet<String>,
    classify_io: &dyn Fn(&str) -> String,
    recipe_definitions: &mut Vec<String>,
    skipped_recipes: &mut Vec<String>,
) {
    let table = value.as_table().expect("Recipe must be a table");

    let recipe_type = table
        .get("type")
        .and_then(|v| v.as_str())
        .expect(&format!("Recipe {} missing 'type' field", key));

    let output = table
        .get("output")
        .and_then(|v| v.as_str())
        .expect(&format!("Recipe {} missing 'output' field", key));

    let output_amount = table
        .get("amount")
        .and_then(|v| v.as_integer())
        .unwrap_or(1) as u32;

    let inputs = table
        .get("inputs")
        .and_then(|v| v.as_array())
        .expect(&format!("Recipe {} missing 'inputs' field", key));

    // Classify output
    let output_classification = classify_io(output);
    if output_classification == "unknown" {
        skipped_recipes.push(format!("{} (output '{}' not recognized)", key, output));
        return;
    }

    // Generate input definitions
    let mut input_defs = Vec::new();
    for input in inputs {
        let input_table = input.as_table().expect("Input must be a table");
        let input_type = input_table
            .get("type")
            .and_then(|v| v.as_str())
            .expect("Input missing 'type' field");
        let input_amount = input_table
            .get("amount")
            .and_then(|v| v.as_integer())
            .unwrap_or(1) as u32;

        let input_classification = classify_io(input_type);
        if input_classification == "unknown" {
            skipped_recipes.push(format!("{} (input '{}' not recognized)", key, input_type));
            return;
        }

        let input_normalized = input_type.replace(" ", "_");

        match input_classification.as_str() {
            "item" => {
                let input_item = to_pascal_case(&input_normalized);
                input_defs.push(format!(
                    "RecipeIOType::Item {{ item_type: ItemType::{}, amount: {} }}",
                    input_item, input_amount
                ));
            }
            "tool_pickaxe" => {
                input_defs.push(format!(
                    "RecipeIOType::ToolPickaxe {{ level: \"{}\" }}",
                    input_normalized
                ));
            }
            "tool_dash" => {
                input_defs.push(format!(
                    "RecipeIOType::ToolDash {{ level: \"{}\" }}",
                    input_normalized
                ));
            }
            "tool_glider" => {
                input_defs.push(format!(
                    "RecipeIOType::ToolGlider {{ level: \"{}\" }}",
                    input_normalized
                ));
            }
            "tool_tideclock" => {
                input_defs.push("RecipeIOType::ToolTideClock".to_string());
            }
            _ => {}
        }
    }

    let recipe_type_enum = match recipe_type {
        "always" => "RecipeType::Always",
        "workbench" => "RecipeType::Workbench",
        "furnace" => "RecipeType::Furnace",
        "anvil" => "RecipeType::Anvil",
        _ => panic!("Unknown recipe type: {}", recipe_type),
    };

    // Generate output definition
    let output_normalized = output.replace(" ", "_");
    let output_def = match output_classification.as_str() {
        "item" => {
            let output_item = to_pascal_case(&output_normalized);
            format!(
                "RecipeIOType::Item {{ item_type: ItemType::{}, amount: {} }}",
                output_item, output_amount
            )
        }
        "tool_pickaxe" => {
            format!(
                "RecipeIOType::ToolPickaxe {{ level: \"{}\" }}",
                output_normalized
            )
        }
        "tool_dash" => {
            format!(
                "RecipeIOType::ToolDash {{ level: \"{}\" }}",
                output_normalized
            )
        }
        "tool_glider" => {
            format!(
                "RecipeIOType::ToolGlider {{ level: \"{}\" }}",
                output_normalized
            )
        }
        "tool_tideclock" => "RecipeIOType::ToolTideClock".to_string(),
        _ => return,
    };

    recipe_definitions.push(format!(
        r#"    Recipe {{
        name: "{}",
        recipe_type: {},
        inputs: &[{}],
        output: {},
    }}"#,
        key,
        recipe_type_enum,
        input_defs.join(", "),
        output_def
    ));
}

fn generate_recipes(out_dir: &str, game_data: &toml::Table) {
    let dest_path = Path::new(&out_dir).join("generated_recipes.rs");

    // Get both regular recipes and tool recipes
    let recipes = game_data.get("recipes").and_then(|v| v.as_table());

    let tool_recipes = game_data.get("tool_recipes").and_then(|v| v.as_table());

    // Get all available items to validate recipes
    let items = game_data
        .get("items")
        .and_then(|v| v.as_table())
        .expect("Missing [items] table in game data");
    let available_items: std::collections::HashSet<String> = items.keys().cloned().collect();

    // Read tool names from tools.toml
    let tools_toml_path = Path::new("assets/data/tools.toml");
    let tools_content = fs::read_to_string(tools_toml_path).expect("Failed to read tools.toml");
    let tools_data: toml::Table = toml::from_str(&tools_content).expect("Failed to parse tools.toml");
    
    let mut tool_names = Vec::new();
    
    // Extract pickaxe names
    if let Some(pick_table) = tools_data.get("pick").and_then(|v| v.as_table()) {
        for (level, data) in pick_table.iter() {
            if let Some(name) = data.as_table().and_then(|t| t.get("name")).and_then(|n| n.as_str()) {
                tool_names.push(format!("            \"{}\" => \"{}\",", level, name));
            }
        }
    }
    
    // Extract dash names
    if let Some(dash_table) = tools_data.get("dash").and_then(|v| v.as_table()) {
        for (level, data) in dash_table.iter() {
            if let Some(name) = data.as_table().and_then(|t| t.get("name")).and_then(|n| n.as_str()) {
                tool_names.push(format!("            \"{}\" => \"{}\",", level, name));
            }
        }
    }

    // Extract glider names
    if let Some(glider_table) = tools_data.get("glider").and_then(|v| v.as_table()) {
        for (level, data) in glider_table.iter() {
            if let Some(name) = data.as_table().and_then(|t| t.get("name")).and_then(|n| n.as_str()) {
                tool_names.push(format!("            \"{}\" => \"{}\",", level, name));
            }
        }
    }

    let mut recipe_definitions = Vec::new();
    let mut skipped_recipes = Vec::new();

    // Helper function to classify input/output type
    let classify_io = |name: &str| -> String {
        let normalized = name.replace(" ", "_");

        // Check if it's a regular item
        if available_items.contains(&normalized) {
            return String::from("item");
        }

        // Check if it's a tool by name pattern
        if normalized.contains("pickaxe") {
            return String::from("tool_pickaxe");
        }
        if normalized.contains("amulet") {
            return String::from("tool_dash");
        }
        if normalized.contains("glider") {
            return String::from("tool_glider");
        }
        if normalized == "tidalcave_clock" {
            return String::from("tool_tideclock");
        }

        String::from("unknown")
    };

    // Process regular recipes
    if let Some(recipes) = recipes {
        for (key, value) in recipes.iter() {
            process_recipe(
                key,
                value,
                &available_items,
                &classify_io,
                &mut recipe_definitions,
                &mut skipped_recipes,
            );
        }
    }

    // Process tool recipes
    if let Some(tool_recipes) = tool_recipes {
        for (key, value) in tool_recipes.iter() {
            process_recipe(
                key,
                value,
                &available_items,
                &classify_io,
                &mut recipe_definitions,
                &mut skipped_recipes,
            );
        }
    }

    if !skipped_recipes.is_empty() {
        println!(
            "cargo:warning=Skipped {} recipes due to missing items:",
            skipped_recipes.len()
        );
        for skipped in &skipped_recipes {
            println!("cargo:warning=  - {}", skipped);
        }
    }

    let generated_code = format!(
        r#"// Generated recipe definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeType {{
    Workbench,
    Furnace,
    Anvil,
    Always
}}

#[derive(Debug, Clone, Copy)]
pub enum RecipeIOType {{
    Item {{ item_type: ItemType, amount: u32 }},
    ToolPickaxe {{ level: &'static str }},
    ToolDash {{ level: &'static str }},
    ToolGlider {{ level: &'static str }},
    ToolTideClock,
}}

impl RecipeIOType {{
    /// Returns the display name for this recipe input/output
    pub fn name(&self) -> &'static str {{
        match self {{
            RecipeIOType::Item {{ item_type, .. }} => item_type.name(),
            RecipeIOType::ToolPickaxe {{ level }} | RecipeIOType::ToolDash {{ level }} | RecipeIOType::ToolGlider {{ level }} => {{
                match *level {{
{}
                    _ => level,
                }}
            }}
            RecipeIOType::ToolTideClock => "Tidalcave Clock",
        }}
    }}

    /// Returns the amount for this recipe input/output
    pub fn amount(&self) -> u32 {{
        match self {{
            RecipeIOType::Item {{ amount, .. }} => *amount,
            RecipeIOType::ToolPickaxe {{ .. }} => 1,
            RecipeIOType::ToolDash {{ .. }} => 1,
            RecipeIOType::ToolGlider {{ .. }} => 1,
            RecipeIOType::ToolTideClock => 1,
        }}
    }}

    /// Returns the texture for this recipe input/output
    pub fn get_texture<'a>(&self, textures: &'a crate::TextureManager) -> &'a raylib::prelude::Texture2D {{
        match self {{
            RecipeIOType::Item {{ item_type, .. }} => item_type.get_texture(textures),
            RecipeIOType::ToolPickaxe {{ level }} => {{
                crate::tools::ToolPickaxe::texture_for_level(level, textures)
            }}
            RecipeIOType::ToolDash {{ level }} => {{
                crate::tools::ToolDash::texture_for_level(level, textures)
            }}
            RecipeIOType::ToolGlider {{ level }} => {{
                crate::tools::ToolGlider::texture_for_level(level, textures)
            }}
            RecipeIOType::ToolTideClock => {{
                &textures.items.tidalcave_clock
            }}
        }}
    }}

    /// Check if the player has this input available
    pub fn player_has(&self, player: &crate::player::Player) -> bool {{
        match self {{
            RecipeIOType::Item {{ item_type, amount }} => {{
                player.inventory.count(*item_type) >= *amount
            }}
            RecipeIOType::ToolPickaxe {{ level }} => {{
                player.tool_pickaxe.as_ref().map(|p| p.level == *level).unwrap_or(false)
            }}
            RecipeIOType::ToolDash {{ level }} => {{
                player.tool_dash.as_ref().map(|d| d.level == *level).unwrap_or(false)
            }}
            RecipeIOType::ToolGlider {{ level }} => {{
                player.tool_glider.as_ref().map(|g| g.level == *level).unwrap_or(false)
            }}
            RecipeIOType::ToolTideClock => {{
                player.tool_tideclock.is_some()
            }}
        }}
    }}

    /// Get display text with current count for UI
    pub fn display_with_count(&self, player: &crate::player::Player) -> String {{
        match self {{
            RecipeIOType::Item {{ item_type, amount }} => {{
                let count = player.inventory.count(*item_type);
                format!("{{}}x {{}} ({{}})", amount, item_type.name(), count)
            }}
            RecipeIOType::ToolPickaxe {{ level }} => {{
                format!("1x {{}} (tool)", level)
            }}
            RecipeIOType::ToolDash {{ level }} => {{
                format!("1x {{}} (tool)", level)
            }}
            RecipeIOType::ToolGlider {{ level }} => {{
                format!("1x {{}} (tool)", level)
            }}
            RecipeIOType::ToolTideClock => {{
                "1x Tidalcave Clock (tool)".to_string()
            }}
        }}
    }}
}}

#[derive(Debug, Clone)]
pub struct Recipe {{
    pub name: &'static str,
    pub recipe_type: RecipeType,
    pub inputs: &'static [RecipeIOType],
    pub output: RecipeIOType,
}}

impl Recipe {{
    /// Check if the player has the necessary items and tools to craft this recipe
    pub fn can_craft(&self, player: &crate::player::Player) -> bool {{
        for input in self.inputs {{
            match input {{
                RecipeIOType::Item {{ item_type, amount }} => {{
                    if player.inventory.count(*item_type) < *amount {{
                        return false;
                    }}
                }}
                RecipeIOType::ToolPickaxe {{ level }} => {{
                    match &player.tool_pickaxe {{
                        Some(pickaxe) if pickaxe.level == *level => {{}},
                        _ => return false,
                    }}
                }}
                RecipeIOType::ToolDash {{ level }} => {{
                    match &player.tool_dash {{
                        Some(dash) if dash.level == *level => {{}},
                        _ => return false,
                    }}
                }}
                RecipeIOType::ToolGlider {{ level }} => {{
                    match &player.tool_glider {{
                        Some(glider) if glider.level == *level => {{}},
                        _ => return false,
                    }}
                }}
                RecipeIOType::ToolTideClock => {{
                    if player.tool_tideclock.is_none() {{
                        return false;
                    }}
                }}
            }}
        }}
        true
    }}

    /// Craft this recipe, consuming ingredients and tools, producing output
    /// Returns true if successful, false if couldn't craft
    pub fn craft(&self, player: &mut crate::player::Player) -> bool {{
        // Check again to be safe
        if !self.can_craft(player) {{
            return false;
        }}

        // Remove inputs
        for input in self.inputs {{
            if let RecipeIOType::Item {{ item_type, amount }} = input {{
                player.inventory.take(*item_type, *amount);
            }}
            // Tools are consumed implicitly (replaced by output)
        }}

        // Add output (auto-equip tools)
        match self.output {{
            RecipeIOType::Item {{ item_type, amount }} => {{
                player.inventory.add(item_type, amount);
            }}
            RecipeIOType::ToolPickaxe {{ level }} => {{
                player.tool_pickaxe = Some(crate::tools::load_pick(level));
            }}
            RecipeIOType::ToolDash {{ level }} => {{
                player.tool_dash = Some(crate::tools::load_dash(level));
            }}
            RecipeIOType::ToolGlider {{ level }} => {{
                player.tool_glider = Some(crate::tools::load_glider(level));
            }}
            RecipeIOType::ToolTideClock => {{
                player.tool_tideclock = Some(crate::tools::ToolTideClock::new());
            }}
        }}

        true
    }}

    /// Check if this recipe is a repair (tool input with same level as tool output)
    pub fn is_repair(&self) -> bool {{
        match &self.output {{
            RecipeIOType::ToolPickaxe {{ level: out_level }} => {{
                self.inputs.iter().any(|input| {{
                    matches!(input, RecipeIOType::ToolPickaxe {{ level }} if level == out_level)
                }})
            }}
            RecipeIOType::ToolDash {{ level: out_level }} => {{
                self.inputs.iter().any(|input| {{
                    matches!(input, RecipeIOType::ToolDash {{ level }} if level == out_level)
                }})
            }}
            RecipeIOType::ToolGlider {{ level: out_level }} => {{
                self.inputs.iter().any(|input| {{
                    matches!(input, RecipeIOType::ToolGlider {{ level }} if level == out_level)
                }})
            }}
            _ => false,
        }}
    }}

    /// Check if this recipe is an upgrade (tool input with different level than tool output)
    pub fn is_upgrade(&self) -> bool {{
        match &self.output {{
            RecipeIOType::ToolPickaxe {{ level: out_level }} => {{
                self.inputs.iter().any(|input| {{
                    matches!(input, RecipeIOType::ToolPickaxe {{ level }} if level != out_level)
                }})
            }}
            RecipeIOType::ToolDash {{ level: out_level }} => {{
                self.inputs.iter().any(|input| {{
                    matches!(input, RecipeIOType::ToolDash {{ level }} if level != out_level)
                }})
            }}
            RecipeIOType::ToolGlider {{ level: out_level }} => {{
                self.inputs.iter().any(|input| {{
                    matches!(input, RecipeIOType::ToolGlider {{ level }} if level != out_level)
                }})
            }}
            _ => false,
        }}
    }}

    /// Get display name for the recipe list (prefixed with "Repair - " or "Upgrade - " as appropriate)
    pub fn display_name(&self) -> String {{
        if self.is_repair() {{
            format!("Repair - {{}}", self.output.name())
        }} else if self.is_upgrade() {{
            format!("Upgrade - {{}}", self.output.name())
        }} else {{
            format!("{{}}x {{}}", self.output.amount(), self.output.name())
        }}
    }}
}}

pub static ALL_RECIPES: &[Recipe] = &[
{}
];
"#,
        tool_names.join("\n"),
        recipe_definitions.join(",\n")
    );

    fs::write(&dest_path, generated_code).expect("Failed to write generated recipes");
}
