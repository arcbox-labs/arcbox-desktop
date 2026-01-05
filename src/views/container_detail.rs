use gpui::*;

use crate::models::ContainerViewModel;
use crate::theme::colors;

/// Container detail view (placeholder for now)
pub struct ContainerDetailView {
    #[allow(dead_code)]
    container: ContainerViewModel,
}

impl ContainerDetailView {
    pub fn new(container: ContainerViewModel, _cx: &mut Context<Self>) -> Self {
        Self { container }
    }
}

impl Render for ContainerDetailView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .bg(colors::background())
            .child(
                div()
                    .text_color(colors::text_muted())
                    .child("Container Detail (Coming Soon)"),
            )
    }
}
