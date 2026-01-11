//! ArcBox daemon client service.
//!
//! Provides connection management and RPC client access to the arcbox-daemon.

use std::path::PathBuf;

use arcbox_api::generated::{
    container_service_client::ContainerServiceClient,
    image_service_client::ImageServiceClient,
    machine_service_client::MachineServiceClient,
    network_service_client::NetworkServiceClient,
    ListContainersRequest, ListContainersResponse,
    CreateContainerRequest, CreateContainerResponse,
    StartContainerRequest, StopContainerRequest, RemoveContainerRequest,
    ListImagesRequest, ListImagesResponse,
    ListMachinesRequest, ListMachinesResponse,
    ListNetworksRequest, ListNetworksResponse,
    CreateNetworkRequest, RemoveNetworkRequest,
    ContainerLogsRequest, LogEntry,
};
use futures::StreamExt;
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

    /// Get network service client
    pub fn network_client(&self) -> Option<NetworkServiceClient<Channel>> {
        self.channel.clone().map(NetworkServiceClient::new)
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

    /// Create a container
    pub fn create_container(
        &self,
        image: String,
        name: Option<String>,
        cmd: Option<Vec<String>>,
        entrypoint: Option<Vec<String>>,
        working_dir: Option<String>,
        start: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(mut client) = self.container_client() else {
            tracing::warn!("Not connected to daemon");
            cx.emit(DaemonEvent::OperationFailed("Not connected to daemon".to_string()));
            return;
        };
        let runtime = self.tokio_runtime.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let image_clone = image.clone();
            let result = cx.background_executor().spawn(async move {
                runtime.block_on(async {
                    let request = tonic::Request::new(CreateContainerRequest {
                        image: image_clone,
                        name: name.unwrap_or_default(),
                        cmd: cmd.unwrap_or_default(),
                        entrypoint: entrypoint.unwrap_or_default(),
                        working_dir: working_dir.unwrap_or_default(),
                        ..Default::default()
                    });
                    client.create_container(request).await
                })
            }).await;

            match result {
                Ok(response) => {
                    let container_id = response.into_inner().id;
                    tracing::info!("Created container {} from image {}", container_id, image);

                    cx.update(|cx| {
                        this.update(cx, |this, cx| {
                            cx.emit(DaemonEvent::ContainerCreated(container_id.clone()));

                            // Start if requested
                            if start {
                                this.start_container(container_id, cx);
                            } else {
                                // Just refresh the list
                                this.list_containers(true, cx);
                            }
                        })
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Failed to create container from {}: {}", image, e);
                    cx.update(|cx| {
                        this.update(cx, |_this, cx| {
                            cx.emit(DaemonEvent::OperationFailed(format!("Failed to create container: {}", e)));
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

    /// Subscribe to container logs (streaming)
    ///
    /// Emits `LogsReceived` events as log entries arrive.
    /// Returns immediately; logs are delivered asynchronously via events.
    pub fn subscribe_logs(
        &self,
        container_id: String,
        follow: bool,
        tail: Option<u32>,
        cx: &mut Context<Self>,
    ) {
        let Some(mut client) = self.container_client() else {
            tracing::warn!("Not connected to daemon");
            return;
        };
        let runtime = self.tokio_runtime.clone();
        let id = container_id.clone();

        tracing::info!("Subscribing to logs for container {}", container_id);

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let id_for_request = id.clone();
            let runtime_clone = runtime.clone();

            // Connect to log stream in background
            let result = cx.background_executor().spawn(async move {
                runtime_clone.block_on(async {
                    let request = tonic::Request::new(ContainerLogsRequest {
                        id: id_for_request,
                        follow,
                        stdout: true,
                        stderr: true,
                        timestamps: true,
                        since: 0,
                        until: 0,
                        tail: tail.map(i64::from).unwrap_or(100),
                    });
                    client.container_logs(request).await
                })
            }).await;

            match result {
                Ok(response) => {
                    tracing::debug!("Log stream started for container {}", id);

                    // Process stream entries in background using channels
                    let id_for_stream = id.clone();
                    let stream = response.into_inner();

                    // Use std channel for cross-thread communication
                    let (tx, rx) = std::sync::mpsc::channel();

                    // Spawn the stream processing in the tokio runtime
                    cx.background_executor().spawn({
                        let runtime = runtime.clone();
                        let id_for_bg = id_for_stream.clone();
                        async move {
                            runtime.block_on(async {
                                use futures::StreamExt as _;
                                let mut stream = stream;
                                while let Some(entry_result) = stream.next().await {
                                    match entry_result {
                                        Ok(log_entry) => {
                                            if tx.send(log_entry).is_err() {
                                                break; // Receiver dropped
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Log stream error for {}: {}", id_for_bg, e);
                                            break;
                                        }
                                    }
                                }
                                tracing::debug!("Log stream ended for {}", id_for_bg);
                            });
                        }
                    }).detach();

                    // Process received log entries from std channel using non-blocking recv
                    loop {
                        match rx.try_recv() {
                            Ok(log_entry) => {
                                let container_id = id_for_stream.clone();
                                cx.update(|cx| {
                                    this.update(cx, |_this, cx| {
                                        cx.emit(DaemonEvent::LogsReceived {
                                            container_id,
                                            entry: log_entry,
                                        });
                                    })
                                }).ok();
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => {
                                // No message available, yield and try again
                                cx.background_executor().timer(std::time::Duration::from_millis(10)).await;
                            }
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                // Channel closed, stream ended
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to subscribe to logs for {}: {}", id, e);
                }
            }
        }).detach();
    }

    /// List networks
    pub fn list_networks(&self, cx: &mut Context<Self>) {
        let Some(mut client) = self.network_client() else {
            tracing::warn!("Not connected to daemon");
            return;
        };
        let runtime = self.tokio_runtime.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let result = cx.background_executor().spawn(async move {
                runtime.block_on(async {
                    let request = tonic::Request::new(ListNetworksRequest::default());
                    client.list_networks(request).await
                })
            }).await;

            match result {
                Ok(response) => {
                    let networks = response.into_inner();
                    tracing::debug!("Got {} networks", networks.networks.len());
                    cx.update(|cx| {
                        this.update(cx, |_this, cx| {
                            cx.emit(DaemonEvent::NetworksLoaded(networks));
                        })
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Failed to list networks: {}", e);
                }
            }
        }).detach();
    }

    /// Create a network
    pub fn create_network(
        &self,
        name: String,
        driver: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(mut client) = self.network_client() else {
            tracing::warn!("Not connected to daemon");
            cx.emit(DaemonEvent::OperationFailed("Not connected to daemon".to_string()));
            return;
        };
        let runtime = self.tokio_runtime.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let name_clone = name.clone();
            let result = cx.background_executor().spawn(async move {
                runtime.block_on(async {
                    let request = tonic::Request::new(CreateNetworkRequest {
                        name: name_clone,
                        driver: driver.unwrap_or_default(),
                        internal: false,
                        labels: Default::default(),
                    });
                    client.create_network(request).await
                })
            }).await;

            match result {
                Ok(response) => {
                    let network_id = response.into_inner().id;
                    tracing::info!("Created network {} with id {}", name, network_id);
                    cx.update(|cx| {
                        this.update(cx, |this, cx| {
                            cx.emit(DaemonEvent::NetworkCreated(network_id));
                            // Refresh network list
                            this.list_networks(cx);
                        })
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Failed to create network {}: {}", name, e);
                    cx.update(|cx| {
                        this.update(cx, |_this, cx| {
                            cx.emit(DaemonEvent::OperationFailed(format!("Failed to create network: {}", e)));
                        })
                    }).ok();
                }
            }
        }).detach();
    }

    /// Remove a network
    pub fn remove_network(&self, id: String, cx: &mut Context<Self>) {
        let Some(mut client) = self.network_client() else {
            tracing::warn!("Not connected to daemon");
            cx.emit(DaemonEvent::OperationFailed("Not connected to daemon".to_string()));
            return;
        };
        let runtime = self.tokio_runtime.clone();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let id_clone = id.clone();
            let result = cx.background_executor().spawn(async move {
                runtime.block_on(async {
                    let request = tonic::Request::new(RemoveNetworkRequest { id: id_clone });
                    client.remove_network(request).await
                })
            }).await;

            match result {
                Ok(_) => {
                    tracing::info!("Removed network {}", id);
                    cx.update(|cx| {
                        this.update(cx, |this, cx| {
                            cx.emit(DaemonEvent::NetworkRemoved(id));
                            // Refresh network list
                            this.list_networks(cx);
                        })
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Failed to remove network {}: {}", id, e);
                    cx.update(|cx| {
                        this.update(cx, |_this, cx| {
                            cx.emit(DaemonEvent::OperationFailed(format!("Failed to remove network: {}", e)));
                        })
                    }).ok();
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
    NetworksLoaded(ListNetworksResponse),
    /// Container created successfully
    ContainerCreated(String),
    /// Container started successfully
    ContainerStarted(String),
    /// Container stopped successfully
    ContainerStopped(String),
    /// Container removed successfully
    ContainerRemoved(String),
    /// Network created successfully
    NetworkCreated(String),
    /// Network removed successfully
    NetworkRemoved(String),
    /// Operation failed with error message
    OperationFailed(String),
    /// Log entry received from container
    LogsReceived {
        container_id: String,
        entry: LogEntry,
    },
}

impl EventEmitter<DaemonEvent> for DaemonService {}
