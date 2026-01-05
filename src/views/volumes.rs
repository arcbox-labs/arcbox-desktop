use gpui::*;
use gpui::prelude::*;

use crate::models::{VolumeViewModel, dummy_volumes};
use crate::theme::{colors, Theme};

/// Detail tab for volumes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VolumeDetailTab {
    #[default]
    Info,
    Files,
}

/// Volumes list view
pub struct VolumesView {
    volumes: Vec<VolumeViewModel>,
    selected_id: Option<String>,
    active_tab: VolumeDetailTab,
}

impl VolumesView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            volumes: dummy_volumes(),
            selected_id: None,
            active_tab: VolumeDetailTab::Info,
        }
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

        div()
            .flex_1()
            .flex()
            .flex_row()
            .overflow_hidden()
            // Left panel - volume list
            .child(
                div()
                    .w(px(380.0))
                    .h_full()
                    .flex()
                    .flex_col()
                    .border_r_1()
                    .border_color(colors::border())
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
                                    .child(Theme::button_icon().id("sort-volumes").child("↕"))
                                    .child(Theme::button_icon().id("search-volumes").child("⌕"))
                                    .child(Theme::button_icon().id("add-volume").child("+")),
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
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .children(
                                        self.volumes
                                            .iter()
                                            .map(|volume| self.render_volume_row(volume, cx)),
                                    ),
                            ),
                    ),
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
            .child(
                Theme::button_icon()
                    .w(px(24.0))
                    .h(px(24.0))
                    .when(is_selected, |el| el.text_color(colors::on_accent()))
                    .child("🗑"),
            )
    }

    fn render_detail_panel(&self, cx: &Context<Self>) -> impl IntoElement {
        let selected = self.get_selected_volume();

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
                    .child(self.render_tab_button(VolumeDetailTab::Info, "Info", cx))
                    .child(self.render_tab_button(VolumeDetailTab::Files, "Files", cx)),
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

    fn render_tab_button(
        &self,
        tab: VolumeDetailTab,
        label: &'static str,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.active_tab == tab;

        div()
            .id(SharedString::from(format!("volume-tab-{:?}", tab)))
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
