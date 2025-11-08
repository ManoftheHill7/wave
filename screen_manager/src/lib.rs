//! A reusable screen management system for games
//!
//! This crate provides a flexible way to manage game screens (menus, gameplay, inventories, etc.)
//! with support for screen stacking, transitions, and data passing between screens.
//!
//! # Example
//!
//! ```rust,no_run
//! use screen_manager::{Screen, ScreenManager, ScreenCommand};
//! use raylib::prelude::*;
//!
//! struct MyContext {
//!     // Context holds shared game state, but not delta_time
//! }
//!
//! struct MenuScreen;
//!
//! impl Screen for MenuScreen {
//!     type Context = MyContext;
//!
//!     // on_resume and on_pause are optional - only implement if needed
//!
//!     fn update(&mut self, dt: f32, _ctx: &mut Self::Context) -> ScreenCommand<Self::Context> {
//!         println!("Delta time: {}", dt);
//!         ScreenCommand::None
//!     }
//!
//!     fn render(&mut self, d: &mut RaylibDrawHandle, _ctx: &Self::Context) {
//!         d.draw_text("Menu", 10, 10, 20, Color::BLACK);
//!     }
//! }
//!
//! fn main() {
//!     let (mut rl, thread) = raylib::init()
//!         .size(800, 600)
//!         .title("Game")
//!         .build();
//!
//!     let mut ctx = MyContext { };
//!     let mut manager = ScreenManager::new(Box::new(MenuScreen), &mut ctx);
//!
//!     while !rl.window_should_close() && !manager.is_empty() {
//!         let dt = rl.get_frame_time();
//!         manager.update(dt, &mut ctx);
//!         manager.render(&mut rl, &thread, &ctx);
//!     }
//! }
//! ```

use std::fmt::Debug;
use raylib::prelude::*;

/// Represents possible transitions between screens
pub enum ScreenCommand<Ctx> {
    /// Continue with the current screen
    None,

    /// Push a new screen onto the stack (previous screen remains underneath)
    Push(Box<dyn Screen<Context = Ctx>>),

    /// Pop the current screen and return to the previous one
    Pop,

    /// Replace the current screen with a new one (no stack growth)
    Replace(Box<dyn Screen<Context = Ctx>>),

    /// Clear all screens and start fresh with a new screen
    Reset(Box<dyn Screen<Context = Ctx>>),
}

/// Trait that all screens must implement
///
/// Screens have lifecycle methods (on_resume, on_pause) and frame methods (update, render).
/// The `Context` associated type allows you to pass game-specific context to screens.
pub trait Screen {
    /// The context type that will be passed to screen methods
    /// This typically contains things like input state, resource managers, etc.
    type Context;

    /// Called when the screen becomes the active screen
    ///
    /// This is called when:
    /// - The screen is first pushed/replaced onto the stack
    /// - A screen above this one is popped, making this screen active again
    ///
    /// Override this method if you need to perform setup when the screen becomes active.
    fn on_resume(&mut self, _ctx: &mut Self::Context) {
        // Default: do nothing
    }

    /// Called when the screen is no longer the active screen
    ///
    /// This is called when:
    /// - A new screen is being pushed on top of this one
    /// - The screen is being replaced
    /// - The screen is being popped from the stack
    ///
    /// Override this method if you need to perform cleanup when the screen becomes inactive.
    fn on_pause(&mut self, _ctx: &mut Self::Context) {
        // Default: do nothing
    }

    /// Called every frame to update the screen's logic
    ///
    /// # Parameters
    /// - `dt`: Delta time in seconds since last frame
    /// - `ctx`: Mutable reference to the game context
    ///
    /// Returns a `ScreenCommand` to indicate if a screen change should occur
    fn update(&mut self, dt: f32, ctx: &mut Self::Context) -> ScreenCommand<Self::Context>;

    /// Called every frame to render the screen
    ///
    /// # Parameters
    /// - `d`: Mutable reference to RaylibDrawHandle for rendering
    /// - `ctx`: Reference to the game context
    fn render(&mut self, d: &mut RaylibDrawHandle, ctx: &Self::Context);
}

/// Manages a stack of screens and handles transitions between them
///
/// The `ScreenManager` is generic over a context type that will be passed to all screens.
pub struct ScreenManager<Ctx> {
    stack: Vec<Box<dyn Screen<Context = Ctx>>>,
    pub clear_color: Color,
}

impl<Ctx> ScreenManager<Ctx> {
    /// Create a new screen manager with an initial screen
    ///
    /// # Example
    /// ```ignore
    /// let manager = ScreenManager::new(
    ///     Box::new(MainMenuScreen::new()),
    ///     &mut ctx
    /// );
    /// ```
    pub fn new(
        initial_screen: Box<dyn Screen<Context = Ctx>>,
        ctx: &mut Ctx
    ) -> Self {
        let mut manager = Self {
            stack: Vec::new(),
            clear_color: Color::RAYWHITE,
        };
        manager.push_screen(initial_screen, ctx);
        manager
    }

    /// Get the current stack depth
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Check if there are no screens (game should exit)
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Update the current screen and handle any transitions
    ///
    /// # Parameters
    /// - `dt`: Delta time in seconds since last frame
    /// - `ctx`: Mutable reference to the game context
    ///
    /// This calls the `update` method on the active screen and processes any resulting transitions
    pub fn update(&mut self, dt: f32, ctx: &mut Ctx) {
        if self.stack.is_empty() {
            return;
        }

        let command = self.stack.last_mut().unwrap().update(dt, ctx);
        self.handle_command(command, ctx);
    }

    /// Render the current screen
    ///
    /// This automatically calls `begin_drawing()`, clears the background with `clear_color`,
    /// and calls the screen's render method
    ///
    /// # Parameters
    /// - `rl`: Mutable reference to RaylibHandle
    /// - `thread`: Reference to RaylibThread
    /// - `ctx`: Reference to the game context
    pub fn render(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, ctx: &Ctx) {
        if let Some(screen) = self.stack.last_mut() {
            let mut d = rl.begin_drawing(thread);
            d.clear_background(self.clear_color);
            screen.render(&mut d, ctx);
        }
    }

    /// Render all screens in the stack (useful for transparent overlays)
    ///
    /// Renders from bottom to top, allowing you to show dimmed backgrounds.
    /// This automatically calls `begin_drawing()`, clears the background with `clear_color`,
    /// and calls each screen's render method
    ///
    /// # Parameters
    /// - `rl`: Mutable reference to RaylibHandle
    /// - `thread`: Reference to RaylibThread
    /// - `ctx`: Reference to the game context
    pub fn render_all(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, ctx: &Ctx) {
        let mut d = rl.begin_drawing(thread);
        d.clear_background(self.clear_color);
        for screen in self.stack.iter_mut() {
            screen.render(&mut d, ctx);
        }
    }

    /// Handle a screen command
    fn handle_command(&mut self, command: ScreenCommand<Ctx>, ctx: &mut Ctx) {
        match command {
            ScreenCommand::None => {},

            ScreenCommand::Push(screen) => {
                if let Some(current) = self.stack.last_mut() {
                    current.on_pause(ctx);
                }
                self.push_screen(screen, ctx);
            }

            ScreenCommand::Pop => {
                self.pop_screen(ctx);
            }

            ScreenCommand::Replace(screen) => {
                self.pop_screen(ctx);
                self.push_screen(screen, ctx);
            }

            ScreenCommand::Reset(screen) => {
                while !self.stack.is_empty() {
                    self.pop_screen(ctx);
                }
                self.push_screen(screen, ctx);
            }
        }
    }

    /// Push a new screen onto the stack
    fn push_screen(&mut self, mut screen: Box<dyn Screen<Context = Ctx>>, ctx: &mut Ctx) {
        screen.on_resume(ctx);
        self.stack.push(screen);
    }

    /// Pop the current screen from the stack
    fn pop_screen(&mut self, ctx: &mut Ctx) {
        if let Some(mut screen) = self.stack.pop() {
            screen.on_pause(ctx);

            // Call on_resume on the new active screen
            if let Some(resumed_screen) = self.stack.last_mut() {
                resumed_screen.on_resume(ctx);
            }
        }
    }

    /// Get the number of screens in the stack
    pub fn len(&self) -> usize {
        self.stack.len()
    }
}

impl<Ctx> Debug for ScreenManager<Ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScreenManager")
            .field("depth", &self.stack.len())
            .finish()
    }
}
