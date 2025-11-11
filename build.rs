use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/data/items.toml");
    println!("cargo:rerun-if-changed=assets/data/blocks.toml");

    let out_dir = env::var("OUT_DIR").unwrap();
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
}}
"#,
        enum_variants.join("\n"),
        name_match_arms.join("\n"),
        weight_match_arms.join("\n"),
        from_str_match_arms.join("\n"),
        all_items.join("\n")
    );

    fs::write(&dest_path, generated_code).expect("Failed to write generated items");

    // Generate block-item mappings
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

    // Validate: every block has a corresponding item
    for (block_key, _) in blocks.iter() {
        if !block_items.contains_key(block_key) {
            panic!(
                "Block '{}' in blocks.toml has no corresponding item with type='block' in items.toml. \
                All blocks must have a matching item!",
                block_key
            );
        }
    }

    // Generate Block::to_item_type() match arms
    let mut block_to_item_arms = Vec::new();
    let mut block_key_arms = Vec::new();
    for block_key in blocks.keys() {
        let variant = to_pascal_case(block_key);
        block_to_item_arms.push(format!(
            "            Block::{} => ItemType::{},",
            variant, variant
        ));
        block_key_arms.push(format!(
            "            Block::{} => Some(\"{}\"),",
            variant, block_key
        ));
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
    /// Returns the lowercase key used in blocks.toml for this block.
    /// This is used to look up block properties like durability and drops.
    pub fn key(self) -> Option<&'static str> {{
        match self {{
{}
            _ => None,
        }}
    }}

    /// Converts a block to its corresponding item type.
    /// All blocks in blocks.toml are guaranteed to have a matching item.
    pub fn to_item_type(self) -> ItemType {{
        match self {{
{}
            _ => panic!("Block {{:?}} has no corresponding item type. All blocks should be defined in blocks.toml with matching items.", self),
        }}
    }}
    
    pub fn from_item_type(item: ItemType) -> Option<Block> {{
        item.to_block()
    }}
}}

impl ItemType {{
    /// Converts an item to its corresponding block, if it's a block-type item.
    pub fn to_block(self) -> Option<Block> {{
        match self {{
{}
            _ => None,
        }}
    }}
}}
"#,
        block_key_arms.join("\n"),
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
