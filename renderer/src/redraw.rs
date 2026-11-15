use std::sync::Arc;

use winit::window::Window;

pub(crate) fn request_redraw(window: &Arc<dyn Window>) {
    #[cfg(target_os = "macos")]
    {
        let window = Arc::clone(window);
        dispatch2::DispatchQueue::main().exec_async(move || window.request_redraw());
    }
    #[cfg(not(target_os = "macos"))]
    window.request_redraw();
}
