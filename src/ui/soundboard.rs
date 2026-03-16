use crate::app::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("サウンドボード");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label(format!("フォルダ: {}", state.config.soundboard_path));
        if ui.button("再スキャン").clicked() {
            state.pending_soundboard_scan = true;
        }
    });

    ui.add_space(8.0);

    if state.soundboard_files.is_empty() {
        ui.label("音声ファイルが見つかりません。");
        ui.label(format!(
            "フォルダにWAV/MP3/OGGファイルを配置してください: {}",
            state.config.soundboard_path
        ));
    } else {
        let cols = 3;
        egui::Grid::new("soundboard_grid")
            .num_columns(cols)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                let files = state.soundboard_files.clone();
                for (i, (name, path)) in files.iter().enumerate() {
                    let enabled = !state.is_playing && !state.is_synthesizing;
                    if ui.add_enabled(enabled, egui::Button::new(name)).clicked() {
                        state.pending_soundboard_play = Some(path.clone());
                    }
                    if (i + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    if state.is_playing {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("再生中...");
        });
    }
}
