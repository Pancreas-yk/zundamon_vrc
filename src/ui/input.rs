use crate::app::AppState;
use egui::{Key, Modifiers};

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("ずんだもん VRC");
    ui.separator();

    // Speaker selection
    ui.horizontal(|ui| {
        ui.label("スピーカー:");
        egui::ComboBox::from_id_salt("speaker_select")
            .selected_text(
                state
                    .speakers
                    .iter()
                    .flat_map(|s| {
                        s.styles
                            .iter()
                            .map(move |st| (s.name.clone(), st.name.clone(), st.id))
                    })
                    .find(|(_, _, id)| *id == state.config.speaker_id)
                    .map(|(name, style, _)| format!("{} ({})", name, style))
                    .unwrap_or_else(|| format!("ID: {}", state.config.speaker_id)),
            )
            .show_ui(ui, |ui| {
                for speaker in &state.speakers {
                    for style in &speaker.styles {
                        let label = format!("{} ({})", speaker.name, style.name);
                        if ui
                            .selectable_value(&mut state.config.speaker_id, style.id, &label)
                            .changed()
                        {
                            let _ = state.config.save();
                        }
                    }
                }
            });
    });

    ui.add_space(8.0);

    // Text input
    let text_edit = egui::TextEdit::multiline(&mut state.input_text)
        .hint_text("テキストを入力してEnterで送信 (Shift+Enterで改行)")
        .desired_rows(3)
        .desired_width(f32::INFINITY);

    let response = ui.add(text_edit);

    // Handle Enter (send) vs Shift+Enter (newline)
    let enter_pressed =
        ui.input(|i| i.key_pressed(Key::Enter) && !i.modifiers.contains(Modifiers::SHIFT));
    if enter_pressed && response.has_focus() && !state.input_text.trim().is_empty() {
        if state.input_text.ends_with('\n') {
            state.input_text.pop();
        }
        state.pending_send = Some(state.input_text.trim().to_string());
        state.input_text.clear();
    }

    ui.add_space(4.0);

    // Send button
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                !state.input_text.trim().is_empty() && !state.is_synthesizing,
                egui::Button::new("送信"),
            )
            .clicked()
        {
            state.pending_send = Some(state.input_text.trim().to_string());
            state.input_text.clear();
        }
        if state.is_synthesizing {
            ui.spinner();
            ui.label("合成中...");
        }
    });

    ui.add_space(12.0);

    // Templates
    ui.horizontal(|ui| {
        ui.label("テンプレート:");
    });

    let mut to_remove = None;
    let templates = state.config.templates.clone();
    let cols = 2;
    egui::Grid::new("template_grid")
        .num_columns(cols + 1) // button + delete per col pair, but we layout manually
        .spacing([4.0, 4.0])
        .show(ui, |ui| {
            for (i, template) in templates.iter().enumerate() {
                if !template.is_empty() {
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(!state.is_synthesizing, egui::Button::new(template))
                            .clicked()
                        {
                            state.pending_send = Some(template.clone());
                        }
                        if ui.small_button("x").clicked() {
                            to_remove = Some(i);
                        }
                    });
                    if (i + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            }
        });
    if let Some(idx) = to_remove {
        state.config.templates.remove(idx);
        let _ = state.config.save();
    }

    // Add template inline
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
            let _ = state.config.save();
        }
    });

    ui.add_space(12.0);
    ui.separator();

    // Status bar
    ui.horizontal(|ui| {
        if state.voicevox_connected {
            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "VOICEVOX: 接続済み");
        } else if state.voicevox_launching {
            ui.spinner();
            ui.colored_label(egui::Color32::from_rgb(200, 200, 100), "VOICEVOX: 起動中...");
        } else {
            ui.colored_label(egui::Color32::from_rgb(200, 100, 100), "VOICEVOX: 未接続");
            if ui.small_button("起動").clicked() {
                state.pending_launch_voicevox = true;
            }
        }

        ui.separator();

        let (device_text, device_color) = if state.device_ready {
            (
                "仮想デバイス: 準備完了",
                egui::Color32::from_rgb(100, 200, 100),
            )
        } else {
            (
                "仮想デバイス: 未作成",
                egui::Color32::from_rgb(200, 200, 100),
            )
        };
        ui.colored_label(device_color, device_text);

        if !state.device_ready {
            if ui.small_button("作成").clicked() {
                state.pending_create_device = true;
            }
        }
    });

    if let Some(ref err) = state.last_error {
        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), err);
    }
}
