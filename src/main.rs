use screen_manager::ScreenManager;
use wave_github_gameoff2025::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check for flags
    let resume_mode = args.iter().any(|arg| arg == "--resume");
    let new_game_mode = args.iter().any(|arg| arg == "--new");
    let creative_mode = args.iter().any(|arg| arg == "--creative");
    let map_path = args
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with("--"))
        .cloned();

    let (mut rl, thread) = raylib::init().size(1600, 900).title("JGame").build();

    unsafe {
        raylib::ffi::SetTraceLogLevel(raylib::ffi::TraceLogLevel::LOG_WARNING as i32);
    }
    rl.set_target_fps(60);

    let mut ctx = GameContext::new(&mut rl, &thread, map_path);

    // Start with menu or game screen based on flags
    let initial_screen: Box<dyn screen_manager::Screen<Context = GameContext>> =
        if new_game_mode || creative_mode {
            // Start a new game directly
            if creative_mode {
                println!("Starting new game in creative mode...");
                ctx.world_state.player.inventory = inventory::Inventory::new(f32::MAX);
                for item_type in inventory::ItemType::all() {
                    ctx.world_state.player.inventory.add(*item_type, 999);
                }
            } else {
                println!("Starting new game...");
            }

            Box::new(GameScreen::new(&ctx))
        } else if resume_mode {
            // Try to load save, if it fails, go to menu
            if save_load::save_exists(0) {
                match save_load::load_game(0) {
                    Ok(save_data) => {
                        save_load::apply_save_data(&mut ctx.world_state, save_data);
                        println!("✓ Game loaded successfully!");
                        Box::new(GameScreen::new(&ctx))
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to load game: {}, starting menu", e);
                        Box::new(MenuScreen::new())
                    }
                }
            } else {
                println!("✗ No save file found, starting menu");
                Box::new(MenuScreen::new())
            }
        } else {
            Box::new(MenuScreen::new())
        };

    use inventory::ItemType;
    // ctx.world_state.player.inventory.add(ItemType::Log, 5);
    // ctx.world_state.player.inventory.add(ItemType::Workbench, 1);
    // ctx.world_state.player.inventory.add(ItemType::Furnace, 1);
    // ctx.world_state.player.inventory.add(ItemType::Anvil, 1);

    // Testing: Add torches
    ctx.world_state.player.inventory.add(ItemType::Torch, 20);
    ctx.world_state
        .player
        .inventory
        .add(ItemType::Lumostorch, 20);

    let mut manager = ScreenManager::new(initial_screen, &mut ctx);

    while !rl.window_should_close() && !manager.is_empty() {
        let dt = rl.get_frame_time();

        ctx.handle_debug_input(&rl);
        ctx.controller.update(&rl);

        manager.update(dt, &mut ctx);
        manager.render(&mut rl, &thread, &ctx);
    }

    match save_load::save_game(&ctx.world_state, 0) {
        Ok(()) => println!("✓ Game saved successfully!"),
        Err(e) => eprintln!("✗ Failed to save game: {}", e),
    }
}
