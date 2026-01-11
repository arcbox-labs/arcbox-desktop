use gpui::*;
use gpui::prelude::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};
use gpui_component::select::{Select, SelectItem, SelectState};
use gpui_component::switch::Switch;
use gpui_component::Sizable;
use gpui_component::Root;

use crate::theme::colors;
use crate::services::DaemonService;

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

    fn all() -> Vec<RestartPolicy> {
        vec![
            RestartPolicy::No,
            RestartPolicy::Always,
            RestartPolicy::OnFailure,
            RestartPolicy::UnlessStopped,
        ]
    }
}

impl SelectItem for RestartPolicy {
    type Value = Self;

    fn title(&self) -> SharedString {
        SharedString::from(self.label())
    }

    fn value(&self) -> &Self::Value {
        self
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

    fn all() -> Vec<Platform> {
        vec![Platform::Auto, Platform::LinuxAmd64, Platform::LinuxArm64]
    }
}

impl SelectItem for Platform {
    type Value = Self;

    fn title(&self) -> SharedString {
        SharedString::from(self.label())
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

/// New container dialog as a PopUp window
pub struct NewContainerDialog {
    // Basic settings - text inputs using gpui-component
    image_input: Entity<InputState>,
    platform_select: Entity<SelectState<Vec<Platform>>>,
    name_input: Entity<InputState>,
    remove_after_stop: bool,
    restart_policy_select: Entity<SelectState<Vec<RestartPolicy>>>,
    // Payload settings - text inputs using gpui-component
    command_input: Entity<InputState>,
    entrypoint_input: Entity<InputState>,
    workdir_input: Entity<InputState>,
    // Advanced settings
    privileged: bool,
    read_only: bool,
    use_docker_init: bool,
    // Services
    daemon_service: Entity<DaemonService>,
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

        // Create select state entities using gpui-component's SelectState
        // Default to first item (index 0) for both
        let platform_select = cx.new(|cx| {
            SelectState::new(
                Platform::all(),
                Some(gpui_component::IndexPath::default().row(0)),
                window,
                cx,
            )
        });

        let restart_policy_select = cx.new(|cx| {
            SelectState::new(
                RestartPolicy::all(),
                Some(gpui_component::IndexPath::default().row(0)),
                window,
                cx,
            )
        });

        Self {
            image_input,
            platform_select,
            name_input,
            remove_after_stop: false,
            restart_policy_select,
            command_input,
            entrypoint_input,
            workdir_input,
            privileged: false,
            read_only: false,
            use_docker_init: false,
            daemon_service,
        }
    }

    fn create_container(&mut self, start: bool, window: &mut Window, cx: &mut Context<Self>) {
        // Read values from InputState entities
        let image = self.image_input.read(cx).value().to_string();
        let name_value = self.name_input.read(cx).value().to_string();
        let command_value = self.command_input.read(cx).value().to_string();
        let entrypoint_value = self.entrypoint_input.read(cx).value().to_string();
        let workdir_value = self.workdir_input.read(cx).value().to_string();

        // Read selected values from SelectState entities
        let _platform = self.platform_select.read(cx).selected_value().copied().unwrap_or_default();
        let _restart_policy = self.restart_policy_select.read(cx).selected_value().copied().unwrap_or_default();

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
        self.daemon_service.update(cx, |svc, cx| {
            svc.create_container(image, name, cmd, entrypoint, working_dir, start, cx);
        });

        // Close the popup window
        window.remove_window();
    }

    fn close_dialog(&self, window: &mut Window, _cx: &mut Context<Self>) {
        window.remove_window();
    }
}

impl Render for NewContainerDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("new-container-dialog")
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::background())
            .text_color(colors::text())
            .rounded_lg()
            .border_1()
            .border_color(colors::border())
            .shadow_lg()
            // Title bar
            .child(self.render_title_bar(cx))
            // Scrollable form content
            .child(
                div()
                    .id("form-content")
                    .flex_1()
                    .overflow_y_scroll()
                    .px_4()
                    .py_2()
                    .flex()
                    .flex_col()
                    .gap_1()
                    // Basic settings
                    .child(self.render_input_field("Image", self.image_input.clone()))
                    .child(self.render_select_field("Platform", self.platform_select.clone()))
                    .child(self.render_input_field("Name", self.name_input.clone()))
                    .child(self.render_switch_field(
                        "remove-after-stop",
                        "Remove after stop",
                        Some("Automatically delete the container after it stops. (--rm)"),
                        self.remove_after_stop,
                        cx,
                    ))
                    .child(self.render_select_field("Restart policy", self.restart_policy_select.clone()))
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
                    .child(self.render_switch_field(
                        "privileged",
                        "Privileged",
                        Some("Allow access to privileged APIs and resources. (--privileged)"),
                        self.privileged,
                        cx,
                    ))
                    .child(self.render_switch_field(
                        "read-only",
                        "Read-only",
                        Some("Mount the container's root filesystem as read-only. (--read-only)"),
                        self.read_only,
                        cx,
                    ))
                    .child(self.render_switch_field(
                        "docker-init",
                        "Use docker-init",
                        Some("Run the container payload under a docker-init process. (--init)"),
                        self.use_docker_init,
                        cx,
                    ))
            )
            // Fixed footer (outside scroll area)
            .child(self.render_footer(cx))
    }
}

impl NewContainerDialog {
    fn render_title_bar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .h(px(44.0))
            .px_4()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(colors::border())
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::text())
                    .child("New Container"),
            )
            .child(
                div()
                    .id("close-button")
                    .w(px(24.0))
                    .h(px(24.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .hover(|el| el.bg(colors::hover()))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.close_dialog(window, cx);
                    }))
                    .child(
                        svg()
                            .path("icons/close.svg")
                            .size(px(16.0))
                            .text_color(colors::text_secondary()),
                    ),
            )
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
                    .child(Input::new(&input).small()),
            )
    }

    /// Render a text input field with a label and description
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
                            .text_color(colors::text_secondary())
                            .max_w(px(220.0))
                            .child(description),
                    ),
            )
            .child(
                div()
                    .w(px(200.0))
                    .child(Input::new(&input).small()),
            )
    }

    /// Render a switch field using gpui-component's Switch
    fn render_switch_field(
        &self,
        id: &'static str,
        label: &'static str,
        description: Option<&'static str>,
        value: bool,
        cx: &Context<Self>,
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
                                .text_color(colors::text_secondary())
                                .max_w(px(300.0))
                                .child(desc),
                        )
                    }),
            )
            .child(
                Switch::new(SharedString::from(format!("switch-{}", id)))
                    .checked(value)
                    .small()
                    .on_click(cx.listener(move |this, checked: &bool, _window, cx| {
                        match id {
                            "remove-after-stop" => this.remove_after_stop = *checked,
                            "privileged" => this.privileged = *checked,
                            "read-only" => this.read_only = *checked,
                            "docker-init" => this.use_docker_init = *checked,
                            _ => {}
                        }
                        cx.notify();
                    })),
            )
    }

    /// Render a select/dropdown field using gpui-component's Select
    fn render_select_field<D>(
        &self,
        label: &'static str,
        select_state: Entity<SelectState<D>>,
    ) -> impl IntoElement
    where
        D: gpui_component::select::SelectDelegate + 'static,
    {
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
                    .child(Select::new(&select_state).small()),
            )
    }

    fn render_footer(&self, cx: &Context<Self>) -> impl IntoElement {
        // Get entity handle for use in button callbacks
        // gpui-component Button.on_click expects Fn(&ClickEvent, &mut Window, &mut App)
        // so we need to use entity.update() pattern instead of cx.listener()
        let entity = cx.entity();

        let cancel_entity = entity.clone();
        let create_entity = entity.clone();
        let create_start_entity = entity.clone();

        div()
            .px_4()
            .py_3()
            .border_t_1()
            .border_color(colors::border())
            .flex()
            .items_center()
            .justify_between()
            // Help button
            .child(
                Button::new("help-button")
                    .ghost()
                    .small()
                    .child("?")
                    .rounded_full()
            )
            // Action buttons
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    // Cancel button
                    .child(
                        Button::new("cancel-button")
                            .ghost()
                            .small()
                            .child("Cancel")
                            .on_click(move |_, window, cx| {
                                cancel_entity.update(cx, |this, cx| {
                                    this.close_dialog(window, cx);
                                });
                            })
                    )
                    // Create button
                    .child(
                        Button::new("create-button")
                            .ghost()
                            .small()
                            .child("Create")
                            .on_click(move |_, window, cx| {
                                create_entity.update(cx, |this, cx| {
                                    this.create_container(false, window, cx);
                                });
                            })
                    )
                    // Create & Start button (primary)
                    .child(
                        Button::new("create-start-button")
                            .primary()
                            .small()
                            .child("Create & Start")
                            .on_click(move |_, window, cx| {
                                create_start_entity.update(cx, |this, cx| {
                                    this.create_container(true, window, cx);
                                });
                            })
                    ),
            )
    }
}

/// Open the new container dialog as a PopUp window
///
/// The popup window can overflow its bounds (e.g., Select dropdown menus),
/// unlike an in-window overlay dialog.
pub fn open_new_container_dialog(
    daemon_service: Entity<DaemonService>,
    parent_bounds: Bounds<Pixels>,
    cx: &mut App,
) {
    // Dialog size
    let dialog_size = size(px(480.0), px(560.0));

    // Calculate centered position relative to parent window
    let x = parent_bounds.origin.x + (parent_bounds.size.width - dialog_size.width) / 2.0;
    let y = parent_bounds.origin.y + (parent_bounds.size.height - dialog_size.height) / 2.0;

    let bounds = Bounds {
        origin: point(x, y),
        size: dialog_size,
    };

    let window_options = WindowOptions {
        kind: WindowKind::PopUp,
        titlebar: None,
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        focus: true,
        show: true,
        window_background: WindowBackgroundAppearance::Transparent,
        ..Default::default()
    };

    let _ = cx.open_window(window_options, |window, cx| {
        // Initialize gpui-component (required for Input, Select, etc.)
        gpui_component::init(cx);

        // Create the dialog content view
        let dialog_view = cx.new(|cx| NewContainerDialog::new(daemon_service, window, cx));

        // Wrap in Root - gpui-component requires Root to be the window's root view
        // for its components (Input, Select, Switch, etc.) to work properly
        cx.new(|cx| Root::new(dialog_view, window, cx))
    });
}
