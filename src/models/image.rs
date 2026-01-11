use arcbox_api::generated::ImageSummary;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Image view model for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageViewModel {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub in_use: bool,
    pub os: String,
    pub architecture: String,
}

impl ImageViewModel {
    /// Full image name with tag
    pub fn full_name(&self) -> String {
        if self.repository == "<none>" {
            format!("<none>:{}", self.tag)
        } else {
            format!("{}:{}", self.repository, self.tag)
        }
    }

    /// Display size in human readable format
    pub fn size_display(&self) -> String {
        let mb = self.size_bytes as f64 / 1_000_000.0;
        if mb >= 1000.0 {
            format!("{:.1} GB", mb / 1000.0)
        } else {
            format!("{:.0} MB", mb)
        }
    }

    /// Display relative time since creation
    pub fn created_ago(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.created_at);

        if duration.num_days() >= 30 {
            let months = duration.num_days() / 30;
            format!("{} month{} ago", months, if months > 1 { "s" } else { "" })
        } else if duration.num_days() >= 7 {
            let weeks = duration.num_days() / 7;
            format!("{} week{} ago", weeks, if weeks > 1 { "s" } else { "" })
        } else if duration.num_days() > 0 {
            format!(
                "{} day{} ago",
                duration.num_days(),
                if duration.num_days() > 1 { "s" } else { "" }
            )
        } else if duration.num_hours() > 0 {
            format!(
                "{} hour{} ago",
                duration.num_hours(),
                if duration.num_hours() > 1 { "s" } else { "" }
            )
        } else {
            "just now".to_string()
        }
    }
}

/// Calculate total and unused image sizes
pub fn calculate_image_stats(images: &[ImageViewModel]) -> (u64, u64, usize, usize) {
    let total_size: u64 = images.iter().map(|i| i.size_bytes).sum();
    let unused_size: u64 = images.iter().filter(|i| !i.in_use).map(|i| i.size_bytes).sum();
    let total_count = images.len();
    let unused_count = images.iter().filter(|i| !i.in_use).count();
    (total_size, unused_size, total_count, unused_count)
}

impl From<ImageSummary> for ImageViewModel {
    fn from(summary: ImageSummary) -> Self {
        // Parse repository and tag from repo_tags
        let (repository, tag) = summary
            .repo_tags
            .first()
            .map(|rt| {
                if let Some((repo, tag)) = rt.rsplit_once(':') {
                    (repo.to_string(), tag.to_string())
                } else {
                    (rt.clone(), "latest".to_string())
                }
            })
            .unwrap_or_else(|| ("<none>".to_string(), "<none>".to_string()));

        // Parse created timestamp (Unix seconds)
        let created_at = Utc
            .timestamp_opt(summary.created, 0)
            .single()
            .unwrap_or_else(Utc::now);

        Self {
            id: summary.id,
            repository,
            tag,
            size_bytes: summary.size as u64,
            created_at,
            in_use: false, // TODO: Track container usage
            os: "linux".to_string(),
            architecture: "arm64".to_string(), // TODO: Get from image config
        }
    }
}
