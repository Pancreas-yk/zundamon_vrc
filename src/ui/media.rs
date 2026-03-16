use crate::app::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("メディア");
    ui.separator();

    if !state.media_has_ytdlp || !state.media_has_ffmpeg {
        ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "必要なツール:");
        if !state.media_has_ytdlp {
            ui.label("  yt-dlp が見つかりません。インストールしてください。");
        }
        if !state.media_has_ffmpeg {
            ui.label("  ffmpeg が見つかりません。インストールしてください。");
        }
        ui.add_space(8.0);
    }

    ui.label("URL再生");
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.media_url)
                .hint_text("URLを入力...")
                .desired_width(300.0),
        );
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let can_play = !state.media_url.trim().is_empty()
            && !state.media_playing
            && state.media_has_ytdlp
            && state.media_has_ffmpeg;
        if ui.add_enabled(can_play, egui::Button::new("再生")).clicked() {
            state.pending_media_play = Some(state.media_url.trim().to_string());
        }
        if ui.add_enabled(state.media_playing, egui::Button::new("停止")).clicked() {
            state.pending_media_stop = true;
        }
    });

    if state.media_playing {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("メディア再生中...");
        });
    }

    ui.add_space(16.0);
    ui.separator();

    ui.label("デスクトップ音声キャプチャ");
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        if ui.button("アプリ一覧を更新").clicked() {
            state.pending_refresh_sink_inputs = true;
        }
        if state.is_capturing {
            if ui.button("キャプチャ停止").clicked() {
                state.pending_stop_capture = true;
            }
            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "キャプチャ中");
        }
    });

    ui.add_space(4.0);

    if state.sink_inputs.is_empty() {
        ui.label("「アプリ一覧を更新」を押してください");
    } else {
        for input in state.sink_inputs.clone() {
            ui.horizontal(|ui| {
                ui.label(format!("{} (ID: {})", input.name, input.id));
                let can_capture = !state.is_capturing && state.device_ready;
                if ui.add_enabled(can_capture, egui::Button::new("キャプチャ")).clicked() {
                    state.pending_start_capture = Some((input.id, input.sink.clone()));
                }
            });
        }
    }
}
