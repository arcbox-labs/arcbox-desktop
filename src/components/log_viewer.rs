//! Log viewer component for container logs.
//!
//! Displays streaming logs from containers with support for:
//! - Real-time log streaming (follow mode)
//! - stdout/stderr differentiation
//! - Timestamps display
//! - Auto-scroll to bottom

use gpui::*;
use gpui::prelude::*;

use crate::services::{DaemonEvent, DaemonService};
use crate::theme::colors;

/// A single log line with metadata
#[derive(Clone, Debug)]
pub struct LogLine {
    /// Log content
    pub content: String,
    /// Stream type: "stdout" or "stderr"
    pub stream: String,
    /// Unix timestamp in nanoseconds
    pub timestamp: i64,
}

/// Log viewer component
pub struct LogViewer {
    /// Container ID being viewed
    container_id: String,
    /// Daemon service for log subscription
    daemon_service: Entity<DaemonService>,
    /// Log lines buffer
    lines: Vec<LogLine>,
    /// Whether to auto-scroll to bottom
    follow: bool,
    /// Whether to show timestamps
    show_timestamps: bool,
    /// Whether logs have been requested
    subscribed: bool,
    /// Maximum lines to keep in buffer
    max_lines: usize,
}

impl LogViewer {
    pub fn new(
        container_id: String,
        daemon_service: Entity<DaemonService>,
        cx: &mut Context<Self>,
    ) -> Self {
        // Subscribe to daemon events for log entries
        cx.subscribe(&daemon_service, Self::on_daemon_event).detach();

        Self {
            container_id,
            daemon_service,
            lines: Vec::new(),
            follow: true,
            show_timestamps: true,
            subscribed: false,
            max_lines: 10000,
        }
    }

    /// Start subscribing to logs
    pub fn subscribe(&mut self, cx: &mut Context<Self>) {
        if self.subscribed {
            return;
        }
        self.subscribed = true;

        let container_id = self.container_id.clone();
        self.daemon_service.update(cx, |svc, cx| {
            svc.subscribe_logs(container_id, true, Some(100), cx);
        });
    }

    /// Clear all log lines
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.lines.clear();
        cx.notify();
    }

    /// Toggle follow mode
    pub fn toggle_follow(&mut self, cx: &mut Context<Self>) {
        self.follow = !self.follow;
        cx.notify();
    }

    /// Toggle timestamp display
    pub fn toggle_timestamps(&mut self, cx: &mut Context<Self>) {
        self.show_timestamps = !self.show_timestamps;
        cx.notify();
    }

    fn on_daemon_event(
        &mut self,
        _daemon: Entity<DaemonService>,
        event: &DaemonEvent,
        cx: &mut Context<Self>,
    ) {
        if let DaemonEvent::LogsReceived { container_id, entry } = event {
            if container_id == &self.container_id {
                // Decode log data
                let content = String::from_utf8_lossy(&entry.data).to_string();

                // Split by newlines and add each line
                for line_content in content.lines() {
                    if !line_content.is_empty() {
                        self.lines.push(LogLine {
                            content: line_content.to_string(),
                            stream: entry.stream.clone(),
                            timestamp: entry.timestamp,
                        });
                    }
                }

                // Trim buffer if too large
                if self.lines.len() > self.max_lines {
                    let excess = self.lines.len() - self.max_lines;
                    self.lines.drain(0..excess);
                }

                cx.notify();
            }
        }
    }

    /// Format timestamp for display
    fn format_timestamp(&self, timestamp_ns: i64) -> String {
        use chrono::{TimeZone, Utc};
        let secs = timestamp_ns / 1_000_000_000;
        let nsecs = (timestamp_ns % 1_000_000_000) as u32;
        if let Some(dt) = Utc.timestamp_opt(secs, nsecs).single() {
            dt.format("%H:%M:%S").to_string()
        } else {
            String::new()
        }
    }

    fn render_toolbar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(36.0))
            .px_3()
            .border_b_1()
            .border_color(colors::border_subtle())
            .bg(colors::surface())
            // Left side - info
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::text_secondary())
                            .child(format!("{} lines", self.lines.len())),
                    ),
            )
            // Right side - controls
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    // Timestamps toggle
                    .child(
                        div()
                            .id("toggle-timestamps")
                            .px_2()
                            .py_1()
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_xs()
                            .when(self.show_timestamps, |el| {
                                el.bg(colors::selection())
                                    .text_color(colors::on_accent())
                            })
                            .when(!self.show_timestamps, |el| {
                                el.hover(|el| el.bg(colors::hover()))
                                    .text_color(colors::text_secondary())
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_timestamps(cx);
                            }))
                            .child("Time"),
                    )
                    // Follow toggle
                    .child(
                        div()
                            .id("toggle-follow")
                            .px_2()
                            .py_1()
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_xs()
                            .when(self.follow, |el| {
                                el.bg(colors::selection())
                                    .text_color(colors::on_accent())
                            })
                            .when(!self.follow, |el| {
                                el.hover(|el| el.bg(colors::hover()))
                                    .text_color(colors::text_secondary())
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_follow(cx);
                            }))
                            .child("Follow"),
                    )
                    // Clear button
                    .child(
                        div()
                            .id("clear-logs")
                            .px_2()
                            .py_1()
                            .rounded(px(4.0))
                            .cursor_pointer()
                            .text_xs()
                            .text_color(colors::text_secondary())
                            .hover(|el| el.bg(colors::hover()))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear(cx);
                            }))
                            .child("Clear"),
                    ),
            )
    }

    fn render_log_line(&self, line: &LogLine) -> impl IntoElement {
        let is_stderr = line.stream == "stderr";

        div()
            .w_full()
            .min_w_0()
            .flex()
            .items_start()
            .gap_2()
            .px_3()
            .py_0p5()
            .font_family("monospace")
            .text_xs()
            .hover(|el| el.bg(colors::hover()))
            // Timestamp
            .when(self.show_timestamps, |el| {
                el.child(
                    div()
                        .flex_shrink_0()
                        .w(px(64.0))
                        .text_color(colors::text_muted())
                        .child(self.format_timestamp(line.timestamp)),
                )
            })
            // Stream indicator
            .child(
                div()
                    .flex_shrink_0()
                    .w(px(8.0))
                    .h(px(8.0))
                    .mt(px(4.0))
                    .rounded_full()
                    .bg(if is_stderr {
                        colors::error()
                    } else {
                        colors::text_muted()
                    }),
            )
            // Content
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_x_hidden()
                    .text_color(if is_stderr {
                        colors::error()
                    } else {
                        colors::text()
                    })
                    .child(line.content.clone()),
            )
    }

    fn render_empty_state(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .child(
                div()
                    .text_color(colors::text_secondary())
                    .child("No logs yet"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors::text_muted())
                    .child("Logs will appear here when the container produces output"),
            )
    }
}

impl Render for LogViewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Start subscription on first render
        if !self.subscribed {
            self.subscribe(cx);
        }

        div()
            .size_full()
            .min_w_0()
            .flex()
            .flex_col()
            .bg(colors::surface())
            .rounded(px(8.0))
            .border_1()
            .border_color(colors::border_subtle())
            .overflow_hidden()
            // Toolbar
            .child(self.render_toolbar(cx))
            // Log content
            .child(
                div()
                    .id("log-content")
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .overflow_x_hidden()
                    .overflow_y_scroll()
                    .bg(colors::background())
                    .when(self.lines.is_empty(), |el| {
                        el.child(self.render_empty_state())
                    })
                    .when(!self.lines.is_empty(), |el| {
                        el.child(
                            div()
                                .w_full()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .py_2()
                                .children(
                                    self.lines
                                        .iter()
                                        .map(|line| self.render_log_line(line)),
                                ),
                        )
                    }),
            )
    }
}
