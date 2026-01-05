use chrono::{DateTime, Utc};
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

/// Create dummy containers for development
pub fn dummy_containers() -> Vec<ContainerViewModel> {
    use chrono::Duration;
    let now = Utc::now();

    vec![
        ContainerViewModel {
            id: "a1b2c3d4e5f6".to_string(),
            name: "nginx".to_string(),
            image: "nginx:latest".to_string(),
            state: ContainerState::Running,
            ports: vec![
                PortMapping {
                    host_port: 8080,
                    container_port: 80,
                    protocol: "tcp".to_string(),
                },
                PortMapping {
                    host_port: 443,
                    container_port: 443,
                    protocol: "tcp".to_string(),
                },
            ],
            created_at: now - Duration::hours(2),
            compose_project: Some("my-project".to_string()),
            cpu_percent: 0.5,
            memory_mb: 25.4,
            memory_limit_mb: 512.0,
        },
        ContainerViewModel {
            id: "b2c3d4e5f6g7".to_string(),
            name: "postgres".to_string(),
            image: "postgres:15".to_string(),
            state: ContainerState::Running,
            ports: vec![PortMapping {
                host_port: 5432,
                container_port: 5432,
                protocol: "tcp".to_string(),
            }],
            created_at: now - Duration::days(1),
            compose_project: Some("my-project".to_string()),
            cpu_percent: 1.2,
            memory_mb: 128.5,
            memory_limit_mb: 1024.0,
        },
        ContainerViewModel {
            id: "c3d4e5f6g7h8".to_string(),
            name: "redis".to_string(),
            image: "redis:alpine".to_string(),
            state: ContainerState::Running,
            ports: vec![PortMapping {
                host_port: 6379,
                container_port: 6379,
                protocol: "tcp".to_string(),
            }],
            created_at: now - Duration::hours(3),
            compose_project: Some("my-project".to_string()),
            cpu_percent: 0.1,
            memory_mb: 12.3,
            memory_limit_mb: 256.0,
        },
        ContainerViewModel {
            id: "d4e5f6g7h8i9".to_string(),
            name: "my-app".to_string(),
            image: "my-app:dev".to_string(),
            state: ContainerState::Running,
            ports: vec![PortMapping {
                host_port: 3000,
                container_port: 3000,
                protocol: "tcp".to_string(),
            }],
            created_at: now - Duration::minutes(5),
            compose_project: None,
            cpu_percent: 2.5,
            memory_mb: 256.0,
            memory_limit_mb: 512.0,
        },
        ContainerViewModel {
            id: "e5f6g7h8i9j0".to_string(),
            name: "old-service".to_string(),
            image: "node:18".to_string(),
            state: ContainerState::Stopped,
            ports: vec![],
            created_at: now - Duration::days(2),
            compose_project: None,
            cpu_percent: 0.0,
            memory_mb: 0.0,
            memory_limit_mb: 512.0,
        },
    ]
}
