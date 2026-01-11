use gpui::*;
use gpui::prelude::*;
use gpui_component::input::{Input, InputState, InputEvent};

use crate::theme::colors;
use crate::services::DaemonService;

/// Dismiss event for modal dialogs
pub struct DismissEvent;

/// Restart policy options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RestartPolicy {
    #[default]
    No,
    Always,
    OnFailure,
    UnlessStopped,
}

impl RestartPolicy {
    fn label(&self) -> &'static str {
        match self {
            RestartPolicy::No => "no",
            RestartPolicy::Always => "always",
            RestartPolicy::OnFailure => "on-failure",
            RestartPolicy::UnlessStopped => "unless-stopped",
        }
    }

    fn all() -> &'static [RestartPolicy] {
        &[
            RestartPolicy::No,
            RestartPolicy::Always,
            RestartPolicy::OnFailure,
            RestartPolicy::UnlessStopped,
        ]
    }

    fn from_index(idx: usize) -> Self {
        Self::all().get(idx).copied().unwrap_or_default()
    }
}

/// Platform options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Platform {
    #[default]
    Auto,
    LinuxAmd64,
    LinuxArm64,
}

impl Platform {
    fn label(&self) -> &'static str {
        match self {
            Platform::Auto => "auto",
            Platform::LinuxAmd64 => "linux/amd64",
            Platform::LinuxArm64 => "linux/arm64",
        }
    }

    fn all() -> &'static [Platform] {
        &[Platform::Auto, Platform::LinuxAmd64, Platform::LinuxArm64]
    }

    fn from_index(idx: usize) -> Self {
        Self::all().get(idx).copied().unwrap_or_default()
    }
}

/// Dropdown identifiers for the dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogDropdownId {
    Platform,
    RestartPolicy,
}

/// New container dialog form state
pub struct NewContainerDialog {
    // Focus management
    focus_handle: FocusHandle,
    // Basic settings - text inputs using gpui-component
    image_input: Entity<InputState>,
    platform: Platform,
    name_input: Entity<InputState>,
    remove_after_stop: bool,
    restart_policy: RestartPolicy,
    // Payload settings - text inputs using gpui-component
    command_input: Entity<InputState>,
    entrypoint_input: Entity<InputState>,
    workdir_input: Entity<InputState>,
    // Advanced settings
    privileged: bool,
    read_only: bool,
    use_docker_init: bool,
    // UI state
    open_dropdown: Option<DialogDropdownId>,
    // Services
    daemon_service: Entity<DaemonService>,
    // Callbacks
    on_close: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

/// Events emitted by the dialog
#[derive(Debug, Clone)]
pub enum NewContainerDialogEvent {
    Close,
    Created(String),
    CreatedAndStarted(String),
}

impl EventEmitter<NewContainerDialogEvent> for NewContainerDialog {}
impl EventEmitter<DismissEvent> for NewContainerDialog {}

impl Focusable for NewContainerDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl NewContainerDialog {
    pub fn new(daemon_service: Entity<DaemonService>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Create text input entities using gpui-component's InputState
        let image_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("e.g. alpine:latest")
        });

        let name_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("default")
        });

        let command_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("default")
        });

        let entrypoint_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("default")
        });

        let workdir_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("default")
        });

        Self {
            focus_handle: cx.focus_handle(),
            image_input,
            platform: Platform::Auto,
            name_input,
            remove_after_stop: false,
            restart_policy: RestartPolicy::No,
            command_input,
            entrypoint_input,
            workdir_input,
            privileged: false,
            read_only: false,
            use_docker_init: false,
            open_dropdown: None,
            daemon_service,
            on_close: None,
        }
    }

    pub fn set_on_close(&mut self, callback: impl Fn(&mut Window, &mut App) + 'static) {
        self.on_close = Some(Box::new(callback));
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(callback) = self.on_close.take() {
            callback(window, cx);
        }
        cx.emit(NewContainerDialogEvent::Close);
        cx.emit(DismissEvent);
    }

    fn create_container(&mut self, start: bool, window: &mut Window, cx: &mut Context<Self>) {
        // Read values from InputState entities
        let image = self.image_input.read(cx).value().to_string();
        let name_value = self.name_input.read(cx).value().to_string();
        let command_value = self.command_input.read(cx).value().to_string();
        let entrypoint_value = self.entrypoint_input.read(cx).value().to_string();
        let workdir_value = self.workdir_input.read(cx).value().to_string();

        if image.is_empty() {
            // TODO: Show validation error
            return;
        }

        let name = if name_value.is_empty() {
            None
        } else {
            Some(name_value)
        };
        let cmd = if command_value.is_empty() {
            None
        } else {
            // Split command by whitespace for now
            Some(command_value.split_whitespace().map(String::from).collect())
        };
        let entrypoint = if entrypoint_value.is_empty() {
            None
        } else {
            Some(entrypoint_value.split_whitespace().map(String::from).collect())
        };
        let working_dir = if workdir_value.is_empty() {
            None
        } else {
            Some(workdir_value)
        };

        tracing::info!(
            "Creating container: image={}, name={:?}, start={}",
            image,
            name,
            start
        );

        // Call daemon service to create container
        let image_for_event = image.clone();
        self.daemon_service.update(cx, |svc, cx| {
            svc.create_container(image, name, cmd, entrypoint, working_dir, start, cx);
        });

        if start {
            cx.emit(NewContainerDialogEvent::CreatedAndStarted(image_for_event));
        } else {
            cx.emit(NewContainerDialogEvent::Created(image_for_event));
        }

        self.close(window, cx);
    }

    fn toggle_dropdown(&mut self, id: DialogDropdownId, cx: &mut Context<Self>) {
        if self.open_dropdown == Some(id) {
            self.open_dropdown = None;
        } else {
            self.open_dropdown = Some(id);
        }
        cx.notify();
    }

    fn select_platform(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.platform = Platform::from_index(idx);
        self.open_dropdown = None;
        cx.notify();
    }

    fn select_restart_policy(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.restart_policy = RestartPolicy::from_index(idx);
        self.open_dropdown = None;
        cx.notify();
    }
}

impl Render for NewContainerDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Modal overlay - use a layered structure to handle click-outside-to-close
        div()
            .id("new-container-dialog-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            // Background layer (click to close)
            .child(
                div()
                    .id("dialog-backdrop")
                    .absolute()
                    .inset_0()
                    .bg(rgba(0x00000066))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                        this.close(window, cx);
                    })),
            )
            // Dialog layer (on top of backdrop) - with focus management
            .child(self.render_dialog(cx))
    }
}

impl NewContainerDialog {
    fn render_dialog(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let open_dropdown = self.open_dropdown;

        div()
            .id("new-container-dialog")
            .track_focus(&self.focus_handle)
            .relative()
            .w(px(480.0))
            .max_h(px(600.0))
            .bg(colors::background())
            .border_1()
            .border_color(colors::border())
            .rounded_xl()
            .shadow_xl()
            .flex()
            .flex_col()
            // Block mouse events from reaching the backdrop
            .occlude()
            // Header
            .child(
                div()
                    .px_5()
                    .py_4()
                    .border_b_1()
                    .border_color(colors::border())
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::text())
                            .child("New Container"),
                    ),
            )
            // Scrollable content
            .child(
                div()
                    .id("dialog-content")
                    .flex_1()
                    .overflow_y_scroll()
                    .px_5()
                    .py_4()
                    .flex()
                    .flex_col()
                    .gap_1()
                    // Basic settings
                    .child(self.render_input_field("Image", self.image_input.clone()))
                    .child(self.render_dropdown_field(
                        "Platform",
                        DialogDropdownId::Platform,
                        self.platform.label(),
                        Platform::all().iter().map(|p| p.label()).collect(),
                        open_dropdown == Some(DialogDropdownId::Platform),
                        cx,
                    ))
                    .child(self.render_input_field("Name", self.name_input.clone()))
                    .child(self.render_toggle_field(
                        "remove-after-stop",
                        "Remove after stop",
                        Some("Automatically delete the container after it stops. (--rm)"),
                        self.remove_after_stop,
                        cx.listener(|this, _, _window, cx| {
                            this.remove_after_stop = !this.remove_after_stop;
                            cx.notify();
                        }),
                    ))
                    .child(self.render_dropdown_field(
                        "Restart policy",
                        DialogDropdownId::RestartPolicy,
                        self.restart_policy.label(),
                        RestartPolicy::all().iter().map(|r| r.label()).collect(),
                        open_dropdown == Some(DialogDropdownId::RestartPolicy),
                        cx,
                    ))
                    // Payload section
                    .child(self.render_section_header("Payload"))
                    .child(self.render_input_field_with_desc(
                        "Command",
                        self.command_input.clone(),
                        "Command to run in the container",
                    ))
                    .child(self.render_input_field_with_desc(
                        "Entrypoint",
                        self.entrypoint_input.clone(),
                        "If set, command will be passed to the entrypoint instead of shell. (--entrypoint)",
                    ))
                    .child(self.render_input_field_with_desc(
                        "Working directory",
                        self.workdir_input.clone(),
                        "Working directory for the command. (--workdir)",
                    ))
                    // Advanced section
                    .child(self.render_section_header("Advanced"))
                    .child(self.render_toggle_field(
                        "privileged",
                        "Privileged",
                        Some("Allow access to privileged APIs and resources. (--privileged)"),
                        self.privileged,
                        cx.listener(|this, _, _window, cx| {
                            this.privileged = !this.privileged;
                            cx.notify();
                        }),
                    ))
                    .child(self.render_toggle_field(
                        "read-only",
                        "Read-only",
                        Some("Mount the container's root filesystem as read-only. (--read-only)"),
                        self.read_only,
                        cx.listener(|this, _, _window, cx| {
                            this.read_only = !this.read_only;
                            cx.notify();
                        }),
                    ))
                    .child(self.render_toggle_field(
                        "docker-init",
                        "Use docker-init",
                        Some("Run the container payload under a docker-init process. (--init)"),
                        self.use_docker_init,
                        cx.listener(|this, _, _window, cx| {
                            this.use_docker_init = !this.use_docker_init;
                            cx.notify();
                        }),
                    )),
            )
            // Footer
            .child(self.render_footer(cx))
            // Dropdown overlay (rendered on top of everything)
            .child(self.render_dropdown_overlay(cx))
    }

    fn render_section_header(&self, title: &'static str) -> impl IntoElement {
        div()
            .mt_4()
            .mb_1()
            .text_sm()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(colors::text())
            .child(title)
    }

    /// Render a text input field with a label using gpui-component's Input
    fn render_input_field(
        &self,
        label: &'static str,
        input: Entity<InputState>,
    ) -> impl IntoElement {
        div()
            .py_2()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(colors::border())
            .child(
                div()
                    .text_sm()
                    .text_color(colors::text())
                    .child(label),
            )
            .child(
                div()
                    .w(px(200.0))
                    .child(Input::new(&input)),
            )
    }

    /// Render a text input field with a label and description using gpui-component's Input
    fn render_input_field_with_desc(
        &self,
        label: &'static str,
        input: Entity<InputState>,
        description: &'static str,
    ) -> impl IntoElement {
        div()
            .py_2()
            .flex()
            .items_start()
            .justify_between()
            .border_b_1()
            .border_color(colors::border())
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
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::text_muted())
                            .max_w(px(220.0))
                            .child(description),
                    ),
            )
            .child(
                div()
                    .w(px(200.0))
                    .child(Input::new(&input)),
            )
    }

    fn render_toggle_field(
        &self,
        id: &'static str,
        label: &'static str,
        description: Option<&'static str>,
        value: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        div()
            .py_2()
            .flex()
            .items_start()
            .justify_between()
            .border_b_1()
            .border_color(colors::border())
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
                                .text_color(colors::text_muted())
                                .max_w(px(300.0))
                                .child(desc),
                        )
                    }),
            )
            .child(Self::toggle_switch(id, value, on_click))
    }

    fn toggle_switch(
        id: &'static str,
        value: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(format!("toggle-{}", id)))
            .w(px(44.0))
            .h(px(24.0))
            .rounded_full()
            .cursor_pointer()
            .flex_shrink_0()
            .when(value, |el| el.bg(colors::accent()))
            .when(!value, |el| el.bg(rgba(0x78716c40)))
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

    fn render_dropdown_field(
        &self,
        label: &'static str,
        dropdown_id: DialogDropdownId,
        current_value: &'static str,
        _options: Vec<&'static str>,
        _is_open: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        // Note: Dropdown menu is rendered separately at dialog level to avoid clipping
        div()
            .py_2()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(colors::border())
            .child(
                div()
                    .text_sm()
                    .text_color(colors::text())
                    .child(label),
            )
            .child(self.render_dropdown_button(dropdown_id, current_value, cx))
    }

    fn render_dropdown_button(
        &self,
        dropdown_id: DialogDropdownId,
        current_value: &'static str,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(format!("dropdown-{:?}", dropdown_id)))
            .w(px(200.0))
            .px_2()
            .py_1()
            .rounded(px(4.0))
            .border_1()
            .border_color(colors::border())
            .bg(colors::background())
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
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
                div()
                    .text_xs()
                    .text_color(colors::text_muted())
                    .child("▾"),
            )
    }

    fn render_dropdown_option(
        &self,
        dropdown_id: DialogDropdownId,
        idx: usize,
        label: &'static str,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(format!(
                "dropdown-{:?}-option-{}",
                dropdown_id, idx
            )))
            .px_3()
            .py_2()
            .text_sm()
            .text_color(colors::text())
            .cursor_pointer()
            .hover(|el| el.bg(colors::hover()))
            .on_click(cx.listener(move |this, _, _window, cx| {
                match dropdown_id {
                    DialogDropdownId::Platform => this.select_platform(idx, cx),
                    DialogDropdownId::RestartPolicy => this.select_restart_policy(idx, cx),
                }
            }))
            .child(label)
    }

    /// Render dropdown overlay at dialog level (outside scrollable content)
    fn render_dropdown_overlay(&self, cx: &Context<Self>) -> impl IntoElement {
        let open_dropdown = self.open_dropdown;

        div()
            .absolute()
            .inset_0()
            .when(open_dropdown.is_none(), |el| el.invisible())
            .child(
                // Click backdrop to close dropdown
                div()
                    .id("dropdown-backdrop")
                    .absolute()
                    .inset_0()
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.open_dropdown = None;
                        cx.notify();
                    })),
            )
            .when_some(open_dropdown, |el, dropdown_id| {
                let (options, top_offset): (Vec<&'static str>, f32) = match dropdown_id {
                    DialogDropdownId::Platform => (
                        Platform::all().iter().map(|p| p.label()).collect(),
                        // Header (60px) + padding (16px) + first field (40px) + platform field position
                        60.0 + 16.0 + 44.0 + 32.0,
                    ),
                    DialogDropdownId::RestartPolicy => (
                        RestartPolicy::all().iter().map(|r| r.label()).collect(),
                        // Header + padding + image + platform + name + remove_after_stop + restart_policy position
                        60.0 + 16.0 + 44.0 + 44.0 + 44.0 + 56.0 + 32.0,
                    ),
                };

                el.child(
                    div()
                        .absolute()
                        .top(px(top_offset))
                        .right(px(25.0))
                        .w(px(200.0))
                        .bg(colors::background())
                        .border_1()
                        .border_color(colors::border())
                        .rounded_md()
                        .shadow_lg()
                        .overflow_hidden()
                        .children(options.into_iter().enumerate().map(|(idx, option)| {
                            self.render_dropdown_option(dropdown_id, idx, option, cx)
                        }))
                )
            })
    }

    fn render_footer(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .px_5()
            .py_3()
            .border_t_1()
            .border_color(colors::border())
            .flex()
            .items_center()
            .justify_between()
            // Help button
            .child(
                div()
                    .id("help-button")
                    .w(px(28.0))
                    .h(px(28.0))
                    .rounded_full()
                    .border_1()
                    .border_color(colors::border())
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_sm()
                    .text_color(colors::text_muted())
                    .hover(|el| el.bg(colors::hover()))
                    .child("?"),
            )
            // Action buttons
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    // Cancel button
                    .child(
                        div()
                            .id("cancel-button")
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .border_1()
                            .border_color(colors::border())
                            .bg(colors::background())
                            .text_sm()
                            .text_color(colors::text())
                            .cursor_pointer()
                            .hover(|el| el.bg(colors::surface()))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close(window, cx);
                            }))
                            .child("Cancel"),
                    )
                    // Create button
                    .child(
                        div()
                            .id("create-button")
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .border_1()
                            .border_color(colors::border())
                            .bg(colors::background())
                            .text_sm()
                            .text_color(colors::text())
                            .cursor_pointer()
                            .hover(|el| el.bg(colors::surface()))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.create_container(false, window, cx);
                            }))
                            .child("Create"),
                    )
                    // Create & Start button (primary)
                    .child(
                        div()
                            .id("create-start-button")
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(colors::accent())
                            .text_sm()
                            .text_color(colors::on_accent())
                            .cursor_pointer()
                            .hover(|el| el.bg(colors::accent_hover()))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.create_container(true, window, cx);
                            }))
                            .child("Create & Start"),
                    ),
            )
    }
}
