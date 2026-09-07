//! A retained GUI.

pub use image;

pub use retgui_primitives::brush::Brush;
pub use retgui_primitives::{Color, ColorStop, Extend, Gradient, GradientKind, HueDirection, LinearGradientData, RadialGradientData, SweepGradientData, geometry, palette};

pub use retgui_renderer::RendererType;
pub use retgui_renderer::renderer::Renderer;

pub use retgui_resource_manager::resource_type::ResourceType;
pub use retgui_resource_manager::{ResourceError, ResourceId, ResourceManager};

pub use retgui_runtime::{self, RetGuiRuntime};

pub use winit;

pub use crate::app::{App, WindowEventResult};
pub use crate::elements::States;
pub use crate::options::RetGuiOptions;
pub use crate::utils::retgui_error::RetGuiError;
pub use crate::utils::style_helpers::{auto, pct, px, rgb, rgba};

#[cfg(target_os = "android")]
use std::sync::OnceLock;

use retgui_logging::info;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use crate::drivers::Driver;
use crate::drivers::winit::WinitDriver;

pub mod drivers;
pub mod elements;
pub mod events;
pub mod layout;
pub mod style;
pub mod text;

mod accessibility;
mod app;
mod options;
mod perf_stats;
mod utils;
mod window_manager;

#[cfg(target_os = "android")]
static ANDROID_APP: OnceLock<AndroidApp> = OnceLock::new();

/// Starts the RetGui application.
///
/// This will block the current thread until all [`Window`](elements::Window) instances have been closed.
/// On Android, [`retgui_set_android_app`] must be called prior.
///
/// # Example
///
/// ```no_run
/// use retgui::elements::Window;
/// use retgui::{App, States, RetGuiOptions, retgui_main};
///
/// fn main() {
///     let mut app = App::new();
///     let states = States::new();
///     Window::new(&mut app, "RetGui");
///     retgui_main(app, states, RetGuiOptions::default());
/// }
/// ```
pub fn retgui_main(app: App, states: States, options: RetGuiOptions) {
    retgui_main_with_driver::<WinitDriver>(app, states, options);
}

/// Starts the RetGui application using a custom [`Driver`].
///
/// RetGui constructs `D` through [`Driver::new`] and
/// transfers ownership of the app and state store to the driver. This is the advanced
/// counterpart to [`retgui_main`]; most applications should use the default
/// winit driver.
///
/// # Example
///
/// ```no_run
/// use retgui::drivers::Driver;
/// use retgui::{App, States, RetGuiOptions, retgui_main_with_driver};
///
/// struct PlatformDriver {
///     app: App,
///     states: States,
/// }
///
/// impl Driver for PlatformDriver {
///     fn new(app: App, states: States) -> Self {
///         Self { app, states }
///     }
///
///     fn run(mut self) {
///         self.app.on_resume(None);
///         while !self.app.close_requested() {
///             let _ = self.app.on_about_to_wait(None, &mut self.states);
///             // Wait for and forward native events here.
///             break;
///         }
///     }
/// }
///
/// retgui_main_with_driver::<PlatformDriver>(App::new(), States::new(), RetGuiOptions::default());
/// ```
pub fn retgui_main_with_driver<D>(app: App, states: States, options: RetGuiOptions)
where
    D: Driver,
{
    info!("RetGui started: {}", options.app_name);
    D::new(app, states).run();
}

/// Sets the [`AndroidApp`] for retgui to use.
#[cfg(target_os = "android")]
pub fn retgui_set_android_app(app: AndroidApp) {
    ANDROID_APP
        .set(app)
        .expect("retgui_set_android_app may only be called once");
}
