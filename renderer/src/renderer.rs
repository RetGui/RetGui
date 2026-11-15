use std::any::Any;
use std::sync::Arc;

use retgui_primitives::Color;
use retgui_primitives::geometry::Rectangle;

use crate::render_list::RenderList;
pub use crate::screenshot::Screenshot;

use retgui_resource_manager::ResourceManager;

pub trait Renderer: Any {
    #[allow(dead_code)]
    fn surface_width(&self) -> f32;

    #[allow(dead_code)]
    fn surface_height(&self) -> f32;

    fn resize_surface(&mut self, width: f32, height: f32);

    fn surface_set_clear_color(&mut self, color: Color);

    fn set_vsync(&mut self, _enabled: bool) {}
    
    fn prepare(&mut self, render_list: &mut RenderList, resource_manager: Arc<ResourceManager>, window: Rectangle);

    fn submit(&mut self);

    fn screenshot(&self) -> Screenshot {
        Screenshot {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        }
    }
}
