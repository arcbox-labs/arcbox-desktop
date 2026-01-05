use gpui::*;
use gpui::prelude::*;

use crate::theme::{colors, Theme};

/// Settings section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    #[default]
    General,
    Resources,
    About,
}

impl SettingsSection {
    fn label(&self) -> &'static str {
        match self {
            SettingsSection::General => "General",
            SettingsSection::Resources => "Resources",
            SettingsSection::About => "About",
        }
    }

    fn all() -> &'static [SettingsSection] {
        &[
            SettingsSection::General,
            SettingsSection::Resources,
            SettingsSection::About,
        ]
    }
}

/// Settings view
pub struct SettingsView {
    active_section: SettingsSection,
    start_at_login: bool,
    auto_update: bool,
}

impl SettingsView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            active_section: SettingsSection::General,
            start_at_login: true,
            auto_update: true,
        }
    }

    fn set_section(&mut self, section: SettingsSection, cx: &mut Context<Self>) {
        self.active_section = section;
        cx.notify();
    }

    fn toggle_start_at_login(&mut self, cx: &mut Context<Self>) {
        self.start_at_login = !self.start_at_login;
        cx.notify();
    }

    fn toggle_auto_update(&mut self, cx: &mut Context<Self>) {
        self.auto_update = !self.auto_update;
        cx.notify();
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(colors::background())
            // Header
            .child(
                Theme::section_header()
                    .child(Theme::page_title("Settings"))
                    .child(div()),
            )
            // Content
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    // Section navigation
                    .child(
                        div()
                            .w(px(200.0))
                            .border_r_1()
                            .border_color(colors::border())
                            .p_2()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .children(SettingsSection::all().iter().map(|section| {
                                self.render_section_nav(*section, cx)
                            })),
                    )
                    // Section content
                    .child(
                        div()
                            .id("settings-content")
                            .flex_1()
                            .overflow_y_scroll()
                            .p_6()
                            .child(match self.active_section {
                                SettingsSection::General => self.render_general(cx).into_any_element(),
                                SettingsSection::Resources => self.render_resources().into_any_element(),
                                SettingsSection::About => self.render_about().into_any_element(),
                            }),
                    ),
            )
    }
}

impl SettingsView {
    fn render_section_nav(
        &self,
        section: SettingsSection,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.active_section == section;

        div()
            .id(SharedString::from(format!("settings-{:?}", section)))
            .px_3()
            .py_2()
            .rounded_md()
            .text_sm()
            .cursor_pointer()
            .when(is_active, |el| {
                el.bg(colors::selection()).text_color(colors::text())
            })
            .when(!is_active, |el| {
                el.text_color(colors::text_muted())
                    .hover(|el| el.bg(colors::hover()).text_color(colors::text()))
            })
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.set_section(section, cx);
            }))
            .child(section.label())
    }

    fn render_general(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .text_lg()
                    .text_color(colors::text())
                    .child("General"),
            )
            // Start at login
            .child(self.setting_toggle(
                "start-at-login",
                "Start at login",
                "Automatically start ArcBox when you log in",
                self.start_at_login,
                cx.listener(|this, _, _window, cx| {
                    this.toggle_start_at_login(cx);
                }),
            ))
            // Auto update
            .child(self.setting_toggle(
                "auto-update",
                "Automatic updates",
                "Automatically download and install updates",
                self.auto_update,
                cx.listener(|this, _, _window, cx| {
                    this.toggle_auto_update(cx);
                }),
            ))
    }

    fn render_resources(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .text_lg()
                    .text_color(colors::text())
                    .child("Resources"),
            )
            .child(
                Theme::card()
                    .p_4()
                    .child(
                        div()
                            .text_color(colors::text_muted())
                            .child("Resource settings coming soon..."),
                    ),
            )
    }

    fn render_about(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .items_center()
            .pt_8()
            // Logo placeholder
            .child(
                div()
                    .w(px(80.0))
                    .h(px(80.0))
                    .rounded_2xl()
                    .bg(colors::accent())
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_2xl()
                    .child("📦"),
            )
            // App name
            .child(
                div()
                    .text_2xl()
                    .text_color(colors::text())
                    .child("ArcBox"),
            )
            // Version
            .child(
                div()
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child("Version 0.1.0 (Development)"),
            )
            // Description
            .child(
                div()
                    .max_w(px(400.0))
                    .text_center()
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child("High-performance container and VM runtime for macOS and Linux"),
            )
    }

    fn setting_toggle(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        description: impl Into<SharedString>,
        value: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        let id: SharedString = id.into();
        let label: SharedString = label.into();
        let description: SharedString = description.into();

        Theme::card()
            .p_4()
            .flex()
            .items_center()
            .gap_4()
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_color(colors::text()).child(label))
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors::text_muted())
                            .child(description),
                    ),
            )
            .child(
                // Toggle switch
                div()
                    .id(SharedString::from(format!("toggle-{}", id)))
                    .w(px(44.0))
                    .h(px(24.0))
                    .rounded_full()
                    .cursor_pointer()
                    .when(value, |el| el.bg(colors::accent()))
                    .when(!value, |el| el.bg(colors::surface_elevated()))
                    .on_click(on_click)
                    .child(
                        div()
                            .w(px(20.0))
                            .h(px(20.0))
                            .mt(px(2.0))
                            .rounded_full()
                            .bg(rgb(0xffffff))
                            .when(value, |el| el.ml(px(22.0)))
                            .when(!value, |el| el.ml(px(2.0))),
                    ),
            )
    }
}
