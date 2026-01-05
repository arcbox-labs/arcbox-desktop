use gpui::*;

use crate::models::{VolumeViewModel, dummy_volumes};
use crate::theme::{colors, Theme};

/// Volumes list view
pub struct VolumesView {
    volumes: Vec<VolumeViewModel>,
}

impl VolumesView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            volumes: dummy_volumes(),
        }
    }

    fn create_volume(&mut self, cx: &mut Context<Self>) {
        tracing::info!("Create volume");
        cx.notify();
    }
}

impl Render for VolumesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let total_count = self.volumes.len();
        let total_size: u64 = self.volumes.iter().filter_map(|v| v.size_bytes).sum();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(colors::background())
            // Header
            .child(
                Theme::section_header()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(Theme::page_title("Volumes"))
                            .child(
                                Theme::badge().child(format!(
                                    "{} volumes, {}",
                                    total_count,
                                    format_size(total_size)
                                )),
                            ),
                    )
                    .child(
                        Theme::button_primary()
                            .id("create-volume")
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.create_volume(cx);
                            }))
                            .child("+ Create Volume"),
                    ),
            )
            // Volume table
            .child(
                div()
                    .id("volumes-list")
                    .flex_1()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            // Table header
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .px_4()
                                    .py_2()
                                    .bg(colors::surface())
                                    .text_xs()
                                    .text_color(colors::text_muted())
                                    .child(div().flex_1().child("NAME"))
                                    .child(div().w(px(80.0)).child("DRIVER"))
                                    .child(div().w(px(100.0)).child("SIZE"))
                                    .child(div().w(px(120.0)).child("CREATED")),
                            )
                            // Table rows
                            .children(self.volumes.iter().map(|volume| {
                                self.render_volume_row(volume)
                            })),
                    ),
            )
    }
}

impl VolumesView {
    fn render_volume_row(&self, volume: &VolumeViewModel) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(colors::border())
            .hover(|el| el.bg(colors::hover()))
            // Name
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(colors::text())
                    .child(volume.name.clone()),
            )
            // Driver
            .child(
                div()
                    .w(px(80.0))
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(volume.driver.clone()),
            )
            // Size
            .child(
                div()
                    .w(px(100.0))
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(volume.size_display()),
            )
            // Created
            .child(
                div()
                    .w(px(120.0))
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(volume.created_ago()),
            )
    }
}

fn format_size(bytes: u64) -> String {
    let mb = bytes as f64 / 1_000_000.0;
    if mb >= 1000.0 {
        format!("{:.1} GB", mb / 1000.0)
    } else {
        format!("{:.0} MB", mb)
    }
}
