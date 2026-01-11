use gpui::*;

/// Font family for monospace text (Berkeley Mono)
pub const MONO_FONT: &str = "Berkeley Mono Variable WY0QJ1YX";

/// ArcBox theme colors - Light theme with macOS native feel
pub mod colors {
    use gpui::*;

    // Status colors - macOS style
    pub fn running() -> Rgba {
        rgba(0x34c759ff) // macOS green
    }
    pub fn stopped() -> Rgba {
        rgba(0x8e8e93ff) // macOS gray (not red for stopped)
    }
    pub fn error() -> Rgba {
        rgba(0xff3b30ff) // macOS red
    }
    pub fn warning() -> Rgba {
        rgba(0xff9500ff) // macOS orange
    }

    // Base colors - Light theme with macOS feel
    pub fn background() -> Rgba {
        rgba(0xffffffff) // White content background
    }
    pub fn sidebar() -> Rgba {
        rgba(0xf6f6f6ff) // macOS sidebar gray (slightly warmer)
    }
    pub fn surface() -> Rgba {
        rgba(0xfafafaff) // Slightly off-white
    }
    pub fn surface_elevated() -> Rgba {
        rgba(0xf2f2f7ff) // macOS secondary system background
    }
    pub fn border() -> Rgba {
        rgba(0xe5e5eaff) // macOS separator color (subtle)
    }
    pub fn border_subtle() -> Rgba {
        rgba(0x00000010) // Very subtle border
    }
    pub fn border_focused() -> Rgba {
        rgba(0x007affff) // macOS blue for focus
    }

    // Text colors - macOS label colors
    pub fn text() -> Rgba {
        rgba(0x000000ff) // macOS label color (primary)
    }
    pub fn text_secondary() -> Rgba {
        rgba(0x3c3c4399) // macOS secondary label (60% opacity)
    }
    pub fn text_muted() -> Rgba {
        rgba(0x3c3c434d) // macOS tertiary label (30% opacity)
    }
    pub fn text_subtle() -> Rgba {
        rgba(0x3c3c432e) // macOS quaternary label (18% opacity)
    }

    // Accent colors - macOS system blue
    pub fn accent() -> Rgba {
        rgba(0x007affff) // macOS system blue
    }
    pub fn accent_hover() -> Rgba {
        rgba(0x0066d6ff) // Darker blue
    }
    pub fn on_accent() -> Rgba {
        rgba(0xffffffff) // White text on accent
    }

    // Interactive states - macOS style
    pub fn hover() -> Rgba {
        rgba(0x00000008) // Very subtle hover (5% black)
    }
    pub fn selection() -> Rgba {
        rgba(0x007affff) // macOS blue selection
    }
    pub fn selection_inactive() -> Rgba {
        rgba(0xd1d1d6ff) // macOS unemphasized selected content
    }

    // Section header text - macOS style uppercase headers
    pub fn section_header() -> Rgba {
        rgba(0x6e6e73ff) // macOS secondary label for headers
    }

    // Sidebar specific - macOS uses subtle gray, not colored
    pub fn sidebar_item_hover() -> Rgba {
        rgba(0x00000008) // Subtle hover for sidebar items
    }
    pub fn sidebar_item_selected() -> Rgba {
        rgba(0x00000010) // Subtle gray tint for selected sidebar item
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
            .rounded(px(10.0)) // macOS uses 10px radius for cards
    }

    /// Primary button style (purple)
    pub fn button_primary() -> Div {
        div()
            .px_3()
            .py_1p5()
            .rounded(px(6.0)) // macOS button radius
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
            .rounded(px(6.0))
            .text_color(colors::text_secondary())
            .text_sm()
            .cursor_pointer()
            .hover(|el| el.bg(colors::hover()).text_color(colors::text()))
    }

    /// Icon button (square, small) - macOS style
    pub fn button_icon() -> Div {
        div()
            .w(px(26.0))
            .h(px(26.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(5.0)) // Slightly rounded
            .cursor_pointer()
            .hover(|el| el.bg(colors::hover()))
    }

    /// Input field style - macOS style
    pub fn input() -> Div {
        div()
            .px_2()
            .py_1p5()
            .rounded(px(6.0))
            .bg(colors::background())
            .border_1()
            .border_color(colors::border())
            .text_color(colors::text())
            .text_sm()
    }

    /// Badge/tag style - macOS capsule style
    pub fn badge() -> Div {
        div()
            .px_2()
            .py_0p5()
            .rounded(px(4.0))
            .bg(colors::surface_elevated())
            .text_color(colors::text_secondary())
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

    /// Page title - macOS style
    pub fn page_title(title: impl Into<SharedString>) -> Div {
        div()
            .text_sm()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(colors::text())
            .child(title.into())
    }

    /// Subtitle / secondary text
    pub fn subtitle(text: impl Into<SharedString>) -> Div {
        div()
            .text_xs()
            .text_color(colors::text_secondary())
            .child(text.into())
    }

    /// Sidebar section header (e.g., "DOCKER", "LINUX") - macOS uppercase style
    pub fn sidebar_section_header(title: &'static str) -> Div {
        div()
            .px_4()
            .pt_4()
            .pb_1()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(colors::section_header())
            .child(title.to_uppercase())
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

    /// Selected list row - macOS style
    pub fn list_row_selected() -> Div {
        div()
            .flex()
            .items_center()
            .px_4()
            .py_2()
            .cursor_pointer()
            .rounded(px(6.0))
            .bg(colors::selection())
            .text_color(colors::on_accent())
    }

    /// Detail panel container
    pub fn detail_panel() -> Div {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(colors::background())
    }

    /// Tab button (for detail panel tabs) - not used anymore, using TabBar instead
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
            base.text_color(colors::text_secondary())
                .hover(|el| el.text_color(colors::text()))
        }
    }

    /// Info row in detail panel - macOS style with subtle separator
    pub fn info_row(label: &'static str, value: impl Into<SharedString>) -> Div {
        let value: SharedString = value.into();
        div()
            .flex()
            .items_center()
            .py_2()
            .border_b_1()
            .border_color(colors::border_subtle())
            .child(
                div()
                    .w(px(100.0))
                    .flex_shrink_0()
                    .text_sm()
                    .text_color(colors::text_secondary())
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
pub fn init(cx: &mut App) {
    // Load Berkeley Mono font from assets
    let font_path = if cfg!(debug_assertions) {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/fonts/berkeley-mono-variable.otf")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("assets/fonts/berkeley-mono-variable.otf"))
            .unwrap_or_default()
    };

    if let Ok(font_data) = std::fs::read(&font_path) {
        if let Err(e) = cx.text_system().add_fonts(vec![std::borrow::Cow::Owned(font_data)]) {
            tracing::warn!("Failed to load Berkeley Mono font: {}", e);
        } else {
            tracing::debug!("Loaded Berkeley Mono font from {:?}", font_path);
        }
    } else {
        tracing::warn!("Berkeley Mono font not found at {:?}", font_path);
    }
}
