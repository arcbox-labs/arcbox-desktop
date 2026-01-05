use gpui::*;

/// ArcBox theme colors - Light theme matching OrbStack style
pub mod colors {
    use gpui::*;

    // Status colors
    pub fn running() -> Rgba {
        rgba(0x22c55eff) // Green
    }
    pub fn stopped() -> Rgba {
        rgba(0xef4444ff) // Red
    }
    pub fn error() -> Rgba {
        rgba(0xef4444ff)
    }
    pub fn warning() -> Rgba {
        rgba(0xf59e0bff)
    }

    // Base colors - Light theme
    pub fn background() -> Rgba {
        rgba(0xffffffff) // White content background
    }
    pub fn sidebar() -> Rgba {
        rgba(0xf5f5f4ff) // Light gray sidebar (stone-100)
    }
    pub fn surface() -> Rgba {
        rgba(0xfafafaff) // Slightly off-white
    }
    pub fn surface_elevated() -> Rgba {
        rgba(0xf5f5f5ff)
    }
    pub fn border() -> Rgba {
        rgba(0xe5e5e5ff) // Light border
    }
    pub fn border_focused() -> Rgba {
        rgba(0xd4d4d4ff)
    }

    // Text colors - Dark text for light theme
    pub fn text() -> Rgba {
        rgba(0x171717ff) // Near black
    }
    pub fn text_muted() -> Rgba {
        rgba(0x737373ff) // Gray
    }
    pub fn text_subtle() -> Rgba {
        rgba(0xa3a3a3ff) // Lighter gray
    }

    // Accent colors - Purple like OrbStack
    pub fn accent() -> Rgba {
        rgba(0x7c3aedff) // Purple (violet-600)
    }
    pub fn accent_hover() -> Rgba {
        rgba(0x6d28d9ff) // Darker purple
    }
    pub fn on_accent() -> Rgba {
        rgba(0xffffffff) // White text on accent
    }

    // Interactive states
    pub fn hover() -> Rgba {
        rgba(0x00000008) // Very subtle hover
    }
    pub fn selection() -> Rgba {
        rgba(0x7c3aedff) // Purple selection (same as accent)
    }
    pub fn selection_muted() -> Rgba {
        rgba(0x7c3aed20) // Light purple for secondary selection
    }

    // Section header text
    pub fn section_header() -> Rgba {
        rgba(0x78716cff) // Stone-500 for section headers
    }
}

/// Theme-aware UI helpers
pub struct Theme;

impl Theme {
    /// Card background style (for detail panels)
    pub fn card() -> Div {
        div()
            .bg(colors::background())
            .border_1()
            .border_color(colors::border())
            .rounded_lg()
    }

    /// Primary button style (purple)
    pub fn button_primary() -> Div {
        div()
            .px_3()
            .py_1p5()
            .rounded_md()
            .bg(colors::accent())
            .text_color(colors::on_accent())
            .text_sm()
            .cursor_pointer()
            .hover(|el| el.bg(colors::accent_hover()))
    }

    /// Secondary/ghost button style
    pub fn button_ghost() -> Div {
        div()
            .px_3()
            .py_1p5()
            .rounded_md()
            .text_color(colors::text_muted())
            .text_sm()
            .cursor_pointer()
            .hover(|el| el.bg(colors::hover()).text_color(colors::text()))
    }

    /// Icon button (square, small)
    pub fn button_icon() -> Div {
        div()
            .w(px(28.0))
            .h(px(28.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .text_color(colors::text_muted())
            .cursor_pointer()
            .hover(|el| el.bg(colors::hover()).text_color(colors::text()))
    }

    /// Input field style
    pub fn input() -> Div {
        div()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(colors::background())
            .border_1()
            .border_color(colors::border())
            .text_color(colors::text())
            .text_sm()
    }

    /// Badge/tag style
    pub fn badge() -> Div {
        div()
            .px_2()
            .py_0p5()
            .rounded_md()
            .bg(colors::surface_elevated())
            .text_color(colors::text_muted())
            .text_xs()
    }

    /// Section header in content area
    pub fn section_header() -> Div {
        div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(colors::border())
            .bg(colors::background())
    }

    /// Page title
    pub fn page_title(title: impl Into<SharedString>) -> Div {
        div()
            .text_base()
            .font_weight(FontWeight::MEDIUM)
            .text_color(colors::text())
            .child(title.into())
    }

    /// Subtitle / secondary text
    pub fn subtitle(text: impl Into<SharedString>) -> Div {
        div()
            .text_sm()
            .text_color(colors::text_muted())
            .child(text.into())
    }

    /// Sidebar section header (e.g., "Docker", "Linux")
    pub fn sidebar_section_header(title: &'static str) -> Div {
        div()
            .px_3()
            .py_1()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(colors::section_header())
            .child(title)
    }

    /// List row (flat, no card)
    pub fn list_row() -> Div {
        div()
            .flex()
            .items_center()
            .px_4()
            .py_2()
            .cursor_pointer()
            .hover(|el| el.bg(colors::hover()))
    }

    /// Selected list row
    pub fn list_row_selected() -> Div {
        div()
            .flex()
            .items_center()
            .px_4()
            .py_2()
            .cursor_pointer()
            .bg(colors::selection())
            .text_color(colors::on_accent())
    }

    /// Detail panel container
    pub fn detail_panel() -> Div {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(colors::border())
            .bg(colors::background())
    }

    /// Tab button (for detail panel tabs)
    pub fn tab_button(active: bool) -> Div {
        let base = div()
            .px_4()
            .py_2()
            .text_sm()
            .cursor_pointer();

        if active {
            base.bg(colors::background())
                .text_color(colors::text())
                .border_b_2()
                .border_color(colors::accent())
        } else {
            base.text_color(colors::text_muted())
                .hover(|el| el.text_color(colors::text()))
        }
    }

    /// Info row in detail panel
    pub fn info_row(label: &'static str, value: impl Into<SharedString>) -> Div {
        let value: SharedString = value.into();
        div()
            .flex()
            .items_center()
            .py_2()
            .border_b_1()
            .border_color(colors::border())
            .child(
                div()
                    .w(px(120.0))
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(label),
            )
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(colors::text())
                    .child(value),
            )
    }
}

/// Initialize the theme (called in main)
pub fn init(_cx: &mut App) {
    // Future: Register custom fonts, theme preferences, etc.
}
