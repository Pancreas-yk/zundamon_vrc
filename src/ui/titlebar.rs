use crate::ui::theme::Theme;
use egui::{Align, CornerRadius, Layout, Sense};

pub fn show(ctx: &egui::Context, theme: &Theme) {
    let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    let titlebar_height = 32.0;

    egui::TopBottomPanel::top("titlebar")
        .exact_height(titlebar_height)
        .frame(egui::Frame::NONE.fill(theme.color(theme.titlebar_background)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let drag_rect = ui.available_rect_before_wrap();
                let drag_response = ui.interact(
                    drag_rect,
                    ui.id().with("titlebar_drag"),
                    Sense::click_and_drag(),
                );
                if drag_response.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if drag_response.double_clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                }

                let title_rect = ui.available_rect_before_wrap();
                ui.painter().text(
                    title_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "ZUNDAMON VRC",
                    egui::FontId::proportional(11.0),
                    theme.color(theme.titlebar_text),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let close_btn = ui.add(
                        egui::Button::new(egui::RichText::new("\u{2715}").size(12.0)).frame(false),
                    );
                    if close_btn.clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if close_btn.hovered() {
                        ui.painter().rect_filled(
                            close_btn.rect,
                            CornerRadius::same(4),
                            theme.color(theme.status_error),
                        );
                    }

                    let max_icon = if is_maximized { "\u{25A3}" } else { "\u{25A1}" };
                    let max_btn = ui.add(
                        egui::Button::new(egui::RichText::new(max_icon).size(12.0)).frame(false),
                    );
                    if max_btn.clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                    }

                    let min_btn = ui.add(
                        egui::Button::new(egui::RichText::new("\u{2212}").size(12.0)).frame(false),
                    );
                    if min_btn.clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });
        });
}
