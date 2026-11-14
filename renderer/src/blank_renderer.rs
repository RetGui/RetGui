use std::sync::Arc;

use retgui_primitives::Color;
use retgui_primitives::geometry::Rectangle;
use retgui_resource_manager::ResourceManager;

use crate::render_list::RenderList;
use crate::renderer::Renderer;

pub struct BlankRenderer {
    render_list: RenderList,
    width: f32,
    height: f32,
}

impl Default for BlankRenderer {
    fn default() -> Self {
        Self {
            render_list: RenderList::default(),
            width: 0.0,
            height: 0.0,
        }
    }
}

impl Renderer for BlankRenderer {
    fn surface_width(&self) -> f32 {
        self.width
    }

    fn surface_height(&self) -> f32 {
        self.height
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    fn surface_set_clear_color(&mut self, _color: Color) {}


    fn prepare(&mut self, _render_list: &mut RenderList, _resource_manager: Arc<ResourceManager>, _window: Rectangle) {}

    fn submit(&mut self) {}
}
