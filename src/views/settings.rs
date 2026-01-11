use gpui::*;
use gpui::prelude::*;

use crate::theme::colors;

/// Settings section in sidebar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    #[default]
    General,
    System,
    Network,
    Storage,
    Machines,
    Docker,
    Kubernetes,
}

impl SettingsSection {
    fn label(&self) -> &'static str {
        match self {
            SettingsSection::General => "General",
            SettingsSection::System => "System",
            SettingsSection::Network => "Network",
            SettingsSection::Storage => "Storage",
            SettingsSection::Machines => "Machines",
            SettingsSection::Docker => "Docker",
            SettingsSection::Kubernetes => "Kubernetes",
        }
    }

    fn icon_path(&self) -> &'static str {
        match self {
            SettingsSection::General => "icons/settings.svg",
            SettingsSection::System => "icons/system.svg",
            SettingsSection::Network => "icons/network.svg",
            SettingsSection::Storage => "icons/storage.svg",
            SettingsSection::Machines => "icons/machine.svg",
            SettingsSection::Docker => "icons/docker.svg",
            SettingsSection::Kubernetes => "icons/kubernetes.svg",
        }
    }

    fn all() -> &'static [SettingsSection] {
        &[
            SettingsSection::General,
            SettingsSection::System,
            SettingsSection::Network,
            SettingsSection::Storage,
            SettingsSection::Machines,
            SettingsSection::Docker,
            SettingsSection::Kubernetes,
        ]
    }
}

/// Update channel options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpdateChannel {
    #[default]
    Stable,
    Beta,
    Nightly,
}

impl UpdateChannel {
    fn label(&self) -> &'static str {
        match self {
            UpdateChannel::Stable => "Stable",
            UpdateChannel::Beta => "Beta",
            UpdateChannel::Nightly => "Nightly",
        }
    }

    fn all() -> &'static [UpdateChannel] {
        &[UpdateChannel::Stable, UpdateChannel::Beta, UpdateChannel::Nightly]
    }

    fn from_index(idx: usize) -> Self {
        Self::all().get(idx).copied().unwrap_or_default()
    }
}

/// Terminal theme options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TerminalTheme {
    #[default]
    System,
    Light,
    Dark,
}

impl TerminalTheme {
    fn label(&self) -> &'static str {
        match self {
            TerminalTheme::System => "System",
            TerminalTheme::Light => "Light",
            TerminalTheme::Dark => "Dark",
        }
    }

    fn all() -> &'static [TerminalTheme] {
        &[TerminalTheme::System, TerminalTheme::Light, TerminalTheme::Dark]
    }

    fn from_index(idx: usize) -> Self {
        Self::all().get(idx).copied().unwrap_or_default()
    }
}

/// External terminal app options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExternalTerminal {
    #[default]
    LastUsed,
    Terminal,
    ITerm2,
    Warp,
    Alacritty,
}

impl ExternalTerminal {
    fn label(&self) -> &'static str {
        match self {
            ExternalTerminal::LastUsed => "Last used",
            ExternalTerminal::Terminal => "Terminal",
            ExternalTerminal::ITerm2 => "iTerm2",
            ExternalTerminal::Warp => "Warp",
            ExternalTerminal::Alacritty => "Alacritty",
        }
    }

    fn all() -> &'static [ExternalTerminal] {
        &[
            ExternalTerminal::LastUsed,
            ExternalTerminal::Terminal,
            ExternalTerminal::ITerm2,
            ExternalTerminal::Warp,
            ExternalTerminal::Alacritty,
        ]
    }

    fn from_index(idx: usize) -> Self {
        Self::all().get(idx).copied().unwrap_or_default()
    }
}

/// Dropdown identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropdownId {
    UpdateChannel,
    TerminalTheme,
    ExternalTerminal,
}

/// Settings view
pub struct SettingsView {
    active_section: SettingsSection,
    // General settings
    start_at_login: bool,
    show_in_menu_bar: bool,
    keep_running_when_quit: bool,
    // Updates
    auto_download_updates: bool,
    update_channel: UpdateChannel,
    // Terminal
    terminal_theme: TerminalTheme,
    external_terminal: ExternalTerminal,
    // Dropdown state
    open_dropdown: Option<DropdownId>,
}

impl SettingsView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            active_section: SettingsSection::General,
            start_at_login: true,
            show_in_menu_bar: false,
            keep_running_when_quit: false,
            auto_download_updates: true,
            update_channel: UpdateChannel::Stable,
            terminal_theme: TerminalTheme::System,
            external_terminal: ExternalTerminal::LastUsed,
            open_dropdown: None,
        }
    }

    fn set_section(&mut self, section: SettingsSection, cx: &mut Context<Self>) {
        self.active_section = section;
        cx.notify();
    }

    fn toggle_dropdown(&mut self, id: DropdownId, cx: &mut Context<Self>) {
        if self.open_dropdown == Some(id) {
            self.open_dropdown = None;
        } else {
            self.open_dropdown = Some(id);
        }
        cx.notify();
    }

    fn select_dropdown_option(&mut self, dropdown: DropdownId, idx: usize, cx: &mut Context<Self>) {
        match dropdown {
            DropdownId::UpdateChannel => {
                self.update_channel = UpdateChannel::from_index(idx);
            }
            DropdownId::TerminalTheme => {
                self.terminal_theme = TerminalTheme::from_index(idx);
            }
            DropdownId::ExternalTerminal => {
                self.external_terminal = ExternalTerminal::from_index(idx);
            }
        }
        self.open_dropdown = None;
        cx.notify();
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_row()
            .overflow_hidden()
            .bg(colors::sidebar())
            // Section navigation sidebar
            .child(self.render_settings_sidebar(cx))
            // Vertical divider
            .child(
                div()
                    .w(px(1.0))
                    .h_full()
                    .bg(colors::border())
                    .flex_shrink_0(),
            )
            // Section content
            .child(self.render_settings_content(cx))
    }
}

impl SettingsView {
    fn render_settings_sidebar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .w(px(200.0))
            .h_full()
            .flex()
            .flex_col()
            .bg(colors::sidebar())
            .flex_shrink_0()
            // Titlebar area (for traffic lights on macOS)
            .child(div().h(px(52.0)))
            .children(SettingsSection::all().iter().map(|section| {
                self.render_section_nav(*section, cx)
            }))
    }

    fn render_section_nav(
        &self,
        section: SettingsSection,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.active_section == section;

        div()
            .id(SharedString::from(format!("settings-{:?}", section)))
            .mx_2()
            .px_3()
            .py_2()
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
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.set_section(section, cx);
            }))
            .child(
                svg()
                    .path(section.icon_path())
                    .size(px(18.0))
                    .flex_shrink_0()
                    .text_color(if is_active {
                        colors::on_accent()
                    } else {
                        colors::text_secondary()
                    }),
            )
            .child(section.label())
    }

    fn render_settings_content(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("settings-content")
            .flex_1()
            .h_full()
            .overflow_y_scroll()
            .bg(colors::background())
            // Titlebar area spacer
            .pt(px(52.0))
            .child(
                div()
                    .px_6()
                    .pb_6()
                    .pt_4()
                    .max_w(px(600.0))
                    .flex()
                    .flex_col()
                    .gap_6()
                    // Section title
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::text())
                            .child(self.active_section.label()),
                    )
                    // Section content
                    .child(match self.active_section {
                        SettingsSection::General => self.render_general(cx).into_any_element(),
                        SettingsSection::System => self.render_placeholder("System").into_any_element(),
                        SettingsSection::Network => self.render_placeholder("Network").into_any_element(),
                        SettingsSection::Storage => self.render_placeholder("Storage").into_any_element(),
                        SettingsSection::Machines => self.render_placeholder("Machines").into_any_element(),
                        SettingsSection::Docker => self.render_placeholder("Docker").into_any_element(),
                        SettingsSection::Kubernetes => self.render_placeholder("Kubernetes").into_any_element(),
                    }),
            )
    }

    fn render_general(&self, cx: &Context<Self>) -> impl IntoElement {
        let start_at_login = self.start_at_login;
        let show_in_menu_bar = self.show_in_menu_bar;
        let keep_running_when_quit = self.keep_running_when_quit;
        let auto_download_updates = self.auto_download_updates;
        let update_channel = self.update_channel;
        let terminal_theme = self.terminal_theme;
        let external_terminal = self.external_terminal;
        let open_dropdown = self.open_dropdown;

        div()
            .flex()
            .flex_col()
            .gap_6()
            // Startup settings group (no title)
            .child(
                Self::settings_group(None)
                    .child(Self::toggle_row(
                        "start-at-login",
                        "Start at login",
                        None,
                        start_at_login,
                        cx.listener(|this, _, _window, cx| {
                            this.start_at_login = !this.start_at_login;
                            cx.notify();
                        }),
                    ))
                    .child(Self::divider())
                    .child(Self::toggle_row(
                        "show-menu-bar",
                        "Show in menu bar",
                        None,
                        show_in_menu_bar,
                        cx.listener(|this, _, _window, cx| {
                            this.show_in_menu_bar = !this.show_in_menu_bar;
                            cx.notify();
                        }),
                    ))
                    .child(Self::divider())
                    .child(Self::toggle_row(
                        "keep-running",
                        "Keep running when app is quit",
                        None,
                        keep_running_when_quit,
                        cx.listener(|this, _, _window, cx| {
                            this.keep_running_when_quit = !this.keep_running_when_quit;
                            cx.notify();
                        }),
                    )),
            )
            // Updates group
            .child(
                Self::settings_group(Some("Updates"))
                    .child(Self::toggle_row(
                        "auto-updates",
                        "Automatically download updates",
                        None,
                        auto_download_updates,
                        cx.listener(|this, _, _window, cx| {
                            this.auto_download_updates = !this.auto_download_updates;
                            cx.notify();
                        }),
                    ))
                    .child(Self::divider())
                    .child(self.render_dropdown(
                        DropdownId::UpdateChannel,
                        "Update channel",
                        None,
                        update_channel.label(),
                        UpdateChannel::all().iter().map(|c| c.label()).collect(),
                        open_dropdown == Some(DropdownId::UpdateChannel),
                        cx,
                    )),
            )
            // Terminal group
            .child(
                Self::settings_group(Some("Terminal"))
                    .child(self.render_dropdown(
                        DropdownId::TerminalTheme,
                        "Terminal theme",
                        None,
                        terminal_theme.label(),
                        TerminalTheme::all().iter().map(|t| t.label()).collect(),
                        open_dropdown == Some(DropdownId::TerminalTheme),
                        cx,
                    ))
                    .child(Self::divider())
                    .child(self.render_dropdown(
                        DropdownId::ExternalTerminal,
                        "External terminal app",
                        Some("Used when opening terminal in a new window."),
                        external_terminal.label(),
                        ExternalTerminal::all().iter().map(|t| t.label()).collect(),
                        open_dropdown == Some(DropdownId::ExternalTerminal),
                        cx,
                    )),
            )
    }

    fn render_placeholder(&self, name: &'static str) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                Self::settings_group(None)
                    .p_6()
                    .child(
                        div()
                            .text_color(colors::text_secondary())
                            .child(format!("{} settings coming soon...", name)),
                    ),
            )
    }

    // ===== UI Components =====

    /// Settings group card with optional title
    fn settings_group(title: Option<&'static str>) -> Div {
        let card = div()
            .bg(colors::background())
            .border_1()
            .border_color(colors::border())
            .rounded_lg()
            .overflow_hidden();

        if let Some(title) = title {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::text())
                        .child(title),
                )
                .child(card)
        } else {
            card
        }
    }

    /// Horizontal divider line
    fn divider() -> Div {
        div()
            .h(px(1.0))
            .bg(colors::border())
            .mx_4()
    }

    /// Toggle switch row
    fn toggle_row(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        description: Option<&'static str>,
        value: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        let id: SharedString = id.into();
        let label: SharedString = label.into();

        div()
            .px_4()
            .py_3()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors::text())
                            .child(label),
                    )
                    .when_some(description, |el, desc| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(colors::text_secondary())
                                .child(desc),
                        )
                    }),
            )
            .child(Self::toggle_switch(id, value, on_click))
    }

    /// Toggle switch component
    fn toggle_switch(
        id: impl Into<SharedString>,
        value: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        let id: SharedString = id.into();

        div()
            .id(SharedString::from(format!("toggle-{}", id)))
            .w(px(44.0))
            .h(px(24.0))
            .rounded_full()
            .cursor_pointer()
            .flex_shrink_0()
            .when(value, |el| el.bg(colors::accent()))
            .when(!value, |el| el.bg(rgba(0x78716c40))) // Gray background when off
            .on_click(on_click)
            .child(
                div()
                    .w(px(20.0))
                    .h(px(20.0))
                    .mt(px(2.0))
                    .rounded_full()
                    .bg(rgb(0xffffff))
                    .shadow_sm()
                    .when(value, |el| el.ml(px(22.0)))
                    .when(!value, |el| el.ml(px(2.0))),
            )
    }

    /// Dropdown row with all options rendered inline
    fn render_dropdown(
        &self,
        dropdown_id: DropdownId,
        label: &'static str,
        description: Option<&'static str>,
        current_value: &'static str,
        options: Vec<&'static str>,
        is_open: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .relative()
            .px_4()
            .py_3()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors::text())
                            .child(label),
                    )
                    .when_some(description, |el, desc| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(colors::text_secondary())
                                .child(desc),
                        )
                    }),
            )
            .child(
                div()
                    .relative()
                    .child(self.render_dropdown_button(dropdown_id, current_value, cx))
                    .when(is_open, |el| {
                        el.child(self.render_dropdown_menu(dropdown_id, options, cx))
                    }),
            )
    }

    /// Dropdown button
    fn render_dropdown_button(
        &self,
        dropdown_id: DropdownId,
        current_value: &'static str,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(format!("dropdown-{:?}", dropdown_id)))
            .px_3()
            .py_1p5()
            .rounded_md()
            .border_1()
            .border_color(colors::border())
            .bg(colors::background())
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2()
            .hover(|el| el.bg(colors::surface()))
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.toggle_dropdown(dropdown_id, cx);
            }))
            .child(
                div()
                    .text_sm()
                    .text_color(colors::text())
                    .child(current_value),
            )
            .child(
                // Dropdown arrow
                div()
                    .text_xs()
                    .text_color(colors::text_secondary())
                    .child("▾"),
            )
    }

    /// Dropdown menu with options
    fn render_dropdown_menu(
        &self,
        dropdown_id: DropdownId,
        options: Vec<&'static str>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .absolute()
            .top(px(36.0))
            .right(px(0.0))
            .min_w(px(120.0))
            .bg(colors::background())
            .border_1()
            .border_color(colors::border())
            .rounded_md()
            .shadow_lg()
            .overflow_hidden()
            .children(options.into_iter().enumerate().map(|(idx, option)| {
                self.render_dropdown_option(dropdown_id, idx, option, cx)
            }))
    }

    /// Single dropdown option
    fn render_dropdown_option(
        &self,
        dropdown_id: DropdownId,
        idx: usize,
        label: &'static str,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(format!("dropdown-{:?}-option-{}", dropdown_id, idx)))
            .px_3()
            .py_2()
            .text_sm()
            .text_color(colors::text())
            .cursor_pointer()
            .hover(|el| el.bg(colors::hover()))
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.select_dropdown_option(dropdown_id, idx, cx);
            }))
            .child(label)
    }
}
