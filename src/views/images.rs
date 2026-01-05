use gpui::*;

use crate::models::{ImageViewModel, calculate_image_stats, dummy_images};
use crate::theme::{colors, Theme};

/// Images list view
pub struct ImagesView {
    images: Vec<ImageViewModel>,
}

impl ImagesView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            images: dummy_images(),
        }
    }

    fn pull_image(&mut self, cx: &mut Context<Self>) {
        tracing::info!("Pull image");
        cx.notify();
    }
}

impl Render for ImagesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (total_size, _unused_size, total_count, _unused_count) = calculate_image_stats(&self.images);

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
                            .child(Theme::page_title("Images"))
                            .child(
                                Theme::badge().child(format!(
                                    "{} images, {}",
                                    total_count,
                                    format_size(total_size)
                                )),
                            ),
                    )
                    .child(
                        Theme::button_primary()
                            .id("pull-image")
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.pull_image(cx);
                            }))
                            .child("Pull Image"),
                    ),
            )
            // Image table
            .child(
                div()
                    .id("images-list")
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
                                    .child(div().flex_1().child("REPOSITORY"))
                                    .child(div().w(px(100.0)).child("TAG"))
                                    .child(div().w(px(100.0)).child("SIZE"))
                                    .child(div().w(px(120.0)).child("CREATED")),
                            )
                            // Table rows
                            .children(self.images.iter().map(|image| {
                                self.render_image_row(image)
                            })),
                    ),
            )
    }
}

impl ImagesView {
    fn render_image_row(&self, image: &ImageViewModel) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(colors::border())
            .hover(|el| el.bg(colors::hover()))
            // Repository
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(colors::text())
                    .child(image.repository.clone()),
            )
            // Tag
            .child(
                div()
                    .w(px(100.0))
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(image.tag.clone()),
            )
            // Size
            .child(
                div()
                    .w(px(100.0))
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(image.size_display()),
            )
            // Created
            .child(
                div()
                    .w(px(120.0))
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(image.created_ago()),
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
