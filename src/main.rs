use raylib::prelude::*;
use screen_manager::ScreenManager;
use wave_github_gameoff2025::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let map_path = args.get(1).cloned();

    let (mut rl, thread) = raylib::init().size(1600, 900).title("JGame").build();

    rl.set_target_fps(60);

    let mut ctx = GameContext::new(&mut rl, &thread, map_path);
    let mut manager = ScreenManager::new(Box::new(GameScreen::new(&ctx)), &mut ctx);

    use inventory::ItemType;
    ctx.world_state.player.inventory.add(ItemType::Stone, 3);
    ctx.world_state.player.inventory.add(ItemType::Dirt, 10);

    while !rl.window_should_close() && !manager.is_empty() {
        let dt = rl.get_frame_time();

        ctx.handle_global_input(&rl);
        ctx.controller.update(&rl);

        manager.update(dt, &mut ctx);
        manager.render(&mut rl, &thread, &ctx);
    }
}
