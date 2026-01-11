use std::collections::HashMap;

use arcbox_api::generated::{PortBinding as ProtoPortBinding, ContainerSummary};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Container state representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerState {
    Running,
    Stopped,
    Restarting,
    Paused,
    Dead,
}

impl ContainerState {
    pub fn is_running(&self) -> bool {
        matches!(self, ContainerState::Running)
    }

    pub fn label(&self) -> &'static str {
        match self {
            ContainerState::Running => "Running",
            ContainerState::Stopped => "Stopped",
            ContainerState::Restarting => "Restarting",
            ContainerState::Paused => "Paused",
            ContainerState::Dead => "Dead",
        }
    }
}

/// Port mapping for container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

/// Container view model for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerViewModel {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: ContainerState,
    pub ports: Vec<PortMapping>,
    pub created_at: DateTime<Utc>,
    pub compose_project: Option<String>,
    pub labels: HashMap<String, String>,
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub memory_limit_mb: f64,
}

impl ContainerViewModel {
    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    /// Display ports as string like "8080:80, 443:443"
    pub fn ports_display(&self) -> String {
        if self.ports.is_empty() {
            return "-".to_string();
        }
        self.ports
            .iter()
            .map(|p| format!("{}:{}", p.host_port, p.container_port))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Display relative time since creation
    pub fn created_ago(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.created_at);

        if duration.num_days() > 0 {
            format!("{}d ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{}h ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{}m ago", duration.num_minutes())
        } else {
            "just now".to_string()
        }
    }
}

impl From<&ProtoPortBinding> for PortMapping {
    fn from(pb: &ProtoPortBinding) -> Self {
        Self {
            host_port: pb.host_port as u16,
            container_port: pb.container_port as u16,
            protocol: pb.protocol.clone(),
        }
    }
}

impl From<ContainerSummary> for ContainerViewModel {
    fn from(summary: ContainerSummary) -> Self {
        // Parse state from string
        let state = match summary.state.as_str() {
            "running" => ContainerState::Running,
            "paused" => ContainerState::Paused,
            "restarting" => ContainerState::Restarting,
            "dead" => ContainerState::Dead,
            _ => ContainerState::Stopped,
        };

        // Extract container name (remove leading /)
        let name = if summary.name.is_empty() {
            summary.id.chars().take(12).collect()
        } else {
            summary.name.trim_start_matches('/').to_string()
        };

        // Parse created timestamp (Unix seconds)
        let created_at = Utc.timestamp_opt(summary.created, 0).single().unwrap_or_else(Utc::now);

        // Check for compose project label
        let compose_project = summary.labels.get("com.docker.compose.project").cloned();

        Self {
            id: summary.id,
            name,
            image: summary.image,
            state,
            ports: summary.ports.iter().map(PortMapping::from).collect(),
            created_at,
            compose_project,
            labels: summary.labels,
            // Resource usage requires separate stats call, default to 0
            cpu_percent: 0.0,
            memory_mb: 0.0,
            memory_limit_mb: 0.0,
        }
    }
}

