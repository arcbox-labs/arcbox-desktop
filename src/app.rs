use gpui::*;
use gpui::prelude::*;

use crate::services::{
    DaemonEvent, DaemonManager, DaemonManagerEvent, DaemonService, DaemonState, ImageIconService,
};
use crate::theme::{colors, Theme};
use crate::views::*;

// Define actions using the actions! macro
actions!(arcbox, [OpenSettings, Quit]);

/// Sidebar resize drag state
#[derive(Clone)]
struct SidebarResizeDrag {
    initial_width: f32,
}

/// Navigation item in sidebar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavItem {
    // Docker section
    #[default]
    Containers,
    Volumes,
    Images,
    Networks,
    // Linux section
    Machines,
}

impl NavItem {
    fn label(&self) -> &'static str {
        match self {
            NavItem::Containers => "Containers",
            NavItem::Volumes => "Volumes",
            NavItem::Images => "Images",
            NavItem::Networks => "Networks",
            NavItem::Machines => "Machines",
        }
    }

    fn icon_path(&self) -> &'static str {
        match self {
            NavItem::Containers => "icons/container.svg",
            NavItem::Volumes => "icons/volume.svg",
            NavItem::Images => "icons/image.svg",
            NavItem::Networks => "icons/network.svg",
            NavItem::Machines => "icons/machine.svg",
        }
    }
}

/// Main application state
pub struct ArcBoxApp {
    current_nav: NavItem,
    sidebar_width: f32,
    // Lifecycle management
    daemon_manager: Entity<DaemonManager>,
    // Shared services
    daemon_service: Entity<DaemonService>,
    image_icon_service: Entity<ImageIconService>,
    // Views
    containers_view: Entity<ContainersView>,
    machines_view: Entity<MachinesView>,
    images_view: Entity<ImagesView>,
    volumes_view: Entity<VolumesView>,
}

const SIDEBAR_MIN_WIDTH: f32 = 120.0;
const SIDEBAR_MAX_WIDTH: f32 = 300.0;
const SIDEBAR_DEFAULT_WIDTH: f32 = 180.0;

impl ArcBoxApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Create daemon manager first (handles process lifecycle)
        let daemon_manager = cx.new(DaemonManager::new);

        // Create daemon service with gRPC socket path from manager
        let grpc_socket_path = daemon_manager.read(cx).grpc_socket_path();
        let daemon_service = cx.new(|cx| DaemonService::with_socket_path(grpc_socket_path, cx));

        let image_icon_service = cx.new(ImageIconService::new);

        // Create views (some may depend on shared services)
        let containers_view = cx.new(|cx| {
            ContainersView::new(daemon_service.clone(), image_icon_service.clone(), cx)
        });
        let machines_view = cx.new(MachinesView::new);
        let images_view = cx.new(|cx| {
            ImagesView::new(daemon_service.clone(), image_icon_service.clone(), cx)
        });
        let volumes_view = cx.new(VolumesView::new);

        // Subscribe to daemon manager events - connect when daemon is ready
        let daemon_service_clone = daemon_service.clone();
        cx.subscribe(
            &daemon_manager,
            move |_this, _, event: &DaemonManagerEvent, cx| {
                if let DaemonManagerEvent::StateChanged(DaemonState::Running) = event {
                    tracing::info!("Daemon is running, connecting gRPC client...");
                    daemon_service_clone.update(cx, |svc, cx| {
                        svc.connect(cx);
                    });
                }
            },
        )
        .detach();

        // Subscribe to daemon service events and forward to views
        cx.subscribe(&daemon_service, |this, _, event: &DaemonEvent, cx| {
            match event {
                DaemonEvent::ContainersLoaded(response) => {
                    this.containers_view.update(cx, |view, cx| {
                        view.on_containers_loaded(response.clone(), cx);
                    });
                }
                DaemonEvent::MachinesLoaded(_response) => {
                    // TODO: Forward to machines view
                }
                DaemonEvent::ImagesLoaded(response) => {
                    this.images_view.update(cx, |view, cx| {
                        view.on_images_loaded(response.clone(), cx);
                    });
                }
                DaemonEvent::ContainerCreated(id) => {
                    tracing::info!("Container created: {}", id);
                }
                DaemonEvent::ContainerStarted(id) => {
                    tracing::info!("Container started: {}", id);
                }
                DaemonEvent::ContainerStopped(id) => {
                    tracing::info!("Container stopped: {}", id);
                }
                DaemonEvent::ContainerRemoved(id) => {
                    tracing::info!("Container removed: {}", id);
                }
                DaemonEvent::OperationFailed(error) => {
                    tracing::error!("Operation failed: {}", error);
                    // TODO: Show error notification to user
                }
            }
        })
        .detach();

        // Start daemon on app launch
        daemon_manager.update(cx, |mgr, cx| {
            mgr.start(cx);
        });

        Self {
            current_nav: NavItem::Containers,
            sidebar_width: SIDEBAR_DEFAULT_WIDTH,
            daemon_manager,
            daemon_service,
            image_icon_service,
            containers_view,
            machines_view,
            images_view,
            volumes_view,
        }
    }

    fn navigate(&mut self, item: NavItem, cx: &mut Context<Self>) {
        self.current_nav = item;
        cx.notify();
    }

    fn resize_sidebar(&mut self, new_width: f32, cx: &mut Context<Self>) {
        self.sidebar_width = new_width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        cx.notify();
    }
}

impl Render for ArcBoxApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar_width = self.sidebar_width;

        div()
            .size_full()
            .relative()
            .flex()
            .flex_row()
            .bg(colors::background())
            .text_color(colors::text())
            // Sidebar
            .child(self.render_sidebar(cx))
            // Resize handle
            .child(self.render_resize_handle(sidebar_width, cx))
            // Main content
            .child(self.render_main_content())
    }
}

impl ArcBoxApp {
    fn render_sidebar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .w(px(self.sidebar_width))
            .h_full()
            .flex()
            .flex_col()
            .bg(colors::sidebar())
            .flex_shrink_0()
            // Titlebar area (for traffic lights on macOS)
            .child(div().h(px(52.0)))
            // Docker section
            .child(Theme::sidebar_section_header("Docker"))
            .child(self.render_nav_item(NavItem::Containers, cx))
            .child(self.render_nav_item(NavItem::Volumes, cx))
            .child(self.render_nav_item(NavItem::Images, cx))
            .child(self.render_nav_item(NavItem::Networks, cx))
            // Linux section
            .child(div().h(px(8.0))) // Spacer
            .child(Theme::sidebar_section_header("Linux"))
            .child(self.render_nav_item(NavItem::Machines, cx))
            // Bottom spacer
            .child(div().flex_1())
    }

    fn render_resize_handle(&self, current_width: f32, cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("sidebar-resize-handle")
            .w(px(4.0))
            .h_full()
            .cursor(CursorStyle::ResizeLeftRight)
            .bg(colors::sidebar())
            .hover(|el| el.bg(colors::border()))
            .on_drag(
                SidebarResizeDrag {
                    initial_width: current_width,
                },
                |drag, _point, _window, cx| {
                    cx.new(|_cx| ResizeHandleVisual {
                        initial_width: drag.initial_width,
                    })
                },
            )
            .on_drag_move::<SidebarResizeDrag>(cx.listener(
                move |this, event: &DragMoveEvent<SidebarResizeDrag>, _window, cx| {
                    // The sidebar starts at x=0, so the mouse x position is approximately
                    // where the user wants the sidebar edge to be
                    let new_width: f32 = event.event.position.x.into();
                    this.resize_sidebar(new_width, cx);
                },
            ))
    }

    fn render_nav_item(&self, item: NavItem, cx: &Context<Self>) -> impl IntoElement {
        let is_active = self.current_nav == item;

        div()
            .id(SharedString::from(format!("nav-{:?}", item)))
            .mx_2()
            .ml_3() // Indent nav items
            .px_2()
            .py_1()
            .rounded_md()
            .flex()
            .items_center()
            .gap_2()
            .text_sm()
            .cursor_pointer()
            .when(is_active, |el| {
                el.bg(colors::selection()).text_color(colors::on_accent())
            })
            .when(!is_active, |el| {
                el.text_color(colors::text())
                    .hover(|el| el.bg(colors::hover()))
            })
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.navigate(item, cx);
            }))
            .child(
                svg()
                    .path(item.icon_path())
                    .size(px(16.0))
                    .flex_shrink_0()
                    .text_color(if is_active {
                        colors::on_accent()
                    } else {
                        colors::text()
                    }),
            )
            .child(item.label())
    }

    fn render_main_content(&self) -> impl IntoElement {
        div()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .bg(colors::background())
            .child(match self.current_nav {
                NavItem::Containers => self.containers_view.clone().into_any_element(),
                NavItem::Machines => self.machines_view.clone().into_any_element(),
                NavItem::Images => self.images_view.clone().into_any_element(),
                NavItem::Volumes => self.volumes_view.clone().into_any_element(),
                NavItem::Networks => div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(colors::text_muted())
                    .child("Networks (Coming Soon)")
                    .into_any_element(),
            })
    }
}

/// Visual element shown during drag (invisible)
struct ResizeHandleVisual {
    #[allow(dead_code)]
    initial_width: f32,
}

impl Render for ResizeHandleVisual {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Invisible during drag
        div().w(px(0.0)).h(px(0.0))
    }
}

// ===== Settings Window =====

/// Open settings window (called from menu or Cmd+,)
pub fn open_settings(cx: &mut App) {
    // Check if settings window already exists
    for window in cx.windows() {
        if let Some(handle) = window.downcast::<SettingsView>() {
            // Focus existing window
            let _ = handle.update(cx, |_, window, _cx| {
                window.activate_window();
            });
            return;
        }
    }

    // Create new settings window
    let bounds = Bounds::centered(None, size(px(700.0), px(500.0)), cx);
    let window_options = WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some("Settings".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(9.0), px(9.0))),
        }),
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        focus: true,
        show: true,
        kind: WindowKind::Normal,
        is_movable: true,
        window_background: WindowBackgroundAppearance::Opaque,
        ..Default::default()
    };

    cx.open_window(window_options, |_window, cx| cx.new(SettingsView::new))
        .expect("Failed to open settings window");
}
