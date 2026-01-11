//! Simple text input component for GPUI.
//!
//! This is a basic implementation that handles keyboard input via the action system.

use gpui::*;
use gpui::prelude::*;

use crate::theme::colors;

/// Actions for text input
actions!(text_input, [Backspace, DeleteChar]);

/// A simple text input field
pub struct TextInput {
    focus_handle: FocusHandle,
    value: String,
    placeholder: String,
    on_change: Option<Box<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
}

impl TextInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            value: String::new(),
            placeholder: String::new(),
            on_change: None,
        }
    }

    pub fn with_value(value: impl Into<String>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            value: value.into(),
            placeholder: String::new(),
            on_change: None,
        }
    }

    pub fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.value = value.into();
        cx.notify();
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    pub fn set_on_change(&mut self, callback: impl Fn(&str, &mut Window, &mut App) + 'static) {
        self.on_change = Some(Box::new(callback));
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    fn handle_backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if !self.value.is_empty() {
            self.value.pop();
            if let Some(on_change) = &self.on_change {
                on_change(&self.value, window, cx);
            }
            cx.notify();
        }
    }

    fn handle_input(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.value.push_str(text);
        if let Some(on_change) = &self.on_change {
            on_change(&self.value, window, cx);
        }
        cx.notify();
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let is_empty = self.value.is_empty();
        let display_text = if is_empty {
            self.placeholder.clone()
        } else {
            self.value.clone()
        };

        div()
            .id("text-input")
            .track_focus(&self.focus_handle)
            .key_context("TextInput")
            .on_action(cx.listener(Self::handle_backspace))
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                // Handle printable characters via key_char (includes IME input)
                if let Some(key_char) = &event.keystroke.key_char {
                    this.handle_input(key_char, window, cx);
                } else if event.keystroke.key.len() == 1 && !event.keystroke.modifiers.control && !event.keystroke.modifiers.platform {
                    // Single character key (a-z, 0-9, symbols)
                    let key = if event.keystroke.modifiers.shift {
                        event.keystroke.key.to_uppercase()
                    } else {
                        event.keystroke.key.clone()
                    };
                    this.handle_input(&key, window, cx);
                }
            }))
            .w_full()
            .px_2()
            .py_1()
            .rounded(px(4.0))
            .border_1()
            .when(is_focused, |el| el.border_color(colors::accent()))
            .when(!is_focused, |el| el.border_color(colors::border()))
            .bg(colors::background())
            .text_sm()
            .cursor_text()
            .on_click(cx.listener(move |this, _, window, _cx| {
                this.focus_handle.focus(window);
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .text_color(if is_empty {
                        colors::text_muted()
                    } else {
                        colors::text()
                    })
                    .child(display_text)
                    // Cursor indicator when focused
                    .when(is_focused, |el| {
                        el.child(
                            div()
                                .w(px(1.0))
                                .h(px(14.0))
                                .bg(colors::text())
                                .ml(px(1.0))
                        )
                    })
            )
    }
}

/// Register text input actions with key bindings
pub fn register_text_input_bindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, Some("TextInput")),
    ]);
}
