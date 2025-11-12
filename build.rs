use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/data/items.toml");
    println!("cargo:rerun-if-changed=assets/data/blocks.toml");

    let out_dir = env::var("OUT_DIR").unwrap();

    // Generate items
    generate_items(&out_dir);

    // Generate blocks
    generate_blocks(&out_dir);

    // Generate block-item mappings
    generate_block_mappings(&out_dir);
}

fn generate_items(out_dir: &str) {
    let dest_path = Path::new(&out_dir).join("generated_items.rs");

    let toml_content =
        fs::read_to_string("assets/data/items.toml").expect("Failed to read items.toml");

    let items_table: toml::Table =
        toml::from_str(&toml_content).expect("Failed to parse items.toml");

    let items = items_table
        .get("items")
        .and_then(|v| v.as_table())
        .expect("Missing [items] table in items.toml");

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
        r#"#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

fn generate_blocks(out_dir: &str) {
    let dest_path = Path::new(&out_dir).join("generated_blocks.rs");

    let toml_content =
        fs::read_to_string("assets/data/blocks.toml").expect("Failed to read blocks.toml");

    let blocks_table: toml::Table =
        toml::from_str(&toml_content).expect("Failed to parse blocks.toml");

    let blocks = blocks_table
        .get("blocks")
        .and_then(|v| v.as_table())
        .expect("Missing [blocks] table in blocks.toml");

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
    }

    let generated_code = format!(
        r#"// Generated block definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {{
{}
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
        all_blocks.join("\n")
    );

    fs::write(&dest_path, generated_code).expect("Failed to write generated blocks");
}

fn generate_block_mappings(out_dir: &str) {
    let items_content =
        fs::read_to_string("assets/data/items.toml").expect("Failed to read items.toml");
    let items_table: toml::Table =
        toml::from_str(&items_content).expect("Failed to parse items.toml");
    let items = items_table
        .get("items")
        .and_then(|v| v.as_table())
        .expect("Missing [items] table in items.toml");

    let blocks_content =
        fs::read_to_string("assets/data/blocks.toml").expect("Failed to read blocks.toml");
    let blocks_table: toml::Table =
        toml::from_str(&blocks_content).expect("Failed to parse blocks.toml");
    let blocks = blocks_table
        .get("blocks")
        .and_then(|v| v.as_table())
        .expect("Missing [blocks] table in blocks.toml");

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
