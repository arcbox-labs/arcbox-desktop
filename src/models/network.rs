use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Network view model for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkViewModel {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub created_at: DateTime<Utc>,
    pub internal: bool,
    pub attachable: bool,
    pub container_count: usize,
}

impl NetworkViewModel {
    /// Display short ID (first 12 chars)
    pub fn short_id(&self) -> String {
        self.id.chars().take(12).collect()
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

    /// Display driver info with scope
    pub fn driver_display(&self) -> String {
        format!("{} ({})", self.driver, self.scope)
    }

    /// Display usage status
    pub fn usage_display(&self) -> String {
        if self.container_count == 0 {
            "No containers".to_string()
        } else if self.container_count == 1 {
            "1 container".to_string()
        } else {
            format!("{} containers", self.container_count)
        }
    }

    /// Check if this is a default/system network
    pub fn is_system(&self) -> bool {
        matches!(self.name.as_str(), "bridge" | "host" | "none")
    }
}
