use gpui::*;
use gpui::prelude::*;
use gpui_component::tab::TabBar;
use gpui_component::Sizable;

use crate::models::NetworkViewModel;
use crate::theme::{colors, Theme, MONO_FONT};

/// Detail tab for networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkDetailTab {
    #[default]
    Info,
    Containers,
}

impl NetworkDetailTab {
    const ALL: [NetworkDetailTab; 2] = [NetworkDetailTab::Info, NetworkDetailTab::Containers];

    fn label(&self) -> &'static str {
        match self {
            NetworkDetailTab::Info => "Info",
            NetworkDetailTab::Containers => "Containers",
        }
    }

    fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or_default()
    }

    fn to_index(&self) -> usize {
        Self::ALL.iter().position(|t| t == self).unwrap_or(0)
    }
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
const LIST_DEFAULT_WIDTH: f32 = 380.0;

/// Networks list view
pub struct NetworksView {
    networks: Vec<NetworkViewModel>,
    selected_id: Option<String>,
    active_tab: NetworkDetailTab,
    list_width: f32,
}

impl NetworksView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            networks: Vec::new(),
            selected_id: None,
            active_tab: NetworkDetailTab::Info,
            list_width: LIST_DEFAULT_WIDTH,
        }
    }

    fn resize_list(&mut self, new_width: f32, cx: &mut Context<Self>) {
        self.list_width = new_width.clamp(LIST_MIN_WIDTH, LIST_MAX_WIDTH);
        cx.notify();
    }

    fn select_network(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }

    fn set_tab(&mut self, tab: NetworkDetailTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
    }

    fn get_selected_network(&self) -> Option<&NetworkViewModel> {
        self.selected_id
            .as_ref()
            .and_then(|id| self.networks.iter().find(|n| &n.id == id))
    }
}

impl Render for NetworksView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let network_count = self.networks.len();
        let list_width = self.list_width;
        let sidebar_width: f32 = 180.0;

        div()
            .size_full()
            .flex()
            .flex_row()
            .overflow_hidden()
            // Left panel - network list
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
                                            .child("Networks"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(colors::text_secondary())
                                            .child(format!("{} networks", network_count)),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        Theme::button_icon()
                                            .id("sort-networks")
                                            .child(svg().path("icons/sort.svg").size(px(16.0)).text_color(colors::text_secondary()))
                                    )
                                    .child(
                                        Theme::button_icon()
                                            .id("search-networks")
                                            .child(svg().path("icons/search.svg").size(px(16.0)).text_color(colors::text_secondary()))
                                    )
                                    .child(
                                        Theme::button_icon()
                                            .id("add-network")
                                            .child(svg().path("icons/add.svg").size(px(16.0)).text_color(colors::text_secondary()))
                                    ),
                            ),
                    )
                    // Network list
                    .child(
                        div()
                            .id("networks-list")
                            .flex_1()
                            .overflow_y_scroll()
                            .when(self.networks.is_empty(), |el| {
                                el.child(self.render_empty_state())
                            })
                            .when(!self.networks.is_empty(), |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .children(
                                            self.networks
                                                .iter()
                                                .map(|network| self.render_network_row(network, cx)),
                                        ),
                                )
                            }),
                    ),
            )
            // Resize handle
            .child(
                div()
                    .id("network-list-resize")
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

impl NetworksView {
    fn render_network_row(
        &self,
        network: &NetworkViewModel,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let id = network.id.clone();
        let id_for_select = network.id.clone();
        let is_selected = self.selected_id.as_ref() == Some(&id);
        let is_system = network.is_system();

        let base = div()
            .id(SharedString::from(format!("network-{}", &id)))
            .flex()
            .items_center()
            .gap_3()
            .px_4()
            .py_2()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.select_network(id_for_select.clone(), cx);
            }));

        let base = if is_selected {
            base.bg(colors::selection()).text_color(colors::on_accent())
        } else {
            base.hover(|el| el.bg(colors::hover()))
        };

        base
            // Network icon
            .child(
                div()
                    .w(px(32.0))
                    .h(px(32.0))
                    .rounded_md()
                    .bg(if is_selected {
                        rgba(0xffffff30)
                    } else {
                        colors::surface_elevated()
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .child(if is_system { "🌐" } else { "🔗" }),
            )
            // Name and driver
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
                            .child(network.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .when(is_selected, |el| el.text_color(rgba(0xffffffaa)))
                            .when(!is_selected, |el| el.text_color(colors::text_secondary()))
                            .child(network.driver_display()),
                    ),
            )
            // Delete button (only for non-system networks)
            .when(!is_system, |el| {
                let icon_color = if is_selected { colors::on_accent() } else { colors::text_secondary() };
                el.child(
                    Theme::button_icon()
                        .w(px(24.0))
                        .h(px(24.0))
                        .child(svg().path("icons/delete.svg").size(px(16.0)).text_color(icon_color))
                )
            })
    }

    fn render_detail_panel(&self, cx: &Context<Self>) -> impl IntoElement {
        let selected = self.get_selected_network();
        let selected_index = self.active_tab.to_index();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(colors::background())
            // Tab bar
            .child(
                div()
                    .h(px(52.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .border_b_1()
                    .border_color(colors::border())
                    .child(
                        TabBar::new("network-detail-tabs")
                            .segmented()
                            .children(NetworkDetailTab::ALL.iter().map(|tab| tab.label()))
                            .selected_index(selected_index)
                            .on_click(cx.listener(|this, index: &usize, _window, cx| {
                                this.set_tab(NetworkDetailTab::from_index(*index), cx);
                            })),
                    ),
            )
            // Tab content
            .child(
                div()
                    .id("network-detail-content")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_4()
                    .child(if let Some(network) = selected {
                        self.render_detail_content(network).into_any_element()
                    } else {
                        self.render_no_selection().into_any_element()
                    }),
            )
    }

    fn render_no_selection(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_color(colors::text_secondary())
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
                    .text_color(colors::text_secondary())
                    .text_sm()
                    .child("No networks yet"),
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
                            .text_color(colors::text_secondary())
                            .child("Create a network:"),
                    )
                    .child(Self::render_command_hint(
                        "docker network create mynet",
                        "Create bridge network",
                    ))
                    .child(Self::render_command_hint(
                        "docker network create --driver overlay mynet",
                        "Create overlay network",
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
                    .font_family(MONO_FONT)
                    .text_xs()
                    .text_color(colors::text())
                    .child(command),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors::text_secondary())
                    .child(desc),
            )
    }

    fn render_detail_content(&self, network: &NetworkViewModel) -> impl IntoElement {
        match self.active_tab {
            NetworkDetailTab::Info => self.render_info_tab(network).into_any_element(),
            NetworkDetailTab::Containers => self.render_placeholder_tab("Containers").into_any_element(),
        }
    }

    fn render_info_tab(&self, network: &NetworkViewModel) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(Theme::info_row("Name", network.name.clone()))
                    .child(Theme::info_row("ID", network.short_id()))
                    .child(Theme::info_row("Driver", network.driver.clone()))
                    .child(Theme::info_row("Scope", network.scope.clone()))
                    .child(Theme::info_row("Created", network.created_ago()))
                    .child(Theme::info_row("Internal", if network.internal { "Yes" } else { "No" }))
                    .child(Theme::info_row("Attachable", if network.attachable { "Yes" } else { "No" }))
                    .child(Theme::info_row("Containers", network.usage_display())),
            )
    }

    fn render_placeholder_tab(&self, name: &'static str) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_color(colors::text_secondary())
            .child(format!("{} coming soon...", name))
    }
}
