#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

use retgui_renderer::RenderList;
#[cfg(not(target_arch = "wasm32"))]
use retgui_renderer::renderer::Renderer;

#[cfg(not(target_arch = "wasm32"))]
use retgui_resource_manager::ResourceManager;

use retgui_runtime::sync::Mutex;

#[cfg(not(target_arch = "wasm32"))]
use crate::perf_stats::FrameStats;

pub(crate) struct RenderBuffers {
    slots: Mutex<[(bool, Option<RenderList>); 2]>,
}

impl RenderBuffers {
    pub(crate) fn new() -> Self {
        Self {
            slots: Mutex::new(std::array::from_fn(|_| (true, Some(RenderList::new())))),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn acquire(&self) -> Option<(usize, RenderList)> {
        let mut slots = self.slots.try_lock().ok()?;
        let index = slots.iter().position(|(available, _)| *available)?;
        slots[index].0 = false;
        Some((index, slots[index].1.take().unwrap()))
    }

    pub(crate) fn has_available(&self) -> bool {
        self.slots
            .try_lock()
            .is_ok_and(|slots| slots.iter().any(|(available, _)| *available))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn is_idle(&self) -> bool {
        self.slots.lock().await.iter().all(|(available, _)| *available)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn queue(&self, index: usize, list: RenderList) {
        let mut slots = self.slots.lock().await;
        slots[index].1 = Some(list);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn take_queued(&self, index: usize) -> RenderList {
        let mut slots = self.slots.lock().await;
        slots[index].1.take().unwrap()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn recycle(&self, index: usize, mut list: RenderList) {
        list.clear();
        self.slots.lock().await[index] = (true, Some(list));
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub enum RenderMessage {
    /// Wake the event loop when presentation work is ready.
    SetEventLoopProxy(winit::event_loop::EventLoopProxy),
    /// Set the render thread's renderer.
    SetRenderer(Box<dyn Renderer + Send>),
    /// Set the render thread's resource manager.
    SetResourceManager(Arc<ResourceManager>),
    /// Render a frame.
    DoRender(usize, FrameRequest, retgui_primitives::geometry::Size<f32>),
    /// Control Vsync.
    SetVsync(bool),
    /// Take a screenshot.
    Screenshot(retgui_runtime::sync::oneshot::Sender<retgui_renderer::Screenshot>),
    /// Close the render thread.
    Close,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct FrameRequest {
    pub(crate) stats: FrameStats,
    pub(crate) synchronize_with_window: bool,
}

pub(crate) enum RenderCompletion {
    #[cfg(not(target_arch = "wasm32"))]
    Frame(FrameStats),
    #[cfg(not(target_arch = "wasm32"))]
    Present {
        present: retgui_renderer::renderer::Presentation,
        finished: retgui_runtime::sync::oneshot::Sender<()>,
    },
}
