//! Platform-specific presentation.

use super::render_context::RenderSurface;

use crate::renderer::Presentation;

#[derive(Default)]
pub(super) struct WindowPresentation {
    #[cfg(target_os = "macos")]
    synchronized: bool,
    #[cfg(target_os = "macos")]
    pending: Option<Presentation>,
}

impl WindowPresentation {
    pub(super) fn begin_frame(&mut self, synchronize_with_window: bool) {
        #[cfg(target_os = "macos")]
        {
            self.synchronized = synchronize_with_window;
        }
        #[cfg(not(target_os = "macos"))]
        let _ = synchronize_with_window;
    }

    pub(super) fn configure(&mut self, surface: &RenderSurface<'_>) {
        #[cfg(target_os = "macos")]
        // SAFETY: the HAL guard keeps the surface alive; only presentation
        // policy is changed. wgpu retains ownership of all GPU resources.
        if let Some(metal) = unsafe { surface.surface.as_hal::<wgpu::hal::api::Metal>() } {
            let layer = metal.render_layer().lock();
            if layer.presentsWithTransaction() != self.synchronized {
                layer.setPresentsWithTransaction(self.synchronized);
            }
        } else {
            self.synchronized = false;
        }
        #[cfg(not(target_os = "macos"))]
        let _ = surface;
    }

    pub(super) fn present(&mut self, queue: &wgpu::Queue, texture: wgpu::SurfaceTexture) {
        #[cfg(target_os = "macos")]
        if self.synchronized {
            let queue = queue.clone();
            self.pending = Some(Box::new(move || queue.present(texture)));
            return;
        }
        queue.present(texture);
    }

    pub(super) fn finish_frame(&mut self) -> Option<Presentation> {
        #[cfg(target_os = "macos")]
        {
            self.synchronized = false;
            return self.pending.take();
        }
        #[cfg(not(target_os = "macos"))]
        None
    }
}
