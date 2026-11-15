use winit::window::Window;

#[cfg(target_os = "macos")]
mod macos;

pub(crate) fn synchronize_presentation(window: Option<&dyn Window>) -> bool {
    #[cfg(target_os = "macos")]
    return window.is_some_and(macos::is_live_resize);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
        false
    }
}
