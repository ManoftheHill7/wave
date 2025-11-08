//! A reusable screen management system for games
//!
//! This crate provides a flexible way to manage game screens (menus, gameplay, inventories, etc.)
//! with support for screen stacking, transitions, and data passing between screens.
//!
//! # Example
//!
//! ```rust
//! use screen_manager::{Screen, ScreenManager, ScreenCommand};
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
//!     fn render(&mut self, _ctx: &Self::Context) {
//!         // Render menu
//!     }
//! }
//!
//! # fn main() {
//! let mut ctx = MyContext { };
//! let mut manager = ScreenManager::new(Box::new(MenuScreen), &mut ctx);
//! manager.update(0.016, &mut ctx);
//! manager.render(&ctx);
//! # }
//! ```

use std::fmt::Debug;

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
    fn render(&mut self, ctx: &Self::Context);
}

/// Manages a stack of screens and handles transitions between them
///
/// The `ScreenManager` is generic over a context type that will be passed to all screens.
pub struct ScreenManager<Ctx> {
    stack: Vec<Box<dyn Screen<Context = Ctx>>>,
}

impl<Ctx> ScreenManager<Ctx> {
    /// Create a new screen manager with an initial screen
    ///
    /// # Example
    /// ```ignore
    /// let manager = ScreenManager::new(Box::new(MainMenuScreen::new()), &mut ctx);
    /// ```
    pub fn new(initial_screen: Box<dyn Screen<Context = Ctx>>, ctx: &mut Ctx) -> Self {
        let mut manager = Self {
            stack: Vec::new(),
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
    /// This calls the `render` method on the active screen
    pub fn render(&mut self, ctx: &Ctx) {
        if let Some(screen) = self.stack.last_mut() {
            screen.render(ctx);
        }
    }

    /// Render all screens in the stack (useful for transparent overlays)
    ///
    /// Renders from bottom to top, allowing you to show dimmed backgrounds
    pub fn render_all(&mut self, ctx: &Ctx) {
        for screen in self.stack.iter_mut() {
            screen.render(ctx);
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
