use gpui::*;

use crate::models::{ContainerState, MachineState};
use crate::theme::colors;

/// Status badge for container state
#[derive(IntoElement)]
pub struct ContainerStatusBadge {
    state: ContainerState,
}

impl ContainerStatusBadge {
    pub fn new(state: ContainerState) -> Self {
        Self { state }
    }
}

impl RenderOnce for ContainerStatusBadge {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (color, label) = match self.state {
            ContainerState::Running => (colors::running(), "Running"),
            ContainerState::Stopped => (colors::stopped(), "Stopped"),
            ContainerState::Restarting => (colors::warning(), "Restarting"),
            ContainerState::Paused => (colors::warning(), "Paused"),
            ContainerState::Dead => (colors::error(), "Dead"),
        };

        div()
            .flex()
            .items_center()
            .gap_1p5()
            .child(
                // Status dot
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .rounded_full()
                    .bg(color),
            )
            .child(
                // Status label
                div().text_sm().text_color(color).child(label),
            )
    }
}

/// Status badge for machine state
#[derive(IntoElement)]
pub struct MachineStatusBadge {
    state: MachineState,
}

impl MachineStatusBadge {
    pub fn new(state: MachineState) -> Self {
        Self { state }
    }
}

impl RenderOnce for MachineStatusBadge {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (color, label) = match self.state {
            MachineState::Running => (colors::running(), "Running"),
            MachineState::Stopped => (colors::stopped(), "Stopped"),
            MachineState::Starting => (colors::warning(), "Starting"),
            MachineState::Stopping => (colors::warning(), "Stopping"),
        };

        div()
            .flex()
            .items_center()
            .gap_1p5()
            .child(
                // Status dot
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .rounded_full()
                    .bg(color),
            )
            .child(
                // Status label
                div().text_sm().text_color(color).child(label),
            )
    }
}
