mod app;
mod assets;
mod components;
mod models;
mod services;
mod theme;
mod views;

use gpui::*;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use app::{ArcBoxApp, OpenSettings, Quit, open_settings};
use assets::AppAssets;

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
        // Initialize tokio runtime for async operations (e.g., dimicon HTTP requests)
        gpui_tokio::init(cx);

        // Initialize HTTP client for loading remote images
        let http_client = reqwest_client::ReqwestClient::user_agent("arcbox-desktop/0.1.0")
            .expect("Failed to create HTTP client");
        cx.set_http_client(std::sync::Arc::new(http_client));

        // Initialize theme
        theme::init(cx);

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
                traffic_light_position: Some(point(px(9.0), px(9.0))),
            }),
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            focus: true,
            show: true,
            kind: WindowKind::Normal,
            is_movable: true,
            window_background: WindowBackgroundAppearance::Opaque,
            ..Default::default()
        };

        cx.open_window(window_options, |_window, cx| cx.new(ArcBoxApp::new))
            .expect("Failed to open main window");

        cx.activate(true);

        tracing::info!("ArcBox Desktop started successfully");
    });
}
