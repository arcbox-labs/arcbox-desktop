use gpui::*;
use gpui::prelude::*;

use crate::models::{ImageViewModel, calculate_image_stats, dummy_images};
use crate::services::{ImageIconService, IconState};
use crate::theme::{colors, Theme};

/// Detail tab for images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageDetailTab {
    #[default]
    Info,
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
const LIST_DEFAULT_WIDTH: f32 = 380.0;

/// Images list view
pub struct ImagesView {
    images: Vec<ImageViewModel>,
    selected_id: Option<String>,
    active_tab: ImageDetailTab,
    list_width: f32,
    icon_service: Entity<ImageIconService>,
}

impl ImagesView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let icon_service = cx.new(ImageIconService::new);

        Self {
            images: dummy_images(),
            selected_id: None,
            active_tab: ImageDetailTab::Info,
            list_width: LIST_DEFAULT_WIDTH,
            icon_service,
        }
    }

    fn resize_list(&mut self, new_width: f32, cx: &mut Context<Self>) {
        self.list_width = new_width.clamp(LIST_MIN_WIDTH, LIST_MAX_WIDTH);
        cx.notify();
    }

    fn select_image(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }

    fn set_tab(&mut self, tab: ImageDetailTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
    }

    fn get_selected_image(&self) -> Option<&ImageViewModel> {
        self.selected_id
            .as_ref()
            .and_then(|id| self.images.iter().find(|i| &i.id == id))
    }
}

impl Render for ImagesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Subscribe to icon service updates for re-rendering
        cx.observe(&self.icon_service, |_, _, cx| cx.notify())
            .detach();

        // Trigger icon fetches for all images
        for image in &self.images {
            let repo = image.repository.clone();
            self.icon_service.update(cx, |svc, cx| {
                svc.get_icon(&repo, cx);
            });
        }

        let (total_size, _unused_size, _total_count, _unused_count) =
            calculate_image_stats(&self.images);
        let list_width = self.list_width;
        let sidebar_width: f32 = 180.0;

        div()
            .size_full()
            .flex()
            .flex_row()
            .overflow_hidden()
            // Left panel - image list
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
                                            .child("Images"),
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
                                            .id("sort-images")
                                            .child(svg().path("icons/sort.svg").size(px(14.0)).text_color(colors::text_muted()))
                                    )
                                    .child(
                                        Theme::button_icon()
                                            .id("search-images")
                                            .child(svg().path("icons/search.svg").size(px(14.0)).text_color(colors::text_muted()))
                                    )
                                    .child(
                                        Theme::button_icon()
                                            .id("add-image")
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
                    // Image list
                    .child(
                        div()
                            .id("images-list")
                            .flex_1()
                            .overflow_y_scroll()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .children(
                                        self.images
                                            .iter()
                                            .map(|image| self.render_image_row(image, cx)),
                                    ),
                            ),
                    ),
            )
            // Resize handle
            .child(
                div()
                    .id("image-list-resize")
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

impl ImagesView {
    fn render_image_row(
        &self,
        image: &ImageViewModel,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let id = image.id.clone();
        let id_for_select = image.id.clone();
        let is_selected = self.selected_id.as_ref() == Some(&id);

        let base = div()
            .id(SharedString::from(format!("image-{}", &id)))
            .flex()
            .items_center()
            .gap_3()
            .px_4()
            .py_2()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.select_image(id_for_select.clone(), cx);
            }));

        let base = if is_selected {
            base.bg(colors::selection()).text_color(colors::on_accent())
        } else {
            base.hover(|el| el.bg(colors::hover()))
        };

        // Get icon state from service
        let icon_state = self.icon_service.read(cx).get_cached(&image.repository).cloned();

        base
            // Image icon
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
                    .overflow_hidden()
                    .child(self.render_image_icon(&image.repository, icon_state, is_selected)),
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
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_ellipsis()
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .child(format!("{}:{}", image.repository, image.tag)),
                            )
                            // Architecture badge
                            .when(image.architecture == "amd64", |el| {
                                el.child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(4.0))
                                        .text_xs()
                                        .bg(if is_selected {
                                            rgba(0xffffff30)
                                        } else {
                                            colors::surface_elevated()
                                        })
                                        .child("amd64"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .when(is_selected, |el| el.text_color(rgba(0xffffffaa)))
                            .when(!is_selected, |el| el.text_color(colors::text_muted()))
                            .child(format!(
                                "{}, {}",
                                image.size_display(),
                                image.created_ago()
                            )),
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

    fn render_image_icon(
        &self,
        repository: &str,
        icon_state: Option<IconState>,
        _is_selected: bool,
    ) -> impl IntoElement {
        match icon_state {
            Some(IconState::Found(url)) => {
                // Display fetched icon from URL
                img(url)
                    .w(px(24.0))
                    .h(px(24.0))
                    .rounded(px(4.0))
                    .into_any_element()
            }
            Some(IconState::Loading) => {
                // Show loading placeholder with box icon
                svg()
                    .path("icons/box.svg")
                    .size(px(20.0))
                    .text_color(colors::text_muted())
                    .into_any_element()
            }
            Some(IconState::NotFound) | Some(IconState::Error(_)) | None => {
                // Fallback to box icon with random color based on repository name
                let color = Self::get_color_for_repository(repository);
                svg()
                    .path("icons/box.svg")
                    .size(px(20.0))
                    .text_color(color)
                    .into_any_element()
            }
        }
    }

    /// Generate a consistent color based on repository name
    fn get_color_for_repository(repository: &str) -> Rgba {
        // Color palette - vibrant colors for visual distinction
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

        // Simple hash based on repository name for consistent color
        let hash: usize = repository.bytes().fold(0usize, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as usize)
        });
        let (r, g, b) = COLORS[hash % COLORS.len()];

        rgba(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xFF)
    }

    fn render_detail_panel(&self, cx: &Context<Self>) -> impl IntoElement {
        let selected = self.get_selected_image();

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
                    .child(self.render_tab_button(ImageDetailTab::Info, "Info", cx))
                    .child(self.render_tab_button(ImageDetailTab::Terminal, "Terminal", cx))
                    .child(self.render_tab_button(ImageDetailTab::Files, "Files", cx)),
            )
            // Tab content
            .child(
                div()
                    .id("image-detail-content")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_4()
                    .child(if let Some(image) = selected {
                        self.render_detail_content(image).into_any_element()
                    } else {
                        self.render_no_selection().into_any_element()
                    }),
            )
    }

    fn render_tab_button(
        &self,
        tab: ImageDetailTab,
        label: &'static str,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.active_tab == tab;

        div()
            .id(SharedString::from(format!("image-tab-{:?}", tab)))
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

    fn render_detail_content(&self, image: &ImageViewModel) -> impl IntoElement {
        match self.active_tab {
            ImageDetailTab::Info => self.render_info_tab(image).into_any_element(),
            ImageDetailTab::Terminal => self.render_placeholder_tab("Terminal").into_any_element(),
            ImageDetailTab::Files => self.render_placeholder_tab("Files").into_any_element(),
        }
    }

    fn render_info_tab(&self, image: &ImageViewModel) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            // Basic info
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(Theme::info_row("ID", image.id.clone()))
                    .child(Theme::info_row(
                        "Tag",
                        format!("{}:{}", image.repository, image.tag),
                    ))
                    .child(Theme::info_row("Created", image.created_ago()))
                    .child(Theme::info_row("Size", image.size_display()))
                    .child(Theme::info_row(
                        "Platform",
                        format!("{}/{}", image.os, image.architecture),
                    )),
            )
            // Export button
            .child(
                div()
                    .mt_4()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(colors::border())
                    .flex()
                    .items_center()
                    .gap_3()
                    .cursor_pointer()
                    .hover(|el| el.bg(colors::hover()))
                    .child(
                        div()
                            .w(px(24.0))
                            .h(px(24.0))
                            .rounded_md()
                            .bg(colors::surface_elevated())
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(svg().path("icons/download.svg").size(px(14.0)).text_color(colors::text())),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(colors::text())
                            .child("Export"),
                    )
                    .child(
                        div()
                            .text_color(colors::text_muted())
                            .child("›"),
                    ),
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
