use std::collections::HashMap;

use gpui::*;
use gpui::prelude::*;

use crate::components::ContainerStatusBadge;
use crate::models::{ContainerState, ContainerViewModel, dummy_containers};
use crate::theme::{colors, Theme};

/// Detail panel tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailTab {
    #[default]
    Info,
    Logs,
    Terminal,
    Files,
}

/// Panel resize drag state
#[derive(Clone)]
struct PanelResizeDrag {
    initial_width: f32,
}

/// Visual element shown during drag (invisible)
struct PanelResizeHandleVisual {
    #[allow(dead_code)]
    initial_width: f32,
}

impl Render for PanelResizeHandleVisual {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().w(px(0.0)).h(px(0.0))
    }
}

const LIST_MIN_WIDTH: f32 = 200.0;
const LIST_MAX_WIDTH: f32 = 500.0;
const LIST_DEFAULT_WIDTH: f32 = 340.0;

/// Containers list view
pub struct ContainersView {
    containers: Vec<ContainerViewModel>,
    selected_id: Option<String>,
    expanded_groups: HashMap<String, bool>,
    active_tab: DetailTab,
    list_width: f32,
}

impl ContainersView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let mut expanded_groups = HashMap::new();
        // Expand all groups by default
        for c in dummy_containers() {
            if let Some(ref project) = c.compose_project {
                expanded_groups.insert(project.clone(), true);
            }
        }

        Self {
            containers: dummy_containers(),
            selected_id: None,
            expanded_groups,
            active_tab: DetailTab::Info,
            list_width: LIST_DEFAULT_WIDTH,
        }
    }

    fn resize_list(&mut self, new_width: f32, cx: &mut Context<Self>) {
        self.list_width = new_width.clamp(LIST_MIN_WIDTH, LIST_MAX_WIDTH);
        cx.notify();
    }

    fn toggle_group(&mut self, group: String, cx: &mut Context<Self>) {
        let expanded = self.expanded_groups.entry(group).or_insert(true);
        *expanded = !*expanded;
        cx.notify();
    }

    fn select_container(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }

    fn set_tab(&mut self, tab: DetailTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
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

    fn get_selected_container(&self) -> Option<&ContainerViewModel> {
        self.selected_id
            .as_ref()
            .and_then(|id| self.containers.iter().find(|c| &c.id == id))
    }
}

impl Render for ContainersView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let running_count = self.containers.iter().filter(|c| c.is_running()).count();

        // Group containers
        let mut compose_groups: HashMap<String, Vec<&ContainerViewModel>> = HashMap::new();
        let mut standalone: Vec<&ContainerViewModel> = Vec::new();

        for container in &self.containers {
            if let Some(ref project) = container.compose_project {
                compose_groups
                    .entry(project.clone())
                    .or_default()
                    .push(container);
            } else {
                standalone.push(container);
            }
        }

        let list_width = self.list_width;

        div()
            .flex_1()
            .flex()
            .flex_row()
            .overflow_hidden()
            // Left panel - container list
            .child(
                div()
                    .w(px(list_width))
                    .h_full()
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .h(px(52.0))
                            .px_4()
                            .border_b_1()
                            .border_color(colors::border())
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(colors::text())
                                            .child("Containers"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(colors::text_muted())
                                            .child(format!("{} running", running_count)),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        Theme::button_icon()
                                            .id("filter-containers")
                                            .child("⊕"),
                                    )
                                    .child(
                                        Theme::button_icon()
                                            .id("search-containers")
                                            .child("⌕"),
                                    ),
                            ),
                    )
                    // Container list
                    .child(
                        div()
                            .id("containers-list")
                            .flex_1()
                            .overflow_y_scroll()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    // Compose groups
                                    .children(
                                        compose_groups
                                            .iter()
                                            .map(|(project, containers)| {
                                                self.render_container_group(
                                                    project.clone(),
                                                    containers.clone(),
                                                    cx,
                                                )
                                            }),
                                    )
                                    // Standalone containers
                                    .when(!standalone.is_empty(), |el| {
                                        el.children(
                                            standalone
                                                .iter()
                                                .map(|c| self.render_container_row(c, false, cx)),
                                        )
                                    }),
                            ),
                    ),
            )
            // Resize handle
            .child(self.render_resize_handle(list_width, cx))
            // Right panel - detail
            .child(self.render_detail_panel(cx))
    }
}

impl ContainersView {
    fn render_resize_handle(&self, current_width: f32, cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("container-list-resize-handle")
            .w(px(4.0))
            .h_full()
            .cursor(CursorStyle::ResizeLeftRight)
            .bg(colors::border())
            .hover(|el| el.bg(colors::accent()))
            .on_drag(
                PanelResizeDrag {
                    initial_width: current_width,
                },
                |drag, _point, _window, cx| {
                    cx.new(|_cx| PanelResizeHandleVisual {
                        initial_width: drag.initial_width,
                    })
                },
            )
            .on_drag_move::<PanelResizeDrag>(cx.listener(
                move |this, event: &DragMoveEvent<PanelResizeDrag>, _window, cx| {
                    // Calculate new width based on mouse position
                    // Account for the sidebar width (approximately 180px)
                    let sidebar_offset: f32 = 180.0;
                    let new_width: f32 = f32::from(event.event.position.x) - sidebar_offset;
                    this.resize_list(new_width, cx);
                },
            ))
    }
}

impl ContainersView {
    fn render_container_group(
        &self,
        project: String,
        containers: Vec<&ContainerViewModel>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let is_expanded = self.expanded_groups.get(&project).copied().unwrap_or(true);
        let project_for_click = project.clone();

        div()
            .flex()
            .flex_col()
            // Group header
            .child(
                div()
                    .id(SharedString::from(format!("group-{}", &project)))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .cursor_pointer()
                    .hover(|el| el.bg(colors::hover()))
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        this.toggle_group(project_for_click.clone(), cx);
                    }))
                    .child(
                        div()
                            .w(px(16.0))
                            .text_xs()
                            .text_color(colors::text_muted())
                            .child(if is_expanded { "▼" } else { "▶" }),
                    )
                    .child(
                        div()
                            .w(px(24.0))
                            .h(px(24.0))
                            .rounded_md()
                            .bg(colors::accent())
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(colors::on_accent())
                            .child("⬡"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::text())
                            .child(project),
                    ),
            )
            // Container rows (if expanded)
            .when(is_expanded, |el| {
                el.children(
                    containers
                        .iter()
                        .map(|c| self.render_container_row(c, true, cx)),
                )
            })
    }

    fn render_container_row(
        &self,
        container: &ContainerViewModel,
        indented: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let id = container.id.clone();
        let id_for_select = container.id.clone();
        let id_for_action = container.id.clone();
        let is_selected = self.selected_id.as_ref() == Some(&id);
        let is_running = container.is_running();

        let base = div()
            .id(SharedString::from(format!("container-{}", &id)))
            .flex()
            .items_center()
            .gap_2()
            .py_1p5()
            .pr_2()
            .cursor_pointer()
            .when(indented, |el| el.pl(px(44.0)))
            .when(!indented, |el| el.pl_4())
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.select_container(id_for_select.clone(), cx);
            }));

        let base = if is_selected {
            base.bg(colors::selection()).text_color(colors::on_accent())
        } else {
            base.hover(|el| el.bg(colors::hover()))
        };

        base
            // Container icon with status dot
            .child(
                div()
                    .relative()
                    .w(px(24.0))
                    .h(px(24.0))
                    .rounded_md()
                    .bg(if is_selected {
                        rgba(0xffffff30)
                    } else {
                        colors::surface_elevated()
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .child(self.get_container_icon(&container.image))
                    // Status dot
                    .child(
                        div()
                            .absolute()
                            .left(px(-2.0))
                            .bottom(px(-2.0))
                            .w(px(8.0))
                            .h(px(8.0))
                            .rounded_full()
                            .bg(if is_running {
                                colors::running()
                            } else {
                                colors::stopped()
                            }),
                    ),
            )
            // Name and image
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_sm()
                            .text_ellipsis()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(container.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_ellipsis()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .when(is_selected, |el| el.text_color(rgba(0xffffffaa)))
                            .when(!is_selected, |el| el.text_color(colors::text_muted()))
                            .child(container.image.clone()),
                    ),
            )
            // Action buttons
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_0p5()
                    .child(if is_running {
                        Theme::button_icon()
                            .id(SharedString::from(format!("stop-{}", &id)))
                            .w(px(24.0))
                            .h(px(24.0))
                            .when(is_selected, |el| el.text_color(colors::on_accent()))
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.stop_container(&id_for_action, cx);
                            }))
                            .child("■")
                    } else {
                        Theme::button_icon()
                            .id(SharedString::from(format!("start-{}", &id)))
                            .w(px(24.0))
                            .h(px(24.0))
                            .when(is_selected, |el| el.text_color(colors::on_accent()))
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.start_container(&id_for_action, cx);
                            }))
                            .child("▶")
                    })
                    .child(
                        Theme::button_icon()
                            .w(px(24.0))
                            .h(px(24.0))
                            .when(is_selected, |el| el.text_color(colors::on_accent()))
                            .child("🗑"),
                    ),
            )
    }

    fn get_container_icon(&self, image: &str) -> &'static str {
        if image.contains("postgres") {
            "🐘"
        } else if image.contains("redis") {
            "◆"
        } else if image.contains("nginx") {
            "▲"
        } else if image.contains("node") {
            "⬢"
        } else if image.contains("mongo") {
            "🍃"
        } else {
            "☐"
        }
    }

    fn render_detail_panel(&self, cx: &Context<Self>) -> impl IntoElement {
        let selected = self.get_selected_container();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(colors::background())
            // Tab bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(52.0))
                    .px_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(colors::border())
                    .child(self.render_tab_button(DetailTab::Info, "Info", cx))
                    .child(self.render_tab_button(DetailTab::Logs, "Logs", cx))
                    .child(self.render_tab_button(DetailTab::Terminal, "Terminal", cx))
                    .child(self.render_tab_button(DetailTab::Files, "Files", cx)),
            )
            // Tab content
            .child(
                div()
                    .id("detail-content")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_4()
                    .child(if let Some(container) = selected {
                        self.render_detail_content(container).into_any_element()
                    } else {
                        self.render_no_selection().into_any_element()
                    }),
            )
    }

    fn render_tab_button(
        &self,
        tab: DetailTab,
        label: &'static str,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.active_tab == tab;

        div()
            .id(SharedString::from(format!("tab-{:?}", tab)))
            .px_3()
            .py_1p5()
            .text_sm()
            .cursor_pointer()
            .rounded_md()
            .when(is_active, |el| {
                el.bg(colors::surface_elevated())
                    .text_color(colors::text())
            })
            .when(!is_active, |el| {
                el.text_color(colors::text_muted())
                    .hover(|el| el.text_color(colors::text()))
            })
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.set_tab(tab, cx);
            }))
            .child(label)
    }

    fn render_no_selection(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_color(colors::text_muted())
            .child("No Selection")
    }

    fn render_detail_content(&self, container: &ContainerViewModel) -> impl IntoElement {
        match self.active_tab {
            DetailTab::Info => self.render_info_tab(container).into_any_element(),
            DetailTab::Logs => self.render_logs_tab().into_any_element(),
            DetailTab::Terminal => self.render_terminal_tab().into_any_element(),
            DetailTab::Files => self.render_files_tab().into_any_element(),
        }
    }

    fn render_info_tab(&self, container: &ContainerViewModel) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            // Basic info section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(Theme::info_row("Name", container.name.clone()))
                    .child(Theme::info_row("ID", container.id.clone()))
                    .child(Theme::info_row("Image", container.image.clone()))
                    .child(Theme::info_row(
                        "Status",
                        if container.is_running() {
                            format!("Up {}", container.created_ago())
                        } else {
                            "Stopped".to_string()
                        },
                    ))
                    .child(Theme::info_row(
                        "Ports",
                        container.ports_display(),
                    )),
            )
            // Resource usage (if running)
            .when(container.is_running(), |el| {
                el.child(
                    div()
                        .mt_4()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors::text())
                                .child("Resources"),
                        )
                        .child(Theme::info_row(
                            "CPU",
                            format!("{:.1}%", container.cpu_percent),
                        ))
                        .child(Theme::info_row(
                            "Memory",
                            format!(
                                "{:.0} MB / {:.0} MB",
                                container.memory_mb, container.memory_limit_mb
                            ),
                        )),
                )
            })
    }

    fn render_logs_tab(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_color(colors::text_muted())
            .child("Logs viewer coming soon...")
    }

    fn render_terminal_tab(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_color(colors::text_muted())
            .child("Terminal coming soon...")
    }

    fn render_files_tab(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_color(colors::text_muted())
            .child("File browser coming soon...")
    }
}
