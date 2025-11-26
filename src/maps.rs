use rand::Rng;
use raylib::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeType {
    Cavern,       // value: 0
    Tunnel,       // values: 1073618943 or 1073680383 (tunnel template has slight variations)
    Unknown(u32), // any other bit pattern
}

impl EdgeType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => EdgeType::Cavern,
            1073618943 | 1073680383 => EdgeType::Tunnel,
            v => EdgeType::Unknown(v),
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            EdgeType::Cavern => 0,
            EdgeType::Tunnel => 1073618943, // Use the most common value
            EdgeType::Unknown(v) => *v,
        }
    }

    pub fn is_compatible(&self, other: &EdgeType) -> bool {
        // Two edges are compatible if they match exactly
        self == other
    }
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::Cavern => write!(f, "Cavern"),
            EdgeType::Tunnel => write!(f, "Tunnel"),
            EdgeType::Unknown(v) => write!(f, "Unknown({})", v),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapOrientation {
    Horizontal, // 64x32
    Vertical,   // 32x64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapEdges {
    pub edges: [EdgeType; 6],
}

impl MapEdges {
    pub fn new(edges: [EdgeType; 6]) -> Self {
        MapEdges { edges }
    }

    pub fn edge_names(orientation: MapOrientation) -> [&'static str; 6] {
        match orientation {
            MapOrientation::Horizontal => [
                "Top Left",
                "Top Right",
                "Bottom Left",
                "Bottom Right",
                "Left",
                "Right",
            ],
            MapOrientation::Vertical => [
                "Left Top",
                "Left Bottom",
                "Right Top",
                "Right Bottom",
                "Top",
                "Bottom",
            ],
        }
    }

    pub fn matches_constraints(&self, constraints: &[EdgeConstraint; 6]) -> bool {
        for (edge, constraint) in self.edges.iter().zip(constraints.iter()) {
            match constraint {
                EdgeConstraint::Any => continue,
                EdgeConstraint::Exact(required) => {
                    if edge != required {
                        return false;
                    }
                }
                EdgeConstraint::Compatible(required) => {
                    if !edge.is_compatible(required) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

pub struct Map {
    pub name: String,
    pub image: Image,
    pub pixels: Vec<u8>, // RGB pixels pre-extracted for fast access
    pub width: i32,
    pub height: i32,
    pub orientation: MapOrientation,
    pub edges: MapEdges,
}

impl Map {
    pub fn from_image(name: String, mut image: Image) -> Result<Self, String> {
        let orientation = match (image.width, image.height) {
            (64, 32) => MapOrientation::Horizontal,
            (32, 64) => MapOrientation::Vertical,
            (w, h) => return Err(format!("Invalid map size: {}x{}", w, h)),
        };

        let edges = extract_edges_from_image(&mut image, orientation);

        // Pre-extract pixels for fast access without needing mutable reference
        let width = image.width;
        let height = image.height;
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let color = image.get_color(x, y);
                pixels.push(color.r);
                pixels.push(color.g);
                pixels.push(color.b);
            }
        }

        Ok(Map {
            name,
            image,
            pixels,
            width,
            height,
            orientation,
            edges,
        })
    }

    /// Get pixel color at position without needing mutable reference
    pub fn get_pixel(&self, x: i32, y: i32) -> (u8, u8, u8) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return (0, 0, 0); // Black for out of bounds
        }
        let idx = ((y * self.width + x) * 3) as usize;
        (self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2])
    }

    /// Create a horizontally mirrored version of this map
    pub fn create_mirrored(&self) -> Self {
        let mut mirrored_pixels = Vec::with_capacity(self.pixels.len());

        // Flip each row horizontally
        for y in 0..self.height {
            for x in (0..self.width).rev() {
                let idx = ((y * self.width + x) * 3) as usize;
                mirrored_pixels.push(self.pixels[idx]);
                mirrored_pixels.push(self.pixels[idx + 1]);
                mirrored_pixels.push(self.pixels[idx + 2]);
            }
        }

        // Mirror the edges based on orientation
        let mirrored_edges = match self.orientation {
            MapOrientation::Horizontal => {
                // Horizontal tile (64x32) edges: [top_left, top_right, bottom_left, bottom_right, left, right]
                // When mirrored: swap left<->right, and swap the half-edges
                MapEdges::new([
                    self.edges.edges[1], // top_right -> top_left
                    self.edges.edges[0], // top_left -> top_right
                    self.edges.edges[3], // bottom_right -> bottom_left
                    self.edges.edges[2], // bottom_left -> bottom_right
                    self.edges.edges[5], // right -> left
                    self.edges.edges[4], // left -> right
                ])
            }
            MapOrientation::Vertical => {
                // Vertical tile (32x64) edges: [left_top, left_bottom, right_top, right_bottom, top, bottom]
                // When mirrored: swap left<->right sides
                MapEdges::new([
                    self.edges.edges[2], // right_top -> left_top
                    self.edges.edges[3], // right_bottom -> left_bottom
                    self.edges.edges[0], // left_top -> right_top
                    self.edges.edges[1], // left_bottom -> right_bottom
                    self.edges.edges[4], // top stays the same
                    self.edges.edges[5], // bottom stays the same
                ])
            }
        };

        Map {
            name: format!("{}_mirrored", self.name),
            image: self.image.clone(),
            pixels: mirrored_pixels,
            width: self.width,
            height: self.height,
            orientation: self.orientation,
            edges: mirrored_edges,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EdgeConstraint {
    Any,                  // No constraint
    Exact(EdgeType),      // Must match exactly
    Compatible(EdgeType), // Must be compatible
}

pub struct MapQuery {
    pub orientation: Option<MapOrientation>,
    pub constraints: [EdgeConstraint; 6],
}

impl MapQuery {
    pub fn new() -> Self {
        MapQuery {
            orientation: None,
            constraints: [EdgeConstraint::Any; 6],
        }
    }

    pub fn with_orientation(mut self, orientation: MapOrientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    pub fn with_constraint(mut self, index: usize, constraint: EdgeConstraint) -> Self {
        if index < 6 {
            self.constraints[index] = constraint;
        }
        self
    }

    pub fn matches(&self, map: &Map) -> bool {
        // Check orientation
        if let Some(required_orientation) = self.orientation {
            if map.orientation != required_orientation {
                return false;
            }
        }

        // Check edge constraints
        map.edges.matches_constraints(&self.constraints)
    }
}

impl Default for MapQuery {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MapStats {
    pub total_maps: usize,
    pub horizontal_maps: usize,
    pub vertical_maps: usize,
    pub unique_horizontal_patterns: usize,
    pub unique_vertical_patterns: usize,
    pub pattern_distribution: HashMap<MapEdges, Vec<String>>,
}

pub struct MapSet {
    maps: Vec<Map>,
    horizontal_index: HashMap<MapEdges, Vec<usize>>,
    vertical_index: HashMap<MapEdges, Vec<usize>>,
}

impl MapSet {
    pub fn new() -> Self {
        MapSet {
            maps: Vec::new(),
            horizontal_index: HashMap::new(),
            vertical_index: HashMap::new(),
        }
    }

    pub fn load_from_directory(path: &Path) -> Result<Self, String> {
        let mut map_set = MapSet::new();

        let entries = fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Skip templates
            if filename.starts_with("template") {
                continue;
            }

            // Check if it's an image file
            let extension = path.extension().and_then(|e| e.to_str());
            if !matches!(extension, Some("png") | Some("jpg") | Some("jpeg")) {
                continue;
            }

            // Load image
            let image = Image::load_image(path.to_str().unwrap())
                .map_err(|_| format!("Failed to load image: {}", filename))?;

            // Create map
            match Map::from_image(filename.clone(), image) {
                Ok(map) => {
                    // Add the original map
                    let mirrored = map.create_mirrored();
                    map_set.add_map(map);
                    // Add the mirrored version
                    map_set.add_map(mirrored);
                }
                Err(e) => eprintln!("Warning: Skipping {}: {}", filename, e),
            }
        }

        Ok(map_set)
    }

    pub fn add_map(&mut self, map: Map) {
        let index = self.maps.len();

        // Add to appropriate index
        let index_map = match map.orientation {
            MapOrientation::Horizontal => &mut self.horizontal_index,
            MapOrientation::Vertical => &mut self.vertical_index,
        };

        index_map
            .entry(map.edges)
            .or_insert_with(Vec::new)
            .push(index);

        self.maps.push(map);
    }

    pub fn len(&self) -> usize {
        self.maps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.maps.is_empty()
    }

    pub fn count_horizontal(&self) -> usize {
        self.maps
            .iter()
            .filter(|m| m.orientation == MapOrientation::Horizontal)
            .count()
    }

    pub fn count_vertical(&self) -> usize {
        self.maps
            .iter()
            .filter(|m| m.orientation == MapOrientation::Vertical)
            .count()
    }

    pub fn count_matching(&self, query: &MapQuery) -> usize {
        self.find_matching(query).len()
    }

    pub fn find_matching(&self, query: &MapQuery) -> Vec<&Map> {
        self.maps.iter().filter(|map| query.matches(map)).collect()
    }

    pub fn get_random<R: Rng>(&self, query: &MapQuery, rng: &mut R) -> Option<&Map> {
        let matches = self.find_matching(query);
        if matches.is_empty() {
            None
        } else {
            let index = rng.gen_range(0..matches.len());
            Some(matches[index])
        }
    }

    pub fn get_by_edges(&self, orientation: MapOrientation, edges: MapEdges) -> Vec<&Map> {
        let index_map = match orientation {
            MapOrientation::Horizontal => &self.horizontal_index,
            MapOrientation::Vertical => &self.vertical_index,
        };

        if let Some(indices) = index_map.get(&edges) {
            indices.iter().map(|&i| &self.maps[i]).collect()
        } else {
            Vec::new()
        }
    }

    pub fn unique_edge_patterns(
        &self,
        orientation: Option<MapOrientation>,
    ) -> HashMap<MapEdges, usize> {
        let mut patterns: HashMap<MapEdges, usize> = HashMap::new();

        for map in &self.maps {
            if let Some(required_orientation) = orientation {
                if map.orientation != required_orientation {
                    continue;
                }
            }

            *patterns.entry(map.edges).or_insert(0) += 1;
        }

        patterns
    }

    pub fn edge_pattern_stats(&self) -> MapStats {
        let horizontal_patterns = self.unique_edge_patterns(Some(MapOrientation::Horizontal));
        let vertical_patterns = self.unique_edge_patterns(Some(MapOrientation::Vertical));

        let mut pattern_distribution: HashMap<MapEdges, Vec<String>> = HashMap::new();

        for map in &self.maps {
            pattern_distribution
                .entry(map.edges)
                .or_insert_with(Vec::new)
                .push(map.name.clone());
        }

        MapStats {
            total_maps: self.len(),
            horizontal_maps: self.count_horizontal(),
            vertical_maps: self.count_vertical(),
            unique_horizontal_patterns: horizontal_patterns.len(),
            unique_vertical_patterns: vertical_patterns.len(),
            pattern_distribution,
        }
    }
}

impl Default for MapSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Read 32 pixels along an edge and convert to a 32-bit number
/// Each pixel contributes 1 bit:
///   - 1 = solid (black, grey pixels)
///   - 0 = air (white, blue pixels)
fn edge_to_number(image: &mut Image, pixels: &[(i32, i32)]) -> u32 {
    let mut result = 0u32;

    for (i, &(x, y)) in pixels.iter().enumerate() {
        let color = image.get_color(x, y);

        // Check if this is an air pixel:
        // - White: RGB(255, 255, 255)
        // - Blue/Tide: RGB(0, 149, 199)
        let is_white = color.r == 255 && color.g == 255 && color.b == 255;
        let is_blue = color.r == 0 && color.g == 149 && color.b == 199;
        let is_air = is_white || is_blue;

        // If not air, it's solid (black/grey) - set the bit
        if !is_air {
            result |= 1 << i;
        }
    }

    result
}

/// Extract 6 edges from maps, sampling only the middle 30 pixels of each edge
/// Returns: [edge1, edge2, edge3, edge4, edge5, edge6]
fn extract_edge_line(image: &mut Image, start: (i32, i32), end: (i32, i32)) -> u32 {
    let pixels: Vec<(i32, i32)> = if start.0 == end.0 {
        // Vertical line
        (start.1.min(end.1)..=start.1.max(end.1))
            .map(|y| (start.0, y))
            .collect()
    } else {
        // Horizontal line
        (start.0.min(end.0)..=start.0.max(end.0))
            .map(|x| (x, start.1))
            .collect()
    };
    edge_to_number(image, &pixels)
}

fn extract_horizontal_edges(image: &mut Image) -> [u32; 6] {
    // Horizontal (64x32): only sample middle 30 pixels of each edge
    // Top/Bottom edges: skip first and last pixel (columns 1-32 and 33-62 become 1-30 and 34-63)
    // Left/Right edges: skip first and last row (rows 1-30)
    [
        extract_edge_line(image, (1, 0), (30, 0)), // Top left (middle 30)
        extract_edge_line(image, (34, 0), (63, 0)), // Top right (middle 30)
        extract_edge_line(image, (1, 31), (30, 31)), // Bottom left (middle 30)
        extract_edge_line(image, (34, 31), (63, 31)), // Bottom right (middle 30)
        extract_edge_line(image, (0, 1), (0, 30)), // Left (middle 30)
        extract_edge_line(image, (63, 1), (63, 30)), // Right (middle 30)
    ]
}

fn extract_vertical_edges(image: &mut Image) -> [u32; 6] {
    // Vertical (32x64): only sample middle 30 pixels of each edge
    // Left/Right edges: skip first and last row (rows 1-30 and 34-63)
    // Top/Bottom edges: skip first and last column (columns 1-30)
    [
        extract_edge_line(image, (0, 1), (0, 30)), // Left top (middle 30)
        extract_edge_line(image, (0, 34), (0, 63)), // Left bottom (middle 30)
        extract_edge_line(image, (31, 1), (31, 30)), // Right top (middle 30)
        extract_edge_line(image, (31, 34), (31, 63)), // Right bottom (middle 30)
        extract_edge_line(image, (1, 0), (30, 0)), // Top (middle 30)
        extract_edge_line(image, (1, 63), (30, 63)), // Bottom (middle 30)
    ]
}

fn extract_edges_from_image(image: &mut Image, orientation: MapOrientation) -> MapEdges {
    let edges = match orientation {
        MapOrientation::Horizontal => extract_horizontal_edges(image),
        MapOrientation::Vertical => extract_vertical_edges(image),
    };

    let edge_types = edges.map(EdgeType::from_u32);
    MapEdges::new(edge_types)
}
