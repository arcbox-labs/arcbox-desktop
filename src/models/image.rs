use chrono::{DateTime, Utc};
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

/// Create dummy images for development
pub fn dummy_images() -> Vec<ImageViewModel> {
    use chrono::Duration;
    let now = Utc::now();

    vec![
        ImageViewModel {
            id: "sha256:abc123".to_string(),
            repository: "nginx".to_string(),
            tag: "latest".to_string(),
            size_bytes: 187_000_000,
            created_at: now - Duration::weeks(2),
            in_use: true,
        },
        ImageViewModel {
            id: "sha256:bcd234".to_string(),
            repository: "postgres".to_string(),
            tag: "15".to_string(),
            size_bytes: 412_000_000,
            created_at: now - Duration::days(30),
            in_use: true,
        },
        ImageViewModel {
            id: "sha256:cde345".to_string(),
            repository: "postgres".to_string(),
            tag: "14".to_string(),
            size_bytes: 398_000_000,
            created_at: now - Duration::days(90),
            in_use: false,
        },
        ImageViewModel {
            id: "sha256:def456".to_string(),
            repository: "redis".to_string(),
            tag: "alpine".to_string(),
            size_bytes: 32_000_000,
            created_at: now - Duration::weeks(1),
            in_use: true,
        },
        ImageViewModel {
            id: "sha256:efg567".to_string(),
            repository: "node".to_string(),
            tag: "20-alpine".to_string(),
            size_bytes: 128_000_000,
            created_at: now - Duration::weeks(2),
            in_use: false,
        },
        ImageViewModel {
            id: "sha256:fgh678".to_string(),
            repository: "my-app".to_string(),
            tag: "dev".to_string(),
            size_bytes: 245_000_000,
            created_at: now - Duration::hours(2),
            in_use: true,
        },
        ImageViewModel {
            id: "sha256:ghi789".to_string(),
            repository: "my-app".to_string(),
            tag: "prod".to_string(),
            size_bytes: 198_000_000,
            created_at: now - Duration::days(1),
            in_use: false,
        },
        ImageViewModel {
            id: "sha256:hij890".to_string(),
            repository: "<none>".to_string(),
            tag: "<none>".to_string(),
            size_bytes: 156_000_000,
            created_at: now - Duration::days(5),
            in_use: false,
        },
    ]
}

/// Calculate total and unused image sizes
pub fn calculate_image_stats(images: &[ImageViewModel]) -> (u64, u64, usize, usize) {
    let total_size: u64 = images.iter().map(|i| i.size_bytes).sum();
    let unused_size: u64 = images.iter().filter(|i| !i.in_use).map(|i| i.size_bytes).sum();
    let total_count = images.len();
    let unused_count = images.iter().filter(|i| !i.in_use).count();
    (total_size, unused_size, total_count, unused_count)
}
