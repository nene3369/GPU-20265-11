pub mod input;
pub mod window;

pub use input::{InputState, KeyCode, MouseButton};
pub use window::{WindowDescriptor, WindowState};

use ce_app::{App, Plugin};

pub struct WindowPlugin {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub resizable: bool,
}

impl Default for WindowPlugin {
    fn default() -> Self {
        Self {
            title: "ChemEngine".to_string(),
            width: 1280,
            height: 720,
            vsync: true,
            resizable: true,
        }
    }
}

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowDescriptor {
            title: self.title.clone(),
            width: self.width,
            height: self.height,
            vsync: self.vsync,
            resizable: self.resizable,
        });
        app.insert_resource(InputState::default());
        log::info!(
            "WindowPlugin loaded: {}x{} '{}'",
            self.width,
            self.height,
            self.title
        );
    }
}
