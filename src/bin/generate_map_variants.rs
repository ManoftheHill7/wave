use image::RgbaImage;
use std::env;

fn main() {
    // Check for --vertical flag
    let args: Vec<String> = env::args().collect();
    let is_vertical = args.iter().any(|arg| arg == "--vertical");

    let orientation = if is_vertical { "v" } else { "h" };

    // Load the two template images
    let cavern_path = format!("assets/maps/template_cavern_{}.png", orientation);
    let tunnel_path = format!("assets/maps/template_tunnel_{}.png", orientation);

    let cavern = image::open(&cavern_path)
        .expect(&format!("Failed to open {}", cavern_path))
        .to_rgba8();
    let tunnel = image::open(&tunnel_path)
        .expect(&format!("Failed to open {}", tunnel_path))
        .to_rgba8();

    let expected_dims = if is_vertical { (32, 64) } else { (64, 32) };
    assert_eq!(cavern.dimensions(), expected_dims);
    assert_eq!(tunnel.dimensions(), expected_dims);

    // Generate all 64 combinations (2^6)
    for combo in 0..64 {
        let img = generate_image(&cavern, &tunnel, combo, is_vertical);

        // Generate filename: gen_X_XXXXXX.png where X is h/v and each X is T (tunnel) or C (cavern)
        let edge_chars: Vec<char> = (0..6)
            .map(|i| if combo & (1 << i) != 0 { 'T' } else { 'C' })
            .collect();
        let filename = format!(
            "assets/maps/gen_{}_{}{}{}{}{}{}.png",
            orientation,
            edge_chars[0], edge_chars[1], edge_chars[2],
            edge_chars[3], edge_chars[4], edge_chars[5]
        );

        img.save(&filename).expect(&format!("Failed to save {}", filename));
        println!("Generated {}", filename);
    }
}

fn generate_image(cavern: &RgbaImage, tunnel: &RgbaImage, combo: u8, is_vertical: bool) -> RgbaImage {
    let (width, height) = if is_vertical { (32, 64) } else { (64, 32) };
    let mut img = RgbaImage::new(width, height);

    // Decode which template to use for each edge (bit 0-5 of combo)
    // 0 = cavern, 1 = tunnel
    let edges = [
        combo & 1 != 0,         // bit 0: left edge
        combo & 2 != 0,         // bit 1: right edge
        combo & 4 != 0,         // bit 2: top-left edge
        combo & 8 != 0,         // bit 3: top-right edge
        combo & 16 != 0,        // bit 4: bottom-left edge
        combo & 32 != 0,        // bit 5: bottom-right edge
    ];

    // For each pixel, determine which template to use
    for y in 0..height {
        for x in 0..width {
            let template = choose_template_for_pixel(x, y, &edges, is_vertical);
            let source = if template { tunnel } else { cavern };
            img.put_pixel(x, y, *source.get_pixel(x, y));
        }
    }

    img
}

fn choose_template_for_pixel(x: u32, y: u32, edges: &[bool; 6], is_vertical: bool) -> bool {
    // Edge definitions (middle 30 pixels):
    // Horizontal: 64x32
    //   Left edge: column 0, rows 1-30
    //   Right edge: column 63, rows 1-30
    //   Top-left edge: row 0, columns 1-32
    //   Top-right edge: row 0, columns 33-62
    //   Bottom-left edge: row 31, columns 1-32
    //   Bottom-right edge: row 31, columns 33-62
    // Vertical: 32x64
    //   Left edge: column 0, rows 1-32
    //   Right edge: column 31, rows 1-32
    //   Top-left edge: row 0, columns 1-30
    //   Top-right edge: row 0, columns 1-30 (same as top-left for vertical)
    //   Bottom-left edge: row 63, columns 1-30
    //   Bottom-right edge: row 63, columns 1-30 (same as bottom-left for vertical)

    let mut min_distance = f32::MAX;
    let mut closest_edge = 0;

    if is_vertical {
        // Vertical orientation (32x64)
        // Left edge: column 0, rows 1-32
        if y >= 1 && y <= 32 {
            let dist = x as f32;
            if dist < min_distance {
                min_distance = dist;
                closest_edge = 0;
            }
        }

        // Right edge: column 31, rows 1-32
        if y >= 1 && y <= 32 {
            let dist = (31 - x) as f32;
            if dist < min_distance {
                min_distance = dist;
                closest_edge = 1;
            }
        }

        // Top-left edge: row 0, columns 1-30
        if x >= 1 && x <= 30 {
            let dist = y as f32;
            if dist < min_distance {
                min_distance = dist;
                closest_edge = 2;
            }
        }

        // Top-right edge: row 0, columns 1-30 (same range for vertical)
        // This will have same distance as top-left, so we keep it consistent

        // Bottom-left edge: row 63, columns 1-30
        if x >= 1 && x <= 30 {
            let dist = (63 - y) as f32;
            if dist < min_distance {
                min_distance = dist;
                closest_edge = 4;
            }
        }

        // Bottom-right edge: row 63, columns 1-30 (same range for vertical)
        // This will have same distance as bottom-left
    } else {
        // Horizontal orientation (64x32)
        // Distance to left edge (column 0, rows 1-30)
        if y >= 1 && y <= 30 {
            let dist = x as f32;
            if dist < min_distance {
                min_distance = dist;
                closest_edge = 0;
            }
        }

        // Distance to right edge (column 63, rows 1-30)
        if y >= 1 && y <= 30 {
            let dist = (63 - x) as f32;
            if dist < min_distance {
                min_distance = dist;
                closest_edge = 1;
            }
        }

        // Distance to top-left edge (row 0, columns 1-32)
        if x >= 1 && x <= 32 {
            let dist = y as f32;
            if dist < min_distance {
                min_distance = dist;
                closest_edge = 2;
            }
        }

        // Distance to top-right edge (row 0, columns 33-62)
        if x >= 33 && x <= 62 {
            let dist = y as f32;
            if dist < min_distance {
                min_distance = dist;
                closest_edge = 3;
            }
        }

        // Distance to bottom-left edge (row 31, columns 1-32)
        if x >= 1 && x <= 32 {
            let dist = (31 - y) as f32;
            if dist < min_distance {
                min_distance = dist;
                closest_edge = 4;
            }
        }

        // Distance to bottom-right edge (row 31, columns 33-62)
        if x >= 33 && x <= 62 {
            let dist = (31 - y) as f32;
            if dist < min_distance {
                min_distance = dist;
                closest_edge = 5;
            }
        }
    }

    edges[closest_edge]
}
