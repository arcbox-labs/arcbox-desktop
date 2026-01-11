//! Tokio bridge for GPUI - allows running tokio async tasks within GPUI context.
//! Based on gpui_tokio from zed.

use std::future::Future;

use gpui::{App, AppContext, Global, Task};

pub use tokio::task::JoinError;

/// Initializes the Tokio wrapper using a new Tokio runtime with 2 worker threads.
pub fn init(cx: &mut App) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio");

    cx.set_global(GlobalTokio::new(RuntimeHolder::Owned(runtime)));
}

enum RuntimeHolder {
    Owned(tokio::runtime::Runtime),
    #[allow(dead_code)]
    Shared(tokio::runtime::Handle),
}

impl RuntimeHolder {
    pub fn handle(&self) -> &tokio::runtime::Handle {
        match self {
            RuntimeHolder::Owned(runtime) => runtime.handle(),
            RuntimeHolder::Shared(handle) => handle,
        }
    }
}

struct GlobalTokio {
    runtime: RuntimeHolder,
}

impl Global for GlobalTokio {}

impl GlobalTokio {
    fn new(runtime: RuntimeHolder) -> Self {
        Self { runtime }
    }
}

/// Helper to abort tokio task when dropped
struct AbortOnDrop(tokio::task::AbortHandle);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub struct Tokio;

impl Tokio {
    /// Spawns the given future on Tokio's thread pool, and returns it via a GPUI task.
    /// Note that the Tokio task will be cancelled if the GPUI task is dropped.
    pub fn spawn<T, Fut, R>(cx: &mut gpui::Context<'_, T>, f: Fut) -> Task<Result<R, JoinError>>
    where
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        let tokio = cx.global::<GlobalTokio>();
        let join_handle = tokio.runtime.handle().spawn(f);
        let abort_guard = AbortOnDrop(join_handle.abort_handle());
        cx.background_spawn(async move {
            let result = join_handle.await;
            drop(abort_guard);
            result
        })
    }
}
