use serde::Deserialize;
use serde::Serialize;

/// Control scheme for preview viewport.
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum PreviewControlScheme {
    Simple,
    Maya,
    Blender,
}

/// The current key state of the mouse.
pub struct ViewportKeyState {
    pub control_scheme: PreviewControlScheme,
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub alt: bool,
    pub shift: bool,
}
