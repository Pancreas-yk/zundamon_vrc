use crate::app::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    let _ = state;
    ui.heading("メディア");
    ui.label("準備中...");
}
