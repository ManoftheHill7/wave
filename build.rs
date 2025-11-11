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
