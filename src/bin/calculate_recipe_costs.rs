use std::collections::HashMap;
use std::fs;
use wave_github_gameoff2025::terrain::CHUNK_SIZE;

#[derive(Debug, Clone)]
struct RecipeInput {
    item_type: String,
    amount: u32,
}

#[derive(Debug, Clone)]
struct Recipe {
    output: String,
    inputs: Vec<RecipeInput>,
}

#[derive(Debug, Clone)]
struct OreGeneration {
    spawns_from: i32,
    spawns_peak: i32,
    spawns_to: i32,
    spawns_pap: f32, // Probability at peak (percentage)
    min_vein_size: u32,
    max_vein_size: u32,
}

fn main() {
    // Load all recipe files
    let workbench_recipes = load_recipes("assets/data/workbench_recipes.toml");
    let anvil_recipes = load_recipes("assets/data/anvil_recipes.toml");
    let furnace_recipes = load_recipes("assets/data/furnace_recipes.toml");

    // Combine all recipes into one map
    let mut all_recipes: HashMap<String, Recipe> = HashMap::new();
    all_recipes.extend(workbench_recipes);
    all_recipes.extend(anvil_recipes);
    all_recipes.extend(furnace_recipes);

    // Load ore generation data
    let ore_gen = load_ore_generation("assets/data/ore_generation.toml");

    // Define upgrade paths for each tool type
    let pickaxe_upgrades = vec![
        ("stone_pickaxe", "Stone Pickaxe"),
        ("copper_pickaxe", "Copper Pickaxe"),
        ("bronze_pickaxe", "Bronze Pickaxe"),
        ("iron_pickaxe", "Iron Pickaxe"),
        ("steel_pickaxe", "Steel Pickaxe"),
        ("platinum_pickaxe", "Platinum Pickaxe"),
        ("mithril_pickaxe", "Mithril Pickaxe"),
        ("ozathinum_pickaxe", "Ozathinum Pickaxe"),
        ("torzite_pickaxe", "Torzite Pickaxe"),
        ("etherealite_pickaxe", "Etherealite Pickaxe"),
    ];

    let amulet_upgrades = vec![
        ("white_pearl_amulet", "White Pearl Amulet"),
        ("amethyst_amulet", "Amethyst Amulet"),
        ("jasper_amulet", "Jasper Amulet"),
        ("emerald_amulet", "Emerald Amulet"),
        ("black_pearl_amulet", "Black Pearl Amulet"),
        ("topaz_amulet", "Topaz Amulet"),
        ("ruby_amulet", "Ruby Amulet"),
        ("diamond_amulet", "Diamond Amulet"),
    ];

    let glider_upgrades = vec![
        ("linen_glider", "Linen Glider"),
        ("pearl_glider", "Pearl Glider"),
        ("amethyst_glider", "Amethyst Glider"),
        ("emerald_glider", "Emerald Glider"),
        ("topaz_glider", "Topaz Glider"),
        ("ruby_glider", "Ruby Glider"),
        ("diamond_glider", "Diamond Glider"),
    ];

    println!("=== CUMULATIVE RESOURCE COSTS BY UPGRADE LEVEL ===\n");

    // Print pickaxe upgrades
    println!("PICKAXE UPGRADES:");
    println!("=================\n");
    print_upgrade_path(&pickaxe_upgrades, &all_recipes);

    // Print amulet upgrades
    println!("\nAMULET UPGRADES:");
    println!("================\n");
    print_upgrade_path(&amulet_upgrades, &all_recipes);

    // Print glider upgrades
    println!("\nGLIDER UPGRADES:");
    println!("================\n");
    print_upgrade_path(&glider_upgrades, &all_recipes);

    // Print other tools
    println!("\nOTHER TOOLS:");
    println!("============\n");
    let other_tools = vec![
        ("tidalcave_clock", "Tidalcave Clock"),
    ];
    for (tool_id, tool_name) in other_tools {
        println!("{}:", tool_name);
        let costs = calculate_total_cost(tool_id, &all_recipes, &mut HashMap::new());
        print_costs(&costs);
        println!();
    }

    println!("\n=== ESTIMATED CHUNKS TO EXPLORE ===\n");
    
    for (tool_id, tool_name) in &[
        ("etherealite_pickaxe", "Etherealite Pickaxe"),
        ("diamond_amulet", "Diamond Amulet"),
    ] {
        println!("{}:", tool_name);
        println!("{}", "=".repeat(tool_name.len() + 1));
        
        let costs = calculate_total_cost(tool_id, &all_recipes, &mut HashMap::new());
        estimate_chunks_needed(&costs, &ore_gen);
        println!();
    }
}

fn calculate_total_cost(
    item: &str,
    recipes: &HashMap<String, Recipe>,
    memo: &mut HashMap<String, HashMap<String, u32>>,
) -> HashMap<String, u32> {
    // Check memo cache
    if let Some(cached) = memo.get(item) {
        return cached.clone();
    }

    let mut total_costs: HashMap<String, u32> = HashMap::new();

    // If there's no recipe, this is a base resource
    if let Some(recipe) = recipes.get(item) {
        // Recursively calculate cost for each input
        for input in &recipe.inputs {
            // Skip self-referential inputs (repairs)
            if input.item_type == item {
                continue;
            }

            // Recursively expand this input (whether it has a recipe or not)
            let sub_costs = calculate_total_cost(&input.item_type, recipes, memo);
            
            if sub_costs.is_empty() {
                // This is a base resource (no recipe and no sub-costs)
                *total_costs.entry(input.item_type.clone()).or_insert(0) += input.amount;
            } else {
                // Add the sub-costs, multiplied by the amount needed
                for (sub_item, sub_amount) in sub_costs {
                    *total_costs.entry(sub_item).or_insert(0) += sub_amount * input.amount;
                }
            }
        }
    }
    // If no recipe exists and we're at base level, return empty (caller will add it)

    memo.insert(item.to_string(), total_costs.clone());
    total_costs
}

fn load_recipes(path: &str) -> HashMap<String, Recipe> {
    let mut recipes = HashMap::new();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Could not read {}: {}", path, e);
            return recipes;
        }
    };

    let toml_data: toml::Value = match toml::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Warning: Could not parse {}: {}", path, e);
            return recipes;
        }
    };

    // Parse regular recipes
    if let Some(recipes_table) = toml_data.get("recipes").and_then(|v| v.as_table()) {
        for (key, value) in recipes_table {
            if let Some(recipe) = parse_recipe(key, value) {
                recipes.insert(recipe.output.clone(), recipe);
            }
        }
    }

    // Parse tool recipes
    if let Some(tool_recipes_table) = toml_data.get("tool_recipes").and_then(|v| v.as_table()) {
        for (key, value) in tool_recipes_table {
            if let Some(recipe) = parse_recipe(key, value) {
                recipes.insert(recipe.output.clone(), recipe);
            }
        }
    }

    recipes
}

fn normalize_name(name: &str) -> String {
    // Normalize spaces to underscores for consistent lookups
    name.replace(" ", "_")
}

fn load_ore_generation(path: &str) -> HashMap<String, OreGeneration> {
    let mut ore_gen = HashMap::new();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Could not read {}: {}", path, e);
            return ore_gen;
        }
    };

    let toml_data: toml::Value = match toml::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Warning: Could not parse {}: {}", path, e);
            return ore_gen;
        }
    };

    if let Some(ore_table) = toml_data.get("ore").and_then(|v| v.as_table()) {
        for (key, value) in ore_table {
            if let Some(table) = value.as_table() {
                let gen = OreGeneration {
                    spawns_from: table.get("spawns_from").and_then(|v| v.as_integer()).unwrap_or(0) as i32,
                    spawns_peak: table.get("spawns_peak").and_then(|v| v.as_integer()).unwrap_or(0) as i32,
                    spawns_to: table.get("spawns_to").and_then(|v| v.as_integer()).unwrap_or(0) as i32,
                    spawns_pap: table.get("spawns_pap").and_then(|v| v.as_float()).unwrap_or(0.0) as f32,
                    min_vein_size: table.get("min_vein_size").and_then(|v| v.as_integer()).unwrap_or(1) as u32,
                    max_vein_size: table.get("max_vein_size").and_then(|v| v.as_integer()).unwrap_or(1) as u32,
                };
                ore_gen.insert(key.clone(), gen);
            }
        }
    }

    ore_gen
}

fn estimate_chunks_needed(costs: &HashMap<String, u32>, ore_gen: &HashMap<String, OreGeneration>) {
    // The ore generation probability is per chunk
    let blocks_per_chunk = (CHUNK_SIZE * CHUNK_SIZE) as f32;
    
    println!("  Ore generation probability is per chunk ({}x{} blocks = {} blocks):\n", 
             CHUNK_SIZE, CHUNK_SIZE, blocks_per_chunk);

    // Group ores by depth range
    let mut depth_ranges: HashMap<&str, Vec<(&str, u32, &OreGeneration)>> = HashMap::new();
    
    for (item_name, &amount) in costs {
        if let Some(gen) = ore_gen.get(item_name) {
            let depth_label = match gen.spawns_peak {
                0..=100 => "Shallow (0-100)",
                101..=300 => "Early (100-300)",
                301..=500 => "Mid (300-500)",
                501..=700 => "Deep (500-700)",
                701..=850 => "Very Deep (700-850)",
                _ => "Extreme (850+)",
            };
            
            depth_ranges.entry(depth_label)
                .or_insert_with(Vec::new)
                .push((item_name.as_str(), amount, gen));
        }
    }

    // Sort depth ranges by depth
    let mut sorted_ranges: Vec<_> = depth_ranges.into_iter().collect();
    sorted_ranges.sort_by_key(|(label, _)| match *label {
        "Shallow (0-100)" => 0,
        "Early (100-300)" => 1,
        "Mid (300-500)" => 2,
        "Deep (500-700)" => 3,
        "Very Deep (700-850)" => 4,
        "Extreme (850+)" => 5,
        _ => 999,
    });

    let mut total_chunks_by_depth: Vec<(&str, f32)> = Vec::new();
    
    for (depth_label, ores) in sorted_ranges {
        println!("  {} depth:", depth_label);
        
        let mut max_chunks_this_depth = 0.0_f32;
        
        for (ore_name, amount, gen) in ores {
            // Calculate average vein size
            let avg_vein_size = (gen.min_vein_size + gen.max_vein_size) as f32 / 2.0;
            
            // Probability per chunk at peak depth (as decimal)
            let prob_per_chunk = gen.spawns_pap / 100.0;
            
            // Expected ore per chunk at peak depth
            // Each chunk has 'prob_per_chunk' chance to spawn a vein of 'avg_vein_size'
            let expected_ore_per_chunk = prob_per_chunk * avg_vein_size;
            
            // Chunks needed (at peak depth)
            let chunks_needed = if expected_ore_per_chunk > 0.0 {
                (amount as f32 / expected_ore_per_chunk).ceil()
            } else {
                f32::INFINITY
            };
            
            max_chunks_this_depth = max_chunks_this_depth.max(chunks_needed);
            
            println!("    {} x {} - ~{:.0} chunks (depth {}-{})",
                amount, ore_name, chunks_needed, gen.spawns_from, gen.spawns_to);
        }
        
        total_chunks_by_depth.push((depth_label, max_chunks_this_depth));
        println!();
    }
    
    println!("  Summary:");
    for (depth_label, max_chunks) in total_chunks_by_depth {
        if max_chunks.is_finite() && max_chunks > 0.0 {
            println!("    {} - explore ~{:.0} chunks", depth_label, max_chunks);
        }
    }
}

fn parse_recipe(key: &str, value: &toml::Value) -> Option<Recipe> {
    let table = value.as_table()?;
    
    let output = table
        .get("output")
        .and_then(|v| v.as_str())
        .unwrap_or(key);
    let output = normalize_name(output);

    let inputs_array = table.get("inputs").and_then(|v| v.as_array())?;
    let mut inputs = Vec::new();

    for input in inputs_array {
        let input_table = input.as_table()?;
        let item_type = input_table.get("type").and_then(|v| v.as_str())?;
        let item_type = normalize_name(item_type);
        let amount = input_table.get("amount").and_then(|v| v.as_integer())? as u32;
        
        inputs.push(RecipeInput { item_type, amount });
    }

    Some(Recipe { output, inputs })
}

fn print_upgrade_path(upgrades: &[(&str, &str)], recipes: &HashMap<String, Recipe>) {
    for (tool_id, tool_name) in upgrades {
        println!("{}:", tool_name);
        let costs = calculate_total_cost(tool_id, recipes, &mut HashMap::new());
        print_costs(&costs);
        println!();
    }
}

fn print_costs(costs: &HashMap<String, u32>) {
    if costs.is_empty() {
        println!("  (no recipe or base item)");
        return;
    }

    // Sort by amount (descending) for better readability
    let mut sorted_costs: Vec<_> = costs.iter().collect();
    sorted_costs.sort_by(|a, b| b.1.cmp(a.1));
    
    for (item, amount) in sorted_costs {
        println!("  {} x {}", amount, item);
    }
}
