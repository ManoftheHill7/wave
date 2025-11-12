use std::path::Path;
use wave_github_gameoff2025::maps::{MapEdges, MapOrientation, MapSet};

fn main() {
    let maps_dir = Path::new("assets/maps");

    if !maps_dir.exists() {
        eprintln!("Error: assets/maps directory not found");
        std::process::exit(1);
    }

    // Load all maps using the new MapSet
    let map_set = match MapSet::load_from_directory(maps_dir) {
        Ok(set) => set,
        Err(e) => {
            eprintln!("Error loading maps: {}", e);
            std::process::exit(1);
        }
    };

    // Get statistics
    let stats = map_set.edge_pattern_stats();

    // Print horizontal maps analysis
    println!("\n=== Horizontal Maps (64x32) Edge Analysis ===");
    println!(
        "Total unique edge groups: {}",
        stats.unique_horizontal_patterns
    );

    // Get horizontal maps
    let horizontal_maps: Vec<_> = map_set
        .find_matching(
            &wave_github_gameoff2025::maps::MapQuery::new()
                .with_orientation(MapOrientation::Horizontal),
        )
        .into_iter()
        .collect();

    let mut horizontal_groups: Vec<(MapEdges, Vec<String>)> = stats
        .pattern_distribution
        .iter()
        .filter_map(|(edges, maps)| {
            // Only include maps that are horizontal
            let horizontal_only: Vec<String> = maps
                .iter()
                .filter(|name| horizontal_maps.iter().any(|m| &m.name == *name))
                .cloned()
                .collect();

            if horizontal_only.is_empty() {
                None
            } else {
                Some((*edges, horizontal_only))
            }
        })
        .collect();

    horizontal_groups.sort_by_key(|(_, maps)| std::cmp::Reverse(maps.len()));

    for (edges, maps) in &horizontal_groups {
        print_edge_group(edges, maps, MapOrientation::Horizontal);
    }

    // Print vertical maps analysis
    println!("\n=== Vertical Maps (32x64) Edge Analysis ===");
    println!(
        "Total unique edge groups: {}",
        stats.unique_vertical_patterns
    );

    // Get vertical maps
    let vertical_maps: Vec<_> = map_set
        .find_matching(
            &wave_github_gameoff2025::maps::MapQuery::new()
                .with_orientation(MapOrientation::Vertical),
        )
        .into_iter()
        .collect();

    let mut vertical_groups: Vec<(MapEdges, Vec<String>)> = stats
        .pattern_distribution
        .iter()
        .filter_map(|(edges, maps)| {
            // Only include maps that are vertical
            let vertical_only: Vec<String> = maps
                .iter()
                .filter(|name| vertical_maps.iter().any(|m| &m.name == *name))
                .cloned()
                .collect();

            if vertical_only.is_empty() {
                None
            } else {
                Some((*edges, vertical_only))
            }
        })
        .collect();

    vertical_groups.sort_by_key(|(_, maps)| std::cmp::Reverse(maps.len()));

    for (edges, maps) in &vertical_groups {
        print_edge_group(edges, maps, MapOrientation::Vertical);
    }

    // Print summary
    println!("\n=== Summary ===");
    println!("Total horizontal maps: {}", stats.horizontal_maps);
    println!("Total vertical maps:   {}", stats.vertical_maps);
}

fn print_edge_group(edges: &MapEdges, maps: &[String], orientation: MapOrientation) {
    println!("\nEdge Group:");

    let edge_names = MapEdges::edge_names(orientation);
    for (i, name) in edge_names.iter().enumerate() {
        println!("  {:<13} {}", format!("{}:", name), edges.edges[i]);
    }

    println!("  Maps in group: {}", maps.len());
    for map in maps {
        println!("    - {}", map);
    }
}
