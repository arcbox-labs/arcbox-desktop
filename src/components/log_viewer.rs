use gpui::*;

use crate::theme::colors;

/// Log viewer component (placeholder skeleton)
pub struct LogViewer {
    #[allow(dead_code)]
    container_id: String,
}

impl LogViewer {
    pub fn new(container_id: String, _cx: &mut Context<Self>) -> Self {
        Self { container_id }
    }
}

impl Render for LogViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x1a1a1a))
            .child(
                div()
                    .text_color(colors::text_muted())
                    .child("Log Viewer (Coming Soon)"),
            )
    }
}
