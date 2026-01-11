use std::collections::HashMap;

use arcbox_api::generated::ListContainersResponse;
use gpui::*;
use gpui::prelude::*;

use crate::models::ContainerViewModel;
use crate::services::{DaemonService, ImageIconService, IconState};
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

/// Drag state for resizing the list panel
#[derive(Clone)]
struct ListPanelDrag;

/// Empty view for drag visual (invisible)
struct EmptyView;

impl Render for EmptyView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
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
    daemon_service: Entity<DaemonService>,
    icon_service: Entity<ImageIconService>,
    /// Loading state for container list
    is_loading: bool,
}

impl ContainersView {
    pub fn new(
        daemon_service: Entity<DaemonService>,
        icon_service: Entity<ImageIconService>,
        cx: &mut Context<Self>,
    ) -> Self {
        // Subscribe to icon service updates for re-rendering
        cx.observe(&icon_service, |_, _, cx| cx.notify()).detach();

        // Subscribe to daemon service connection state changes
        cx.observe(&daemon_service, |this, daemon, cx| {
            if daemon.read(cx).is_connected() && this.is_loading {
                // Request container list when connected
                daemon.update(cx, |svc, cx| {
                    svc.list_containers(true, cx);
                });
            }
            cx.notify();
        })
        .detach();

        Self {
            containers: Vec::new(),
            selected_id: None,
            expanded_groups: HashMap::new(),
            active_tab: DetailTab::Info,
            list_width: LIST_DEFAULT_WIDTH,
            daemon_service,
            icon_service,
            is_loading: true,
        }
    }

    /// Handle containers loaded from daemon
    pub fn on_containers_loaded(&mut self, response: ListContainersResponse, cx: &mut Context<Self>) {
        self.is_loading = false;
        self.containers = response
            .containers
            .into_iter()
            .map(ContainerViewModel::from)
            .collect();

        // Update expanded groups
        self.expanded_groups.clear();
        for c in &self.containers {
            if let Some(ref project) = c.compose_project {
                self.expanded_groups.insert(project.clone(), true);
            }
        }

        // Pre-fetch icons for new containers
        for container in &self.containers {
            let repo = Self::extract_repository(&container.image);
            self.icon_service.update(cx, |svc, cx| {
                svc.get_icon(&repo, cx);
            });
        }

        cx.notify();
    }

    /// Refresh container list from daemon
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.is_loading = true;
        self.daemon_service.update(cx, |svc, cx| {
            svc.list_containers(true, cx);
        });
        cx.notify();
    }

    /// Extract repository name from image string (e.g., "nginx:latest" -> "nginx")
    fn extract_repository(image: &str) -> String {
        // Handle formats like "nginx:latest", "postgres:15", "my-app:dev"
        image.split(':').next().unwrap_or(image).to_string()
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

    fn start_container(&mut self, id: &str, cx: &mut Context<Self>) {
        tracing::info!("Starting container: {}", id);
        let id = id.to_string();
        self.daemon_service.update(cx, |svc, cx| {
            svc.start_container(id, cx);
        });
    }

    fn stop_container(&mut self, id: &str, cx: &mut Context<Self>) {
        tracing::info!("Stopping container: {}", id);
        let id = id.to_string();
        self.daemon_service.update(cx, |svc, cx| {
            svc.stop_container(id, 10, cx);
        });
    }

    fn remove_container(&mut self, id: &str, cx: &mut Context<Self>) {
        tracing::info!("Removing container: {}", id);
        let id = id.to_string();
        self.daemon_service.update(cx, |svc, cx| {
            svc.remove_container(id, false, cx);
        });
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
        // Sidebar width for offset calculation
        let sidebar_width: f32 = 180.0;

        div()
            .size_full()
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
                                            .child(
                                                svg()
                                                    .path("icons/add.svg")
                                                    .size(px(14.0))
                                                    .text_color(colors::text_muted())
                                            ),
                                    )
                                    .child(
                                        Theme::button_icon()
                                            .id("search-containers")
                                            .child(
                                                svg()
                                                    .path("icons/search.svg")
                                                    .size(px(14.0))
                                                    .text_color(colors::text_muted())
                                            ),
                                    ),
                            ),
                    )
                    // Container list
                    .child(
                        div()
                            .id("containers-list")
                            .flex_1()
                            .overflow_y_scroll()
                            .when(self.containers.is_empty(), |el| {
                                el.child(self.render_empty_state())
                            })
                            .when(!self.containers.is_empty(), |el| {
                                el.child(
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
                                )
                            }),
                    ),
            )
            // Resize handle
            .child(
                div()
                    .id("container-list-resize")
                    .w(px(1.0))
                    .h_full()
                    .flex_shrink_0()
                    .cursor(CursorStyle::ResizeLeftRight)
                    .bg(colors::border())
                    .on_drag(ListPanelDrag, |_, _, _, cx| cx.new(|_| EmptyView))
                    .on_drag_move::<ListPanelDrag>(cx.listener(
                        move |this, event: &DragMoveEvent<ListPanelDrag>, _, cx| {
                            let x: f32 = event.event.position.x.into();
                            let new_width = x - sidebar_width;
                            this.resize_list(new_width, cx);
                        },
                    )),
            )
            // Right panel - detail
            .child(self.render_detail_panel(cx))
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
                            .w(px(28.0))
                            .h(px(28.0))
                            .rounded_md()
                            .bg(colors::accent())
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                svg()
                                    .path("icons/layer.svg")
                                    .size(px(18.0))
                                    .text_color(colors::on_accent()),
                            ),
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
        let id_for_delete = container.id.clone();
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

        // Check if we have a real icon or need fallback
        let repo = Self::extract_repository(&container.image);
        let has_icon = matches!(
            self.icon_service.read(cx).get_cached(&repo),
            Some(IconState::Found(_))
        );

        base
            // Container icon with status dot
            .child(
                div()
                    .relative()
                    .w(px(28.0))
                    .h(px(28.0))
                    // Only show background and rounded for fallback icons
                    .when(!has_icon, |el| {
                        el.rounded_md()
                            .bg(if is_selected {
                                rgba(0xffffff30)
                            } else {
                                colors::surface_elevated()
                            })
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .child(self.render_container_icon(&container.image, is_selected, cx))
                    // Status dot (bottom-right)
                    .child(
                        div()
                            .absolute()
                            .right(px(-1.0))
                            .bottom(px(-1.0))
                            .w(px(10.0))
                            .h(px(10.0))
                            .rounded_full()
                            .border_2()
                            .border_color(if is_selected {
                                colors::selection()
                            } else {
                                colors::background()
                            })
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
                        let icon_color = if is_selected { colors::on_accent() } else { colors::text_muted() };
                        Theme::button_icon()
                            .id(SharedString::from(format!("stop-{}", &id)))
                            .w(px(24.0))
                            .h(px(24.0))
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.stop_container(&id_for_action, cx);
                            }))
                            .child(
                                svg()
                                    .path("icons/stop.svg")
                                    .size(px(14.0))
                                    .text_color(icon_color)
                            )
                    } else {
                        let icon_color = if is_selected { colors::on_accent() } else { colors::text_muted() };
                        Theme::button_icon()
                            .id(SharedString::from(format!("start-{}", &id)))
                            .w(px(24.0))
                            .h(px(24.0))
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.start_container(&id_for_action, cx);
                            }))
                            .child(
                                svg()
                                    .path("icons/play.svg")
                                    .size(px(14.0))
                                    .text_color(icon_color)
                            )
                    })
                    .child({
                        let icon_color = if is_selected { colors::on_accent() } else { colors::text_muted() };
                        Theme::button_icon()
                            .id(SharedString::from(format!("delete-{}", &id)))
                            .w(px(24.0))
                            .h(px(24.0))
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.remove_container(&id_for_delete, cx);
                            }))
                            .child(
                                svg()
                                    .path("icons/delete.svg")
                                    .size(px(14.0))
                                    .text_color(icon_color)
                            )
                    }),
            )
    }

    fn render_container_icon(
        &self,
        image: &str,
        is_selected: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let repo = Self::extract_repository(image);
        let icon_state = self.icon_service.read(cx).get_cached(&repo).cloned();

        match icon_state {
            Some(IconState::Found(url)) => {
                // Display fetched icon, fill the container
                img(url)
                    .w(px(28.0))
                    .h(px(28.0))
                    .into_any_element()
            }
            // For Loading/NotFound/Error/None, show colored box icon
            _ => {
                let color = Self::get_color_for_repository(&repo);
                svg()
                    .path("icons/box.svg")
                    .size(px(16.0))
                    .text_color(if is_selected { colors::on_accent() } else { color })
                    .into_any_element()
            }
        }
    }

    /// Generate a consistent color based on repository name
    fn get_color_for_repository(repository: &str) -> Rgba {
        const COLORS: &[(u8, u8, u8)] = &[
            (239, 68, 68),   // red
            (249, 115, 22),  // orange
            (234, 179, 8),   // yellow
            (34, 197, 94),   // green
            (6, 182, 212),   // cyan
            (59, 130, 246),  // blue
            (139, 92, 246),  // violet
            (236, 72, 153),  // pink
            (168, 85, 247),  // purple
            (20, 184, 166),  // teal
        ];

        let hash: usize = repository.bytes().fold(0usize, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as usize)
        });
        let (r, g, b) = COLORS[hash % COLORS.len()];

        rgba(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xFF)
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

    fn render_empty_state(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .p_6()
            .child(
                div()
                    .text_color(colors::text_muted())
                    .text_sm()
                    .child("No containers yet"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .rounded_lg()
                    .bg(colors::surface_elevated())
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::text_muted())
                            .child("Quick start:"),
                    )
                    .child(Self::render_command_hint(
                        "docker run -d nginx",
                        "Run nginx server",
                    ))
                    .child(Self::render_command_hint(
                        "docker run -it ubuntu bash",
                        "Interactive Ubuntu shell",
                    ))
                    .child(Self::render_command_hint(
                        "docker compose up -d",
                        "Start compose project",
                    )),
            )
    }

    fn render_command_hint(command: &'static str, desc: &'static str) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_0p5()
            .child(
                div()
                    .px_2()
                    .py_1()
                    .rounded(px(4.0))
                    .bg(colors::background())
                    .font_family("monospace")
                    .text_xs()
                    .text_color(colors::text())
                    .child(command),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors::text_muted())
                    .child(desc),
            )
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
