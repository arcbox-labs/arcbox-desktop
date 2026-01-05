use gpui::*;

use crate::models::MachineViewModel;
use crate::theme::colors;

/// Machine detail view (placeholder)
pub struct MachineDetailView {
    #[allow(dead_code)]
    machine: MachineViewModel,
}

impl MachineDetailView {
    pub fn new(machine: MachineViewModel, _cx: &mut Context<Self>) -> Self {
        Self { machine }
    }
}

impl Render for MachineDetailView {
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
                    .child("Machine Detail (Coming Soon)"),
            )
    }
}
