pub mod input;
pub mod settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Input,
    Settings,
}
