use gpui::*;

/// ArcBox theme color functions
/// Using functions instead of constants because gpui::rgb/rgba are not const fn
pub mod colors {
    use gpui::*;

    // Semantic colors
    pub fn running() -> Rgba {
        rgba(0x22c55eff)
    }
    pub fn stopped() -> Rgba {
        rgba(0x71717aff)
    }
    pub fn error() -> Rgba {
        rgba(0xef4444ff)
    }
    pub fn warning() -> Rgba {
        rgba(0xf59e0bff)
    }

    // Base colors
    pub fn background() -> Rgba {
        rgba(0x0a0a0aff)
    }
    pub fn surface() -> Rgba {
        rgba(0x171717ff)
    }
    pub fn surface_elevated() -> Rgba {
        rgba(0x1f1f1fff)
    }
    pub fn border() -> Rgba {
        rgba(0x27272aff)
    }
    pub fn border_focused() -> Rgba {
        rgba(0x3f3f46ff)
    }

    // Text colors
    pub fn text() -> Rgba {
        rgba(0xe4e4e7ff)
    }
    pub fn text_muted() -> Rgba {
        rgba(0xa1a1aaff)
    }
    pub fn text_subtle() -> Rgba {
        rgba(0x71717aff)
    }

    // Accent colors
    pub fn accent() -> Rgba {
        rgba(0x3b82f6ff)
    }
    pub fn accent_hover() -> Rgba {
        rgba(0x2563ebff)
    }
    pub fn on_accent() -> Rgba {
        rgba(0xffffffff)
    }

    // Interactive states
    pub fn hover() -> Rgba {
        rgba(0xffffff0a)
    }
    pub fn selection() -> Rgba {
        rgba(0x3b82f620)
    }
}

/// Theme-aware UI helpers
pub struct Theme;

impl Theme {
    /// Card background style
    pub fn card() -> Div {
        div()
            .bg(colors::surface())
            .border_1()
            .border_color(colors::border())
            .rounded_lg()
    }

    /// Primary button style
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
            .w(px(32.0))
            .h(px(32.0))
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
            .bg(colors::surface())
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

    /// Section header
    pub fn section_header() -> Div {
        div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(colors::border())
    }

    /// Page title
    pub fn page_title(title: impl Into<SharedString>) -> Div {
        div()
            .text_xl()
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
}

/// Initialize the theme (called in main)
pub fn init(_cx: &mut App) {
    // Future: Register custom fonts, theme preferences, etc.
}
