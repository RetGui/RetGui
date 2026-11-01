use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::Mutex;

use winit::window::Window;

use crate::blank_renderer::BlankRenderer;
use crate::renderer::Renderer;
use crate::SharedRenderer;
#[cfg(feature = "vello_cpu_renderer")]
use crate::vello_cpu::VelloCpuRenderer;
#[cfg(feature = "vello_gpu_renderer")]
use crate::vello_gpu::VelloHybridRenderer;

/// An enumeration of the available renderer types for RetGui.
///
/// Depending on compile-time features, different renderers can be enabled.
/// When the `vello_gpu_renderer` feature is enabled, the [`VelloHybrid`](RendererType::VelloHybrid)
/// variant is available; otherwise, the [`Blank`](RendererType::Blank) variant is used.
#[derive(Copy, Clone, Debug)]
pub enum RendererType {
    #[cfg(feature = "vello_cpu_renderer")]
    VelloCPU,
    #[cfg(feature = "vello_gpu_renderer")]
    VelloHybrid,
    Blank,
}

#[allow(clippy::derivable_impls)]
impl Default for RendererType {
    fn default() -> Self {
        cfg_select! {
            feature = "vello_gpu_renderer" => RendererType::VelloHybrid,
            feature = "vello_cpu_renderer" => RendererType::VelloCPU,
            _ => RendererType::Blank,
        }
    }
}

impl Display for RendererType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "vello_cpu_renderer")]
            RendererType::VelloCPU => write!(f, "vello/cpu"),
            #[cfg(feature = "vello_gpu_renderer")]
            RendererType::VelloHybrid => write!(f, "vello/hybrid"),
            RendererType::Blank => write!(f, "blank"),
        }
    }
}

impl RendererType {
    pub async fn create(&self, window: Arc<dyn Window>) -> SharedRenderer {
        match self {
            #[cfg(feature = "vello_cpu_renderer")]
            RendererType::VelloCPU => Arc::new(Mutex::new(VelloCpuRenderer::new(window))),
            #[cfg(feature = "vello_gpu_renderer")]
            RendererType::VelloHybrid => Arc::new(Mutex::new(VelloHybridRenderer::new(window).await)),
            RendererType::Blank => {
                // So the linter does not complain about window being unused.
                let _ = window;
                Arc::new(Mutex::new(BlankRenderer::default()))
            }
        }
    }

    pub fn create_headless(&self, width: f32, height: f32) -> SharedRenderer {
        #[allow(unused_mut)]
        let mut blank = || -> SharedRenderer {
            let mut renderer = BlankRenderer::default();
            renderer.resize_surface(width, height);
            Arc::new(Mutex::new(renderer))
        };

        match self {
            #[cfg(feature = "vello_cpu_renderer")]
            RendererType::VelloCPU => Arc::new(Mutex::new(VelloCpuRenderer::new_headless(
                width.max(1.0) as u16,
                height.max(1.0) as u16,
            ))),
            #[cfg(feature = "vello_gpu_renderer")]
            RendererType::VelloHybrid => {
                #[cfg(feature = "vello_cpu_renderer")]
                {
                    Arc::new(Mutex::new(VelloCpuRenderer::new_headless(
                        width.max(1.0) as u16,
                        height.max(1.0) as u16,
                    )))
                }
                #[cfg(not(feature = "vello_cpu_renderer"))]
                {
                    blank()
                }
            }
            RendererType::Blank => blank(),
        }
    }
}
