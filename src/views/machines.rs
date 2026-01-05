use gpui::*;
use gpui::prelude::*;

use crate::components::MachineStatusBadge;
use crate::models::{MachineViewModel, dummy_machines};
use crate::theme::{colors, Theme};

/// Machines list view
pub struct MachinesView {
    machines: Vec<MachineViewModel>,
    selected_id: Option<String>,
}

impl MachinesView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            machines: dummy_machines(),
            selected_id: None,
        }
    }

    fn select_machine(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }

    fn start_machine(&mut self, id: &str, cx: &mut Context<Self>) {
        tracing::info!("Start machine: {}", id);
        cx.notify();
    }

    fn stop_machine(&mut self, id: &str, cx: &mut Context<Self>) {
        tracing::info!("Stop machine: {}", id);
        cx.notify();
    }

    fn create_machine(&mut self, cx: &mut Context<Self>) {
        tracing::info!("Create new machine");
        cx.notify();
    }
}

impl Render for MachinesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let running_count = self.machines.iter().filter(|m| m.is_running()).count();
        let total_count = self.machines.len();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(colors::background())
            // Header
            .child(
                Theme::section_header()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(Theme::page_title("Machines"))
                            .child(
                                Theme::badge()
                                    .child(format!("{} / {} running", running_count, total_count)),
                            ),
                    )
                    .child(
                        Theme::button_primary()
                            .id("create-machine")
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.create_machine(cx);
                            }))
                            .child("+ New Machine"),
                    ),
            )
            // Machine list
            .child(
                div()
                    .id("machines-list")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .children(self.machines.iter().map(|m| self.render_machine_card(m, cx))),
                    ),
            )
    }
}

impl MachinesView {
    fn render_machine_card(
        &self,
        machine: &MachineViewModel,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let id = machine.id.clone();
        let id_for_click = machine.id.clone();
        let id_for_stop = machine.id.clone();
        let id_for_start = machine.id.clone();
        let is_selected = self.selected_id.as_ref() == Some(&id);
        let is_running = machine.is_running();

        Theme::card()
            .id(SharedString::from(format!("machine-{}", &id)))
            .cursor_pointer()
            .when(is_selected, |el| {
                el.border_color(colors::accent())
            })
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.select_machine(id_for_click.clone(), cx);
            }))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p_4()
                    // Header row
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            // Linux icon
                            .child(div().text_2xl().child("🐧"))
                            // Name and distro
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_color(colors::text())
                                            .child(machine.name.clone()),
                                    )
                                    .child(Theme::subtitle(machine.distro.display_name.clone())),
                            )
                            // Status
                            .child(MachineStatusBadge::new(machine.state)),
                    )
                    // Divider
                    .child(div().my_3().h(px(1.0)).bg(colors::border()))
                    // Resource info
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .text_sm()
                            .text_color(colors::text_muted())
                            .child(format!("CPU: {} cores", machine.cpu_cores))
                            .child(format!("Memory: {} GB", machine.memory_gb))
                            .child(format!("Disk: {} GB", machine.disk_gb)),
                    )
                    // IP address (if running)
                    .when_some(machine.ip_address.clone(), |el, ip| {
                        el.child(
                            div()
                                .mt_2()
                                .text_sm()
                                .text_color(colors::text_muted())
                                .child(format!("IP: {}", ip)),
                        )
                    })
                    // Action buttons
                    .child(
                        div()
                            .mt_4()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when(is_running, |el| {
                                el.child(Theme::button_ghost().child("Terminal"))
                                    .child(Theme::button_ghost().child("Files"))
                                    .child(
                                        Theme::button_ghost()
                                            .id(SharedString::from(format!("stop-{}", &id)))
                                            .on_click(cx.listener(move |this, _, _window, cx| {
                                                this.stop_machine(&id_for_stop, cx);
                                            }))
                                            .child("Stop"),
                                    )
                            })
                            .when(!is_running, |el| {
                                el.child(
                                    Theme::button_primary()
                                        .id(SharedString::from(format!("start-{}", &id)))
                                        .on_click(cx.listener(move |this, _, _window, cx| {
                                            this.start_machine(&id_for_start, cx);
                                        }))
                                        .child("Start"),
                                )
                                .child(
                                    Theme::button_ghost()
                                        .text_color(colors::error())
                                        .child("Delete"),
                                )
                            }),
                    ),
            )
    }
}
