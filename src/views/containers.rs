use std::collections::HashMap;

use gpui::*;
use gpui::prelude::*;

use crate::components::ContainerStatusBadge;
use crate::models::{ContainerViewModel, dummy_containers};
use crate::theme::{colors, Theme};

/// Containers list view
pub struct ContainersView {
    containers: Vec<ContainerViewModel>,
    selected_id: Option<String>,
    show_stopped: bool,
}

impl ContainersView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            containers: dummy_containers(),
            selected_id: None,
            show_stopped: true,
        }
    }

    fn toggle_show_stopped(&mut self, cx: &mut Context<Self>) {
        self.show_stopped = !self.show_stopped;
        cx.notify();
    }

    fn select_container(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }

    fn start_container(&mut self, _id: &str, cx: &mut Context<Self>) {
        tracing::info!("Start container: {}", _id);
        cx.notify();
    }

    fn stop_container(&mut self, _id: &str, cx: &mut Context<Self>) {
        tracing::info!("Stop container: {}", _id);
        cx.notify();
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        tracing::info!("Refresh containers");
        cx.notify();
    }
}

impl Render for ContainersView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Filter containers
        let filtered: Vec<_> = self
            .containers
            .iter()
            .filter(|c| self.show_stopped || c.is_running())
            .collect();

        // Group by compose project
        let mut compose_groups: HashMap<String, Vec<&ContainerViewModel>> = HashMap::new();
        let mut standalone: Vec<&ContainerViewModel> = Vec::new();

        for container in &filtered {
            if let Some(ref project) = container.compose_project {
                compose_groups.entry(project.clone()).or_default().push(container);
            } else {
                standalone.push(container);
            }
        }

        let running_count = self.containers.iter().filter(|c| c.is_running()).count();
        let total_count = self.containers.len();

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
                            .child(Theme::page_title("Containers"))
                            .child(
                                Theme::badge().child(format!("{} / {} running", running_count, total_count)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            // Show stopped toggle
                            .child(
                                div()
                                    .id("toggle-show-stopped")
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _window, cx| {
                                        this.toggle_show_stopped(cx);
                                    }))
                                    .child(
                                        div()
                                            .w(px(16.0))
                                            .h(px(16.0))
                                            .rounded(px(4.0))
                                            .border_1()
                                            .border_color(colors::border_focused())
                                            .when(self.show_stopped, |el| {
                                                el.bg(colors::accent()).child(
                                                    div()
                                                        .w_full()
                                                        .h_full()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .text_color(colors::on_accent())
                                                        .text_xs()
                                                        .child("✓"),
                                                )
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(colors::text_muted())
                                            .child("Show stopped"),
                                    ),
                            )
                            // Refresh button
                            .child(
                                Theme::button_icon()
                                    .id("refresh-containers")
                                    .on_click(cx.listener(|this, _, _window, cx| {
                                        this.refresh(cx);
                                    }))
                                    .child("↻"),
                            ),
                    ),
            )
            // Container list
            .child(
                div()
                    .id("containers-list")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            // Compose project groups
                            .children(compose_groups.into_iter().map(|(project, containers)| {
                                self.render_container_group(project, containers, cx)
                            }))
                            // Standalone containers
                            .when(!standalone.is_empty(), |el| {
                                el.child(self.render_container_group("Standalone", standalone, cx))
                            }),
                    ),
            )
    }
}

impl ContainersView {
    fn render_container_group(
        &self,
        title: impl Into<SharedString>,
        containers: Vec<&ContainerViewModel>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let title: SharedString = title.into();
        let count = containers.len();

        Theme::card()
            .flex()
            .flex_col()
            // Group header
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(colors::border())
                    .child(div().text_color(colors::text_muted()).child("📁"))
                    .child(div().text_color(colors::text()).child(title))
                    .child(Theme::badge().child(count.to_string())),
            )
            // Container rows
            .children(containers.into_iter().map(|c| self.render_container_row(c, cx)))
    }

    fn render_container_row(
        &self,
        container: &ContainerViewModel,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let id = container.id.clone();
        let id_for_select = container.id.clone();
        let id_for_action = container.id.clone();
        let is_selected = self.selected_id.as_ref() == Some(&id);
        let is_running = container.is_running();

        div()
            .id(SharedString::from(format!("container-{}", &id)))
            .flex()
            .items_center()
            .gap_3()
            .px_4()
            .py_3()
            .cursor_pointer()
            .when(is_selected, |el| el.bg(colors::selection()))
            .hover(|el| el.bg(colors::hover()))
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.select_container(id_for_select.clone(), cx);
            }))
            // Status badge
            .child(ContainerStatusBadge::new(container.state))
            // Name and image
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(div().text_color(colors::text()).child(container.name.clone()))
                    .child(Theme::subtitle(container.image.clone())),
            )
            // Ports
            .child(
                div()
                    .w(px(120.0))
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(container.ports_display()),
            )
            // Created time
            .child(
                div()
                    .w(px(100.0))
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(container.created_ago()),
            )
            // Action buttons
            .child(
                div()
                    .flex()
                    .gap_1()
                    .child(if is_running {
                        Theme::button_icon()
                            .id(SharedString::from(format!("stop-{}", &id)))
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.stop_container(&id_for_action, cx);
                            }))
                            .child("⏹")
                    } else {
                        Theme::button_icon()
                            .id(SharedString::from(format!("start-{}", &id)))
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.start_container(&id_for_action, cx);
                            }))
                            .child("▶")
                    })
                    .child(Theme::button_icon().child("⋮")),
            )
    }
}
