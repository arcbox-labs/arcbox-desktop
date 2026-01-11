//! ArcBox daemon client service.
//!
//! Provides connection management and RPC client access to the arcbox-daemon.

use std::path::PathBuf;

use arcbox_api::generated::{
    container_service_client::ContainerServiceClient,
    image_service_client::ImageServiceClient,
    machine_service_client::MachineServiceClient,
    ListContainersRequest, ListContainersResponse,
    StartContainerRequest, StopContainerRequest, RemoveContainerRequest,
    ListImagesRequest, ListImagesResponse,
    ListMachinesRequest, ListMachinesResponse,
};
use gpui::*;
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// ArcBox daemon client service
///
/// Manages connection to the daemon and provides access to gRPC clients.
pub struct DaemonService {
    /// Current connection state
    state: ConnectionState,
    /// gRPC channel (when connected)
    channel: Option<Channel>,
    /// Socket path
    socket_path: PathBuf,
    /// Tokio runtime for gRPC operations
    tokio_runtime: std::sync::Arc<tokio::runtime::Runtime>,
}

impl DaemonService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        // Default to user directory socket path
        let socket_path = Self::default_socket_path();
        let tokio_runtime = std::sync::Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime")
        );
        Self {
            state: ConnectionState::Disconnected,
            channel: None,
            socket_path,
            tokio_runtime,
        }
    }

    /// Create with custom socket path
    pub fn with_socket_path(socket_path: PathBuf, _cx: &mut Context<Self>) -> Self {
        let tokio_runtime = std::sync::Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime")
        );
        Self {
            state: ConnectionState::Disconnected,
            channel: None,
            socket_path,
            tokio_runtime,
        }
    }

    /// Get default socket path (matches daemon default: ~/.arcbox/arcbox.sock)
    fn default_socket_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".arcbox")
            .join("arcbox.sock")
    }

    /// Set socket path (must be called before connect)
    pub fn set_socket_path(&mut self, path: PathBuf) {
        self.socket_path = path;
    }

    /// Get current connection state
    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    /// Connect to the daemon
    pub fn connect(&mut self, cx: &mut Context<Self>) {
        if matches!(self.state, ConnectionState::Connecting | ConnectionState::Connected) {
            return;
        }

        self.state = ConnectionState::Connecting;
        cx.notify();

        let socket_path = self.socket_path.clone();
        let runtime = self.tokio_runtime.clone();

        // Use background executor for the connection
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let socket_path_str = socket_path.display().to_string();

            // Run the tokio-dependent code in background thread with our runtime
            let result = cx
                .background_executor()
                .spawn(async move {
                    Self::create_channel_with_runtime(&socket_path, &runtime)
                })
                .await;

            cx.update(|cx| {
                this.update(cx, |this, cx| {
                    match result {
                        Ok(channel) => {
                            tracing::info!("Connected to daemon at {}", socket_path_str);
                            this.channel = Some(channel);
                            this.state = ConnectionState::Connected;
                        }
                        Err(e) => {
                            tracing::error!("Failed to connect to daemon: {}", e);
                            this.state = ConnectionState::Error(e);
                        }
                    }
                    cx.notify();
                })
            }).ok();
        }).detach();
    }

    /// Create gRPC channel over Unix socket using provided runtime
    fn create_channel_with_runtime(
        socket_path: &PathBuf,
        runtime: &tokio::runtime::Runtime,
    ) -> Result<Channel, String> {
        use std::os::unix::net::UnixStream as StdUnixStream;

        let socket_path_str = socket_path.to_string_lossy().to_string();

        // First verify the socket exists and is connectable
        if StdUnixStream::connect(&socket_path_str).is_err() {
            return Err(format!("Cannot connect to socket: {}", socket_path_str));
        }

        // Use the provided runtime
        runtime.block_on(async move {
            let channel = Endpoint::try_from("http://[::]:50051")
                .map_err(|e| format!("Invalid endpoint: {}", e))?
                .connect_with_connector(service_fn(move |_: Uri| {
                    let path = socket_path_str.clone();
                    async move {
                        UnixStream::connect(path).await.map(|s| {
                            hyper_util::rt::TokioIo::new(s)
                        })
                    }
                }))
                .await
                .map_err(|e| format!("Failed to connect: {}", e))?;

            Ok(channel)
        })
    }

    /// Disconnect from the daemon
    pub fn disconnect(&mut self, cx: &mut Context<Self>) {
        self.channel = None;
        self.state = ConnectionState::Disconnected;
        cx.notify();
    }

    /// Get machine service client
    pub fn machine_client(&self) -> Option<MachineServiceClient<Channel>> {
        self.channel.clone().map(MachineServiceClient::new)
    }

    /// Get container service client
    pub fn container_client(&self) -> Option<ContainerServiceClient<Channel>> {
        self.channel.clone().map(ContainerServiceClient::new)
    }

    /// Get image service client
    pub fn image_client(&self) -> Option<ImageServiceClient<Channel>> {
        self.channel.clone().map(ImageServiceClient::new)
    }

    /// List machines
    pub fn list_machines(&self, cx: &mut Context<Self>) {
        let Some(mut client) = self.machine_client() else {
            tracing::warn!("Not connected to daemon");
            return;
        };
        let runtime = self.tokio_runtime.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let result = cx.background_executor().spawn(async move {
                runtime.block_on(async {
                    let request = tonic::Request::new(ListMachinesRequest { all: true });
                    client.list_machines(request).await
                })
            }).await;

            match result {
                Ok(response) => {
                    let machines = response.into_inner();
                    tracing::debug!("Got {} machines", machines.machines.len());
                    cx.update(|cx| {
                        this.update(cx, |_this, cx| {
                            cx.emit(DaemonEvent::MachinesLoaded(machines));
                        })
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Failed to list machines: {}", e);
                }
            }
        }).detach();
    }

    /// List containers
    pub fn list_containers(&self, all: bool, cx: &mut Context<Self>) {
        let Some(mut client) = self.container_client() else {
            tracing::warn!("Not connected to daemon");
            return;
        };
        let runtime = self.tokio_runtime.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let result = cx.background_executor().spawn(async move {
                runtime.block_on(async {
                    let request = tonic::Request::new(ListContainersRequest {
                        all,
                        limit: 0,
                        filters: Default::default(),
                    });
                    client.list_containers(request).await
                })
            }).await;

            match result {
                Ok(response) => {
                    let containers = response.into_inner();
                    tracing::debug!("Got {} containers", containers.containers.len());
                    cx.update(|cx| {
                        this.update(cx, |_this, cx| {
                            cx.emit(DaemonEvent::ContainersLoaded(containers));
                        })
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Failed to list containers: {}", e);
                }
            }
        }).detach();
    }

    /// Start a container
    pub fn start_container(&self, id: String, cx: &mut Context<Self>) {
        let Some(mut client) = self.container_client() else {
            tracing::warn!("Not connected to daemon");
            return;
        };
        let runtime = self.tokio_runtime.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let id_clone = id.clone();
            let result = cx.background_executor().spawn(async move {
                runtime.block_on(async {
                    let request = tonic::Request::new(StartContainerRequest { id: id_clone });
                    client.start_container(request).await
                })
            }).await;

            match result {
                Ok(_) => {
                    tracing::info!("Started container {}", id);
                    cx.update(|cx| {
                        this.update(cx, |this, cx| {
                            cx.emit(DaemonEvent::ContainerStarted(id));
                            // Refresh container list
                            this.list_containers(true, cx);
                        })
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Failed to start container {}: {}", id, e);
                    cx.update(|cx| {
                        this.update(cx, |_this, cx| {
                            cx.emit(DaemonEvent::OperationFailed(format!("Failed to start container: {}", e)));
                        })
                    }).ok();
                }
            }
        }).detach();
    }

    /// Stop a container
    pub fn stop_container(&self, id: String, timeout: u32, cx: &mut Context<Self>) {
        let Some(mut client) = self.container_client() else {
            tracing::warn!("Not connected to daemon");
            return;
        };
        let runtime = self.tokio_runtime.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let id_clone = id.clone();
            let result = cx.background_executor().spawn(async move {
                runtime.block_on(async {
                    let request = tonic::Request::new(StopContainerRequest { id: id_clone, timeout });
                    client.stop_container(request).await
                })
            }).await;

            match result {
                Ok(_) => {
                    tracing::info!("Stopped container {}", id);
                    cx.update(|cx| {
                        this.update(cx, |this, cx| {
                            cx.emit(DaemonEvent::ContainerStopped(id));
                            // Refresh container list
                            this.list_containers(true, cx);
                        })
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Failed to stop container {}: {}", id, e);
                    cx.update(|cx| {
                        this.update(cx, |_this, cx| {
                            cx.emit(DaemonEvent::OperationFailed(format!("Failed to stop container: {}", e)));
                        })
                    }).ok();
                }
            }
        }).detach();
    }

    /// Remove a container
    pub fn remove_container(&self, id: String, force: bool, cx: &mut Context<Self>) {
        let Some(mut client) = self.container_client() else {
            tracing::warn!("Not connected to daemon");
            return;
        };
        let runtime = self.tokio_runtime.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let id_clone = id.clone();
            let result = cx.background_executor().spawn(async move {
                runtime.block_on(async {
                    let request = tonic::Request::new(RemoveContainerRequest {
                        id: id_clone,
                        force,
                        remove_volumes: false,
                    });
                    client.remove_container(request).await
                })
            }).await;

            match result {
                Ok(_) => {
                    tracing::info!("Removed container {}", id);
                    cx.update(|cx| {
                        this.update(cx, |this, cx| {
                            cx.emit(DaemonEvent::ContainerRemoved(id));
                            // Refresh container list
                            this.list_containers(true, cx);
                        })
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Failed to remove container {}: {}", id, e);
                    cx.update(|cx| {
                        this.update(cx, |_this, cx| {
                            cx.emit(DaemonEvent::OperationFailed(format!("Failed to remove container: {}", e)));
                        })
                    }).ok();
                }
            }
        }).detach();
    }

    /// List images
    pub fn list_images(&self, cx: &mut Context<Self>) {
        let Some(mut client) = self.image_client() else {
            tracing::warn!("Not connected to daemon");
            return;
        };
        let runtime = self.tokio_runtime.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let result = cx.background_executor().spawn(async move {
                runtime.block_on(async {
                    let request = tonic::Request::new(ListImagesRequest::default());
                    client.list_images(request).await
                })
            }).await;

            match result {
                Ok(response) => {
                    let images = response.into_inner();
                    tracing::debug!("Got {} images", images.images.len());
                    cx.update(|cx| {
                        this.update(cx, |_this, cx| {
                            cx.emit(DaemonEvent::ImagesLoaded(images));
                        })
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Failed to list images: {}", e);
                }
            }
        }).detach();
    }
}

/// Events emitted by DaemonService
#[derive(Debug, Clone)]
pub enum DaemonEvent {
    MachinesLoaded(ListMachinesResponse),
    ContainersLoaded(ListContainersResponse),
    ImagesLoaded(ListImagesResponse),
    /// Container started successfully
    ContainerStarted(String),
    /// Container stopped successfully
    ContainerStopped(String),
    /// Container removed successfully
    ContainerRemoved(String),
    /// Operation failed with error message
    OperationFailed(String),
}

impl EventEmitter<DaemonEvent> for DaemonService {}
