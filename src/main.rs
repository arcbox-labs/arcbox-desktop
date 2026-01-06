mod app;
mod assets;
mod components;
mod models;
mod services;
mod theme;
mod views;

use gpui::*;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use app::ArcBoxApp;
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
