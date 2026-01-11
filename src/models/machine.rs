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

