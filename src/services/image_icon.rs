use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use dimicon::IconService;
use gpui::*;
use serde::{Deserialize, Serialize};

use crate::tokio_bridge::Tokio;

/// Cached icon state
#[derive(Clone, Debug)]
pub enum IconState {
    /// Icon is being fetched
    Loading,
    /// Icon file path (local cached file) - PathBuf so GPUI treats it as file, not embedded asset
    Found(PathBuf),
    /// Icon was not found
    NotFound,
    /// Error occurred during fetch
    Error(String),
}

/// Persistent cache entry for serialization
#[derive(Serialize, Deserialize)]
struct CacheEntry {
    /// Local file path (relative to icons cache dir)
    file: Option<String>,
}

/// Service for fetching Docker image icons using dimicon
pub struct ImageIconService {
    /// The dimicon service instance
    icon_service: Arc<IconService>,
    /// Cache of icon states by image name
    cache: HashMap<String, IconState>,
    /// Path to the cache metadata file
    cache_path: PathBuf,
    /// Path to the icons cache directory
    icons_dir: PathBuf,
}

impl ImageIconService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let cache_dir = Self::get_cache_dir();
        let cache_path = cache_dir.join("icons.json");
        let icons_dir = cache_dir.join("icons");

        // Ensure icons directory exists
        let _ = fs::create_dir_all(&icons_dir);

        let cache = Self::load_cache(&cache_path, &icons_dir);

        Self {
            icon_service: Arc::new(IconService::new()),
            cache,
            cache_path,
            icons_dir,
        }
    }

    /// Get the cache directory
    fn get_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("arcbox")
    }

    /// Load cache from file, validating that icon files still exist
    fn load_cache(path: &PathBuf, icons_dir: &PathBuf) -> HashMap<String, IconState> {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(entries) = serde_json::from_str::<HashMap<String, CacheEntry>>(&content) {
                tracing::debug!("Loaded {} cached icons from {:?}", entries.len(), path);
                return entries
                    .into_iter()
                    .filter_map(|(k, v)| {
                        match v.file {
                            Some(file) => {
                                let full_path = icons_dir.join(&file);
                                if full_path.exists() {
                                    // Return the full path as PathBuf for GPUI to load as file
                                    Some((k, IconState::Found(full_path)))
                                } else {
                                    // File was deleted, need to re-fetch
                                    None
                                }
                            }
                            None => Some((k, IconState::NotFound)),
                        }
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
                IconState::Found(path) => {
                    // Store only the filename, not the full path
                    let file = path.file_name()?.to_string_lossy().to_string();
                    Some((k.clone(), CacheEntry { file: Some(file) }))
                }
                IconState::NotFound => Some((k.clone(), CacheEntry { file: None })),
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

    /// Fetch icon asynchronously using tokio bridge
    fn fetch_icon(&mut self, image: String, cx: &mut Context<Self>) {
        let service = self.icon_service.clone();
        let image_for_fetch = image.clone();
        let icons_dir = self.icons_dir.clone();

        // Use Tokio bridge to run the async fetch and download
        let task = Tokio::spawn(cx, async move {
            // First get the icon URL from dimicon
            let icon_source = service.get_icon(&image_for_fetch).await?;

            let Some(url) = icon_source.url() else {
                return Ok::<Option<PathBuf>, dimicon::Error>(None);
            };

            // Generate a safe filename from the image name
            let safe_name = image_for_fetch
                .replace('/', "_")
                .replace(':', "_")
                .replace('.', "_");

            // Determine file extension from URL
            let extension = url
                .rsplit('.')
                .next()
                .filter(|ext| ["png", "jpg", "jpeg", "svg", "webp"].contains(ext))
                .unwrap_or("png");

            let filename = format!("{}.{}", safe_name, extension);
            let file_path = icons_dir.join(&filename);

            // Download the image if not already cached
            if !file_path.exists() {
                tracing::debug!("Downloading icon for {} from {}", image_for_fetch, url);

                let response = reqwest::get(url).await.map_err(|e| {
                    dimicon::Error::Network(format!("Failed to download icon: {}", e))
                })?;

                if !response.status().is_success() {
                    return Err(dimicon::Error::Network(format!(
                        "HTTP error: {}",
                        response.status()
                    )));
                }

                let bytes = response.bytes().await.map_err(|e| {
                    dimicon::Error::Network(format!("Failed to read response: {}", e))
                })?;

                // Write to file
                fs::write(&file_path, &bytes).map_err(|e| {
                    dimicon::Error::Network(format!("Failed to write icon file: {}", e))
                })?;

                tracing::debug!("Saved icon to {:?}", file_path);
            }

            Ok(Some(file_path))
        });

        // Handle the result when the task completes
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let result = task.await;

            cx.update(|cx| {
                this.update(cx, |this, cx| {
                    let state = match result {
                        Ok(Ok(Some(path))) => {
                            tracing::debug!("Icon ready for {}: {:?}", image, path);
                            IconState::Found(path)
                        }
                        Ok(Ok(None)) => {
                            tracing::debug!("No icon found for {}", image);
                            IconState::NotFound
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
