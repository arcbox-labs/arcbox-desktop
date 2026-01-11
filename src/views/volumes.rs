use gpui::*;
use gpui::prelude::*;
use gpui_component::tab::TabBar;
use gpui_component::Sizable;

use crate::models::VolumeViewModel;
use crate::theme::{colors, Theme};

/// Detail tab for volumes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VolumeDetailTab {
    #[default]
    Info,
    Files,
}

impl VolumeDetailTab {
    const ALL: [VolumeDetailTab; 2] = [VolumeDetailTab::Info, VolumeDetailTab::Files];

    fn label(&self) -> &'static str {
        match self {
            VolumeDetailTab::Info => "Info",
            VolumeDetailTab::Files => "Files",
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

/// Volumes list view
pub struct VolumesView {
    volumes: Vec<VolumeViewModel>,
    selected_id: Option<String>,
    active_tab: VolumeDetailTab,
    list_width: f32,
}

impl VolumesView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            volumes: Vec::new(),
            selected_id: None,
            active_tab: VolumeDetailTab::Info,
            list_width: LIST_DEFAULT_WIDTH,
        }
    }

    fn resize_list(&mut self, new_width: f32, cx: &mut Context<Self>) {
        self.list_width = new_width.clamp(LIST_MIN_WIDTH, LIST_MAX_WIDTH);
        cx.notify();
    }

    fn select_volume(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }

    fn set_tab(&mut self, tab: VolumeDetailTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
    }

    fn get_selected_volume(&self) -> Option<&VolumeViewModel> {
        self.selected_id
            .as_ref()
            .and_then(|id| self.volumes.iter().find(|v| &v.name == id))
    }
}

impl Render for VolumesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let total_size: u64 = self.volumes.iter().filter_map(|v| v.size_bytes).sum();
        let list_width = self.list_width;
        let sidebar_width: f32 = 180.0;

        div()
            .size_full()
            .flex()
            .flex_row()
            .overflow_hidden()
            // Left panel - volume list
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
                                            .child("Volumes"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(colors::text_muted())
                                            .child(format_size(total_size)),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        Theme::button_icon()
                                            .id("sort-volumes")
                                            .child(svg().path("icons/sort.svg").size(px(14.0)).text_color(colors::text_muted()))
                                    )
                                    .child(
                                        Theme::button_icon()
                                            .id("search-volumes")
                                            .child(svg().path("icons/search.svg").size(px(14.0)).text_color(colors::text_muted()))
                                    )
                                    .child(
                                        Theme::button_icon()
                                            .id("add-volume")
                                            .child(svg().path("icons/add.svg").size(px(14.0)).text_color(colors::text_muted()))
                                    ),
                            ),
                    )
                    // "In Use" section header
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .text_xs()
                            .text_color(colors::text_muted())
                            .child("In Use"),
                    )
                    // Volume list
                    .child(
                        div()
                            .id("volumes-list")
                            .flex_1()
                            .overflow_y_scroll()
                            .when(self.volumes.is_empty(), |el| {
                                el.child(self.render_empty_state())
                            })
                            .when(!self.volumes.is_empty(), |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .children(
                                            self.volumes
                                                .iter()
                                                .map(|volume| self.render_volume_row(volume, cx)),
                                        ),
                                )
                            }),
                    ),
            )
            // Resize handle
            .child(
                div()
                    .id("volume-list-resize")
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

impl VolumesView {
    fn render_volume_row(
        &self,
        volume: &VolumeViewModel,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let id = volume.name.clone();
        let id_for_select = volume.name.clone();
        let is_selected = self.selected_id.as_ref() == Some(&id);

        let base = div()
            .id(SharedString::from(format!("volume-{}", &id)))
            .flex()
            .items_center()
            .gap_3()
            .px_4()
            .py_2()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.select_volume(id_for_select.clone(), cx);
            }));

        let base = if is_selected {
            base.bg(colors::selection()).text_color(colors::on_accent())
        } else {
            base.hover(|el| el.bg(colors::hover()))
        };

        base
            // Volume icon
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
                    .child("▤"),
            )
            // Name and size
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
                            .child(volume.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .when(is_selected, |el| el.text_color(rgba(0xffffffaa)))
                            .when(!is_selected, |el| el.text_color(colors::text_muted()))
                            .child(volume.size_display()),
                    ),
            )
            // Delete button
            .child({
                let icon_color = if is_selected { colors::on_accent() } else { colors::text_muted() };
                Theme::button_icon()
                    .w(px(24.0))
                    .h(px(24.0))
                    .child(svg().path("icons/delete.svg").size(px(14.0)).text_color(icon_color))
            })
    }

    fn render_detail_panel(&self, cx: &Context<Self>) -> impl IntoElement {
        let selected = self.get_selected_volume();
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
                        TabBar::new("volume-detail-tabs")
                            .small()
                            .segmented()
                            .children(VolumeDetailTab::ALL.iter().map(|tab| tab.label()))
                            .selected_index(selected_index)
                            .on_click(cx.listener(|this, index: &usize, _window, cx| {
                                this.set_tab(VolumeDetailTab::from_index(*index), cx);
                            })),
                    ),
            )
            // Tab content
            .child(
                div()
                    .id("volume-detail-content")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_4()
                    .child(if let Some(volume) = selected {
                        self.render_detail_content(volume).into_any_element()
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
                    .child("No volumes yet"),
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
                            .child("Create a volume:"),
                    )
                    .child(Self::render_command_hint(
                        "docker volume create mydata",
                        "Create named volume",
                    ))
                    .child(Self::render_command_hint(
                        "docker run -v mydata:/data nginx",
                        "Mount volume to container",
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

    fn render_detail_content(&self, volume: &VolumeViewModel) -> impl IntoElement {
        match self.active_tab {
            VolumeDetailTab::Info => self.render_info_tab(volume).into_any_element(),
            VolumeDetailTab::Files => self.render_placeholder_tab("Files").into_any_element(),
        }
    }

    fn render_info_tab(&self, volume: &VolumeViewModel) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(Theme::info_row("Name", volume.name.clone()))
                    .child(Theme::info_row("Driver", volume.driver.clone()))
                    .child(Theme::info_row("Size", volume.size_display()))
                    .child(Theme::info_row("Created", volume.created_ago())),
            )
    }

    fn render_placeholder_tab(&self, name: &'static str) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_color(colors::text_muted())
            .child(format!("{} coming soon...", name))
    }
}

fn format_size(bytes: u64) -> String {
    let gb = bytes as f64 / 1_000_000_000.0;
    if gb >= 1.0 {
        format!("{:.2} GB total", gb)
    } else {
        let mb = bytes as f64 / 1_000_000.0;
        format!("{:.0} MB total", mb)
    }
}
