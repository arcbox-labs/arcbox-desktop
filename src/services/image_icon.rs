use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use dimicon::IconService;
use gpui::*;
use gpui_tokio::Tokio;
use serde::{Deserialize, Serialize};

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

/// Persistent cache entry for serialization
#[derive(Serialize, Deserialize)]
struct CacheEntry {
    url: Option<String>,
}

/// Service for fetching Docker image icons using dimicon
pub struct ImageIconService {
    /// The dimicon service instance
    icon_service: Arc<IconService>,
    /// Cache of icon states by image name
    cache: HashMap<String, IconState>,
    /// Path to the cache file
    cache_path: PathBuf,
}

impl ImageIconService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let cache_path = Self::get_cache_path();
        let cache = Self::load_cache(&cache_path);

        Self {
            icon_service: Arc::new(IconService::new()),
            cache,
            cache_path,
        }
    }

    /// Get the cache file path
    fn get_cache_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("arcbox")
            .join("image_icons.json")
    }

    /// Load cache from file
    fn load_cache(path: &PathBuf) -> HashMap<String, IconState> {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(entries) = serde_json::from_str::<HashMap<String, CacheEntry>>(&content) {
                tracing::debug!("Loaded {} cached icons from {:?}", entries.len(), path);
                return entries
                    .into_iter()
                    .map(|(k, v)| {
                        let state = match v.url {
                            Some(url) => IconState::Found(url),
                            None => IconState::NotFound,
                        };
                        (k, state)
                    })
                    .collect();
            }
        }
        HashMap::new()
    }

    /// Save cache to file
    fn save_cache(&self) {
        let entries: HashMap<String, CacheEntry> = self
            .cache
            .iter()
            .filter_map(|(k, v)| match v {
                IconState::Found(url) => Some((
                    k.clone(),
                    CacheEntry {
                        url: Some(url.clone()),
                    },
                )),
                IconState::NotFound => Some((k.clone(), CacheEntry { url: None })),
                _ => None,
            })
            .collect();

        if let Some(parent) = self.cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(content) = serde_json::to_string_pretty(&entries) {
            if let Err(e) = fs::write(&self.cache_path, content) {
                tracing::warn!("Failed to save icon cache: {}", e);
            }
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
                    this.save_cache();
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
