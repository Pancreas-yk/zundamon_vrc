pub mod input;
pub mod media;
pub mod settings;
pub mod soundboard;
pub mod theme;
pub mod titlebar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Input,
    Soundboard,
    Media,
    Settings,
}
