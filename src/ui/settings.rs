use crate::app::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("設定");
        ui.separator();

        // VOICEVOX connection
        ui.collapsing("VOICEVOX接続", |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.text_edit_singleline(&mut state.config.voicevox_url);
            });
            ui.horizontal(|ui| {
                ui.label("実行パス:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.config.voicevox_path)
                        .hint_text("/path/to/VOICEVOX または docker run ..."),
                );
                if ui.button("参照").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        state.config.voicevox_path = path.to_string_lossy().to_string();
                    }
                }
            });
            ui.add_space(4.0);
            ui.checkbox(
                &mut state.config.auto_launch_voicevox,
                "アプリ起動時にVOICEVOXを自動起動",
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("接続テスト").clicked() {
                    state.pending_health_check = true;
                }
                if ui.button("VOICEVOX起動").clicked() {
                    state.pending_launch_voicevox = true;
                }
                if state.voicevox_connected {
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "接続OK");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(200, 100, 100), "未接続");
                }
            });
        });

        ui.add_space(8.0);

        // Voice parameters
        ui.collapsing("音声パラメータ", |ui| {
            let params = &mut state.config.synth_params;
            ui.horizontal(|ui| {
                ui.label("速度:");
                ui.add(egui::Slider::new(&mut params.speed_scale, 0.5..=2.0).step_by(0.05));
            });
            ui.horizontal(|ui| {
                ui.label("ピッチ:");
                ui.add(egui::Slider::new(&mut params.pitch_scale, -0.15..=0.15).step_by(0.01));
            });
            ui.horizontal(|ui| {
                ui.label("抑揚:");
                ui.add(
                    egui::Slider::new(&mut params.intonation_scale, 0.0..=2.0).step_by(0.05),
                );
            });
            ui.horizontal(|ui| {
                ui.label("音量:");
                ui.add(egui::Slider::new(&mut params.volume_scale, 0.0..=2.0).step_by(0.05));
            });
            if ui.button("デフォルトに戻す").clicked() {
                *params = crate::config::SynthParamsConfig::default();
            }
        });

        ui.add_space(8.0);

        // Speaker selection
        ui.collapsing("スピーカー選択", |ui| {
            for speaker in &state.speakers {
                for style in &speaker.styles {
                    let label = format!("{} ({})", speaker.name, style.name);
                    ui.radio_value(&mut state.config.speaker_id, style.id, &label);
                }
            }
            if state.speakers.is_empty() {
                ui.label("VOICEVOXに接続してスピーカー一覧を取得してください");
            }
        });

        ui.add_space(8.0);

        // Audio monitoring
        ui.collapsing("オーディオ", |ui| {
            ui.checkbox(
                &mut state.config.monitor_audio,
                "自分にも音声を再生（モニター）",
            );
            ui.label("有効にすると、仮想マイクに加えてスピーカーからも音声が聞こえます");
        });

        ui.add_space(8.0);

        // Virtual device
        ui.collapsing("仮想デバイス", |ui| {
            ui.horizontal(|ui| {
                ui.label("デバイス名:");
                ui.text_edit_singleline(&mut state.config.virtual_device_name);
            });
            ui.horizontal(|ui| {
                if ui.button("作成").clicked() {
                    state.pending_create_device = true;
                }
                if ui.button("削除").clicked() {
                    state.pending_destroy_device = true;
                }
            });
            if state.device_ready {
                ui.colored_label(
                    egui::Color32::from_rgb(100, 200, 100),
                    format!(
                        "マイクソース: {}.monitor",
                        state.config.virtual_device_name
                    ),
                );
            }
        });

        ui.add_space(8.0);

        // Templates
        ui.collapsing("テンプレート", |ui| {
            let mut to_remove = None;
            for (i, template) in state.config.templates.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(template);
                    if ui.button("削除").clicked() {
                        to_remove = Some(i);
                    }
                });
            }
            if let Some(idx) = to_remove {
                state.config.templates.remove(idx);
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let te = egui::TextEdit::singleline(&mut state.new_template_text)
                    .hint_text("新しいテンプレート...")
                    .desired_width(200.0);
                ui.add(te);
                if ui.button("追加").clicked() && !state.new_template_text.trim().is_empty() {
                    state
                        .config
                        .templates
                        .push(state.new_template_text.trim().to_string());
                    state.new_template_text.clear();
                }
            });
        });

        ui.add_space(12.0);

        if ui.button("設定を保存").clicked() {
            match state.config.save() {
                Ok(()) => state.last_error = None,
                Err(e) => state.last_error = Some(format!("保存失敗: {}", e)),
            }
        }
    });
}
