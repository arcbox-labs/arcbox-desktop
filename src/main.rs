mod app;
mod assets;
mod components;
mod models;
mod services;
mod theme;
mod tokio_bridge;
mod views;

use gpui::*;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use app::{ArcBoxApp, OpenSettings, Quit, open_settings};
use assets::AppAssets;
use components::register_text_input_bindings;

/// Set the application dock icon on macOS.
/// This is needed for `cargo run` since there's no app bundle with Info.plist.
#[cfg(target_os = "macos")]
fn set_dock_icon() {
    use cocoa::appkit::{NSApplication, NSImage};
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;

    unsafe {
        // Get the icon path - try multiple locations
        let icon_path = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .and_then(|dir| {
                // Check relative to executable first (for bundled app)
                let bundled = dir.join("../Resources/AppIcon.icns");
                if bundled.exists() {
                    return Some(bundled);
                }
                // Then check in project directory (for cargo run)
                let project = dir.join("../../bundle/AppIcon.icns");
                if project.exists() {
                    return Some(project);
                }
                // Try absolute path as fallback
                let absolute = std::path::PathBuf::from(
                    "/Users/Shiro/Developer/arcboxd/arcbox-desktop/bundle/AppIcon.icns"
                );
                if absolute.exists() {
                    return Some(absolute);
                }
                None
            });

        if let Some(path) = icon_path {
            let path_str = path.to_string_lossy();
            let ns_path = NSString::alloc(nil).init_str(&path_str);
            let image: id = NSImage::alloc(nil).initWithContentsOfFile_(ns_path);

            if image != nil {
                let app = NSApplication::sharedApplication(nil);
                app.setApplicationIconImage_(image);
                tracing::debug!("Dock icon set from: {}", path_str);
            } else {
                tracing::warn!("Failed to load dock icon from: {}", path_str);
            }
        } else {
            tracing::warn!("Dock icon file not found");
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn set_dock_icon() {
    // No-op on non-macOS platforms
}

fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("arcbox_desktop=debug".parse().unwrap()))
        .init();

    tracing::info!("Starting ArcBox Desktop...");

    Application::new()
        .with_assets(AppAssets::new())
        .run(|cx: &mut App| {
        // Set dock icon for cargo run (no app bundle)
        // Must be called after Application::new() to avoid conflicts with GPUI's NSApplication setup
        set_dock_icon();

        // Initialize tokio runtime for async operations (e.g., dimicon HTTP requests)
        tokio_bridge::init(cx);

        // Initialize gpui-component (theme, input key bindings, etc.)
        gpui_component::init(cx);

        // Initialize our custom theme (after gpui-component)
        theme::init(cx);

        // Register text input bindings
        register_text_input_bindings(cx);

        // Register global actions
        cx.on_action(|_: &OpenSettings, cx| {
            open_settings(cx);
        });

        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        // Bind Cmd+, to open settings
        cx.bind_keys([
            KeyBinding::new("cmd-,", OpenSettings, None),
            KeyBinding::new("cmd-q", Quit, None),
        ]);

        // Set up application menu
        cx.set_menus(vec![
            Menu {
                name: "ArcBox".into(),
                items: vec![
                    MenuItem::action("About ArcBox", gpui::NoAction), // Placeholder
                    MenuItem::separator(),
                    MenuItem::action("Settings...", OpenSettings),
                    MenuItem::separator(),
                    MenuItem::action("Quit ArcBox", Quit),
                ],
            },
            Menu {
                name: "Edit".into(),
                items: vec![
                    MenuItem::os_action("Undo", gpui::NoAction, OsAction::Undo),
                    MenuItem::os_action("Redo", gpui::NoAction, OsAction::Redo),
                    MenuItem::separator(),
                    MenuItem::os_action("Cut", gpui::NoAction, OsAction::Cut),
                    MenuItem::os_action("Copy", gpui::NoAction, OsAction::Copy),
                    MenuItem::os_action("Paste", gpui::NoAction, OsAction::Paste),
                    MenuItem::os_action("Select All", gpui::NoAction, OsAction::SelectAll),
                ],
            },
            Menu {
                name: "Window".into(),
                items: vec![
                    MenuItem::action("Minimize", gpui::NoAction),
                    MenuItem::action("Zoom", gpui::NoAction),
                ],
            },
        ]);

        // Create main window
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
        let window_options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("ArcBox".into()),
                appears_transparent: true,
                // macOS standard traffic light position (slightly inset from edge)
                traffic_light_position: Some(point(px(13.0), px(16.0))),
            }),
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            focus: true,
            show: true,
            kind: WindowKind::Normal,
            is_movable: true,
            window_background: WindowBackgroundAppearance::Opaque,
            ..Default::default()
        };

        cx.open_window(window_options, |window, cx| {
            let app_view = cx.new(ArcBoxApp::new);
            // Wrap in gpui_component::Root for Input/Dialog support
            cx.new(|cx| gpui_component::Root::new(app_view, window, cx))
        })
        .expect("Failed to open main window");

        cx.activate(true);

        tracing::info!("ArcBox Desktop started successfully");
    });
}
