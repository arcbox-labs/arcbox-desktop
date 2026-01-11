use arcbox_api::generated::ListImagesResponse;
use gpui::*;
use gpui::prelude::*;
use gpui_component::tab::TabBar;
use gpui_component::Sizable;

use crate::models::{ImageViewModel, calculate_image_stats};
use crate::services::{ImageIconService, IconState};
use crate::theme::{colors, Theme, MONO_FONT};

/// Detail tab for images
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageDetailTab {
    #[default]
    Info,
    Terminal,
    Files,
}

impl ImageDetailTab {
    const ALL: [ImageDetailTab; 3] = [
        ImageDetailTab::Info,
        ImageDetailTab::Terminal,
        ImageDetailTab::Files,
    ];

    fn label(&self) -> &'static str {
        match self {
            ImageDetailTab::Info => "Info",
            ImageDetailTab::Terminal => "Terminal",
            ImageDetailTab::Files => "Files",
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

/// Images list view
pub struct ImagesView {
    images: Vec<ImageViewModel>,
    selected_id: Option<String>,
    active_tab: ImageDetailTab,
    list_width: f32,
    daemon_service: Entity<crate::services::DaemonService>,
    icon_service: Entity<ImageIconService>,
    is_loading: bool,
}

impl ImagesView {
    pub fn new(
        daemon_service: Entity<crate::services::DaemonService>,
        icon_service: Entity<ImageIconService>,
        cx: &mut Context<Self>,
    ) -> Self {
        // Subscribe to icon service updates for re-rendering
        cx.observe(&icon_service, |_, _, cx| cx.notify()).detach();

        // Subscribe to daemon service connection state changes
        cx.observe(&daemon_service, |this, daemon, cx| {
            if daemon.read(cx).is_connected() && this.is_loading {
                // Request image list when connected
                daemon.update(cx, |svc, cx| {
                    svc.list_images(cx);
                });
            }
            cx.notify();
        })
        .detach();

        Self {
            images: Vec::new(),
            selected_id: None,
            active_tab: ImageDetailTab::Info,
            list_width: LIST_DEFAULT_WIDTH,
            daemon_service,
            icon_service,
            is_loading: true,
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

    /// Handle images loaded from daemon
    pub fn on_images_loaded(&mut self, response: ListImagesResponse, cx: &mut Context<Self>) {
        self.is_loading = false;
        self.images = response
            .images
            .into_iter()
            .map(ImageViewModel::from)
            .collect();

        // Request icons for all images
        for image in &self.images {
            self.icon_service.update(cx, |svc, cx| {
                let _ = svc.get_icon(&image.repository, cx);
            });
        }

        // Select first image if none selected
        if self.selected_id.is_none() {
            if let Some(first) = self.images.first() {
                self.selected_id = Some(first.id.clone());
            }
        }

        cx.notify();
    }
}

impl Render for ImagesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                                            .text_color(colors::text_secondary())
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
                                            .child(svg().path("icons/sort.svg").size(px(16.0)).text_color(colors::text_secondary()))
                                    )
                                    .child(
                                        Theme::button_icon()
                                            .id("search-images")
                                            .child(svg().path("icons/search.svg").size(px(16.0)).text_color(colors::text_secondary()))
                                    )
                                    .child(
                                        Theme::button_icon()
                                            .id("add-image")
                                            .child(svg().path("icons/add.svg").size(px(16.0)).text_color(colors::text_secondary()))
                                    ),
                            ),
                    )
                    // "In Use" section header
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .text_xs()
                            .text_color(colors::text_secondary())
                            .child("In Use"),
                    )
                    // Image list
                    .child(
                        div()
                            .id("images-list")
                            .flex_1()
                            .overflow_y_scroll()
                            .when(self.images.is_empty(), |el| {
                                el.child(self.render_empty_state())
                            })
                            .when(!self.images.is_empty(), |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .children(
                                            self.images
                                                .iter()
                                                .map(|image| self.render_image_row(image, cx)),
                                        ),
                                )
                            }),
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
                            .when(!is_selected, |el| el.text_color(colors::text_secondary()))
                            .child(format!(
                                "{}, {}",
                                image.size_display(),
                                image.created_ago()
                            )),
                    ),
            )
            // Delete button
            .child({
                let icon_color = if is_selected { colors::on_accent() } else { colors::text_secondary() };
                Theme::button_icon()
                    .w(px(24.0))
                    .h(px(24.0))
                    .child(svg().path("icons/delete.svg").size(px(16.0)).text_color(icon_color))
            })
    }

    fn render_image_icon(
        &self,
        repository: &str,
        icon_state: Option<IconState>,
        _is_selected: bool,
    ) -> impl IntoElement {
        match icon_state {
            Some(IconState::Found(path)) => {
                // Display fetched icon from local cache file
                img(path)
                    .w(px(28.0))
                    .h(px(28.0))
                    .rounded(px(4.0))
                    .into_any_element()
            }
            // For Loading/NotFound/Error/None, always show colored box icon
            // This prevents visual "flash" when loading
            _ => {
                let color = Self::get_color_for_repository(repository);
                svg()
                    .path("icons/box.svg")
                    .size(px(18.0))
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
                        TabBar::new("image-detail-tabs")
                            .segmented()
                            .children(ImageDetailTab::ALL.iter().map(|tab| tab.label()))
                            .selected_index(selected_index)
                            .on_click(cx.listener(|this, index: &usize, _window, cx| {
                                this.set_tab(ImageDetailTab::from_index(*index), cx);
                            })),
                    ),
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
                    .child("No images yet"),
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
                            .child("Pull an image:"),
                    )
                    .child(Self::render_command_hint(
                        "docker pull nginx",
                        "Official nginx image",
                    ))
                    .child(Self::render_command_hint(
                        "docker pull postgres:16",
                        "PostgreSQL database",
                    ))
                    .child(Self::render_command_hint(
                        "docker pull redis:alpine",
                        "Redis with Alpine Linux",
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
                            .child(svg().path("icons/download.svg").size(px(16.0)).text_color(colors::text())),
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
                            .text_color(colors::text_secondary())
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
            .text_color(colors::text_secondary())
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
