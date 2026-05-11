/// Configuration for window creation. Inserted as Resource before window is created.
#[derive(Debug, Clone)]
pub struct WindowDescriptor {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub resizable: bool,
}

/// Runtime window state (updated each frame).
#[derive(Debug, Clone, Default)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub focused: bool,
    pub minimized: bool,
    pub scale_factor: f64,
    pub should_close: bool,
}
