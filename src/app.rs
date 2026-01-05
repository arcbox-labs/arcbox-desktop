use gpui::*;
use gpui::prelude::*;

use crate::theme::colors;
use crate::views::*;

/// Navigation item in sidebar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavItem {
    #[default]
    Containers,
    Machines,
    Images,
    Volumes,
    Settings,
}

impl NavItem {
    fn label(&self) -> &'static str {
        match self {
            NavItem::Containers => "Containers",
            NavItem::Machines => "Machines",
            NavItem::Images => "Images",
            NavItem::Volumes => "Volumes",
            NavItem::Settings => "Settings",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            NavItem::Containers => "📦",
            NavItem::Machines => "🖥",
            NavItem::Images => "🖼",
            NavItem::Volumes => "💾",
            NavItem::Settings => "⚙",
        }
    }

    fn all() -> &'static [NavItem] {
        &[
            NavItem::Containers,
            NavItem::Machines,
            NavItem::Images,
            NavItem::Volumes,
        ]
    }
}

/// Main application state
pub struct ArcBoxApp {
    current_nav: NavItem,
    containers_view: Entity<ContainersView>,
    machines_view: Entity<MachinesView>,
    images_view: Entity<ImagesView>,
    volumes_view: Entity<VolumesView>,
    settings_view: Entity<SettingsView>,
    search_query: String,
}

impl ArcBoxApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let containers_view = cx.new(ContainersView::new);
        let machines_view = cx.new(MachinesView::new);
        let images_view = cx.new(ImagesView::new);
        let volumes_view = cx.new(VolumesView::new);
        let settings_view = cx.new(SettingsView::new);

        Self {
            current_nav: NavItem::Containers,
            containers_view,
            machines_view,
            images_view,
            volumes_view,
            settings_view,
            search_query: String::new(),
        }
    }

    fn navigate(&mut self, item: NavItem, cx: &mut Context<Self>) {
        self.current_nav = item;
        cx.notify();
    }

    /// Get counts for sidebar badges
    fn get_running_counts(&self) -> (usize, usize) {
        // TODO: Get actual counts from views
        (4, 2) // (containers, machines)
    }
}

impl Render for ArcBoxApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (containers_count, machines_count) = self.get_running_counts();

        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(colors::background())
            .text_color(colors::text())
            // Sidebar
            .child(self.render_sidebar(containers_count, machines_count, cx))
            // Main content
            .child(self.render_main_content())
    }
}

impl ArcBoxApp {
    fn render_sidebar(
        &self,
        containers_count: usize,
        machines_count: usize,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .w(px(220.0))
            .h_full()
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(colors::border())
            .bg(colors::surface())
            // Titlebar area (for traffic lights on macOS)
            .child(
                div()
                    .h(px(52.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_lg()
                            .text_color(colors::text())
                            .child("ArcBox"),
                    ),
            )
            // Search box
            .child(
                div()
                    .px_3()
                    .pb_3()
                    .child(
                        div()
                            .w_full()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(colors::background())
                            .border_1()
                            .border_color(colors::border())
                            .text_sm()
                            .text_color(colors::text_muted())
                            .child(format!(
                                "🔍 {}",
                                if self.search_query.is_empty() {
                                    "Search..."
                                } else {
                                    &self.search_query
                                }
                            )),
                    ),
            )
            // Navigation items
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .px_2()
                    .children(NavItem::all().iter().map(|item| {
                        let count = match item {
                            NavItem::Containers => Some(containers_count),
                            NavItem::Machines => Some(machines_count),
                            _ => None,
                        };
                        self.render_nav_item(*item, count, cx)
                    })),
            )
            // Bottom settings
            .child(
                div()
                    .mt_auto()
                    .px_2()
                    .pb_3()
                    .border_t_1()
                    .border_color(colors::border())
                    .pt_3()
                    .child(self.render_nav_item(NavItem::Settings, None, cx)),
            )
    }

    fn render_nav_item(
        &self,
        item: NavItem,
        count: Option<usize>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.current_nav == item;

        div()
            .id(SharedString::from(format!("nav-{:?}", item)))
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .when(is_active, |el| {
                el.bg(colors::accent()).text_color(colors::on_accent())
            })
            .when(!is_active, |el| {
                el.text_color(colors::text_muted())
                    .hover(|el| el.bg(colors::hover()).text_color(colors::text()))
            })
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.navigate(item, cx);
            }))
            .child(div().child(item.icon()))
            .child(div().flex_1().text_sm().child(item.label()))
            .when_some(count, |el, count| {
                el.child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded(px(4.0))
                        .text_xs()
                        .when(is_active, |el| el.bg(rgba(0xffffff33)))
                        .when(!is_active, |el| el.bg(colors::surface_elevated()))
                        .child(count.to_string()),
                )
            })
    }

    fn render_main_content(&self) -> impl IntoElement {
        div()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .child(match self.current_nav {
                NavItem::Containers => self.containers_view.clone().into_any_element(),
                NavItem::Machines => self.machines_view.clone().into_any_element(),
                NavItem::Images => self.images_view.clone().into_any_element(),
                NavItem::Volumes => self.volumes_view.clone().into_any_element(),
                NavItem::Settings => self.settings_view.clone().into_any_element(),
            })
    }
}
