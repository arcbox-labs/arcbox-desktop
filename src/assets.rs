use std::borrow::Cow;
use std::path::PathBuf;
use std::{fs, io};

use gpui::{AssetSource, SharedString};

/// Asset source for the application.
/// Loads assets from the assets directory relative to the executable.
pub struct AppAssets {
    base: PathBuf,
}

impl AppAssets {
    pub fn new() -> Self {
        // In development, use the assets directory relative to the project root
        // In production, assets would be bundled with the application
        let base = if cfg!(debug_assertions) {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
        } else {
            // For release builds, look for assets relative to the executable
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                .map(|p| p.join("assets"))
                .unwrap_or_else(|| PathBuf::from("assets"))
        };

        Self { base }
    }
}

impl Default for AppAssets {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        let full_path = self.base.join(path);
        tracing::debug!("Loading asset: {} from {:?}", path, full_path);
        match fs::read(&full_path) {
            Ok(data) => {
                tracing::debug!("Asset loaded successfully: {} ({} bytes)", path, data.len());
                Ok(Some(Cow::Owned(data)))
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                tracing::warn!("Asset not found: {:?}", full_path);
                Ok(None)
            }
            Err(err) => {
                tracing::error!("Error loading asset {:?}: {}", full_path, err);
                Err(err.into())
            }
        }
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        let full_path = self.base.join(path);
        match fs::read_dir(&full_path) {
            Ok(entries) => {
                let files: Vec<SharedString> = entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|e| e.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect();
                Ok(files)
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(err) => Err(err.into()),
        }
    }
}
