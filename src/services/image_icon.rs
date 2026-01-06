use std::collections::HashMap;
use std::sync::Arc;

use dimicon::IconService;
use gpui::*;
use gpui_tokio::Tokio;

/// Cached icon state
#[derive(Clone, Debug)]
pub enum IconState {
    /// Icon is being fetched
    Loading,
    /// Icon URL was found
    Found(String),
    /// Icon was not found
    NotFound,
    /// Error occurred during fetch
    Error(String),
}

/// Service for fetching Docker image icons using dimicon
pub struct ImageIconService {
    /// The dimicon service instance
    icon_service: Arc<IconService>,
    /// Cache of icon states by image name
    cache: HashMap<String, IconState>,
}

impl ImageIconService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            icon_service: Arc::new(IconService::new()),
            cache: HashMap::new(),
        }
    }

    /// Get the icon state for an image, triggering a fetch if not cached
    pub fn get_icon(&mut self, image: &str, cx: &mut Context<Self>) -> IconState {
        // Check cache first
        if let Some(state) = self.cache.get(image) {
            return state.clone();
        }

        // Mark as loading and start fetch
        self.cache.insert(image.to_string(), IconState::Loading);
        self.fetch_icon(image.to_string(), cx);

        IconState::Loading
    }

    /// Fetch icon asynchronously using gpui_tokio
    fn fetch_icon(&mut self, image: String, cx: &mut Context<Self>) {
        let service = self.icon_service.clone();
        let image_for_fetch = image.clone();

        // Use gpui_tokio to run the tokio-based dimicon fetch
        let task = Tokio::spawn(cx, async move {
            service.get_icon(&image_for_fetch).await
        });

        // Handle the result when the task completes
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let result = task.await;

            cx.update(|cx| {
                this.update(cx, |this, cx| {
                    let state = match result {
                        Ok(Ok(icon_source)) => {
                            if let Some(url) = icon_source.url() {
                                tracing::debug!("Found icon for {}: {}", image, url);
                                IconState::Found(url.to_string())
                            } else {
                                tracing::debug!("No icon found for {}", image);
                                IconState::NotFound
                            }
                        }
                        Ok(Err(e)) => {
                            tracing::warn!("Error fetching icon for {}: {}", image, e);
                            IconState::Error(e.to_string())
                        }
                        Err(e) => {
                            tracing::error!("Tokio task error for {}: {}", image, e);
                            IconState::Error(e.to_string())
                        }
                    };

                    this.cache.insert(image, state);
                    cx.notify();
                })
            })
            .ok();
        })
        .detach();
    }

    /// Check if an icon is cached
    pub fn is_cached(&self, image: &str) -> bool {
        self.cache.contains_key(image)
    }

    /// Get cached icon state without triggering a fetch
    pub fn get_cached(&self, image: &str) -> Option<&IconState> {
        self.cache.get(image)
    }

    /// Clear the icon cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
