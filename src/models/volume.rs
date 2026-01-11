use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Volume view model for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeViewModel {
    pub name: String,
    pub driver: String,
    pub mount_point: String,
    pub size_bytes: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub in_use: bool,
    pub container_names: Vec<String>,
}

impl VolumeViewModel {
    /// Display size in human readable format
    pub fn size_display(&self) -> String {
        match self.size_bytes {
            Some(bytes) => {
                let mb = bytes as f64 / 1_000_000.0;
                if mb >= 1000.0 {
                    format!("{:.1} GB", mb / 1000.0)
                } else if mb >= 1.0 {
                    format!("{:.0} MB", mb)
                } else {
                    format!("{:.0} KB", bytes as f64 / 1000.0)
                }
            }
            None => "N/A".to_string(),
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

    /// Display usage status
    pub fn usage_display(&self) -> String {
        if self.in_use {
            if self.container_names.len() == 1 {
                format!("Used by {}", self.container_names[0])
            } else {
                format!("Used by {} containers", self.container_names.len())
            }
        } else {
            "Unused".to_string()
        }
    }
}

