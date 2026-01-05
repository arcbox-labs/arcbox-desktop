use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Machine state representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineState {
    Running,
    Stopped,
    Starting,
    Stopping,
}

impl MachineState {
    pub fn is_running(&self) -> bool {
        matches!(self, MachineState::Running)
    }

    pub fn label(&self) -> &'static str {
        match self {
            MachineState::Running => "Running",
            MachineState::Stopped => "Stopped",
            MachineState::Starting => "Starting",
            MachineState::Stopping => "Stopping",
        }
    }
}

/// Linux distribution info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroInfo {
    pub name: String,
    pub version: String,
    pub display_name: String,
}

/// Machine view model for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineViewModel {
    pub id: String,
    pub name: String,
    pub distro: DistroInfo,
    pub state: MachineState,
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub disk_gb: u32,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl MachineViewModel {
    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    pub fn resources_display(&self) -> String {
        format!(
            "{} cores, {} GB RAM, {} GB disk",
            self.cpu_cores, self.memory_gb, self.disk_gb
        )
    }
}

/// Create dummy machines for development
pub fn dummy_machines() -> Vec<MachineViewModel> {
    use chrono::Duration;
    let now = Utc::now();

    vec![
        MachineViewModel {
            id: "machine-ubuntu-dev".to_string(),
            name: "ubuntu-dev".to_string(),
            distro: DistroInfo {
                name: "ubuntu".to_string(),
                version: "24.04".to_string(),
                display_name: "Ubuntu 24.04 LTS".to_string(),
            },
            state: MachineState::Running,
            cpu_cores: 4,
            memory_gb: 4,
            disk_gb: 20,
            ip_address: Some("198.19.249.2".to_string()),
            created_at: now - Duration::days(7),
        },
        MachineViewModel {
            id: "machine-debian-test".to_string(),
            name: "debian-test".to_string(),
            distro: DistroInfo {
                name: "debian".to_string(),
                version: "12".to_string(),
                display_name: "Debian 12 (Bookworm)".to_string(),
            },
            state: MachineState::Stopped,
            cpu_cores: 2,
            memory_gb: 2,
            disk_gb: 10,
            ip_address: None,
            created_at: now - Duration::days(14),
        },
        MachineViewModel {
            id: "machine-fedora-playground".to_string(),
            name: "fedora-playground".to_string(),
            distro: DistroInfo {
                name: "fedora".to_string(),
                version: "40".to_string(),
                display_name: "Fedora 40".to_string(),
            },
            state: MachineState::Running,
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 30,
            ip_address: Some("198.19.249.3".to_string()),
            created_at: now - Duration::days(3),
        },
    ]
}
