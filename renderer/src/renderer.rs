use std::any::Any;
use std::cell::RefCell;
use std::rc::Weak;
use std::sync::Arc;

use retgui_primitives::Color;
use retgui_primitives::geometry::{Affine, BezPath, Circle, Rectangle, Shape};

use crate::render_command::{BoxShadowCmd, DrawBoxShadow, DrawCircleCmd, DrawCircleOutlineCmd, DrawImageCmd, DrawRectCmd, DrawRectOutlineCmd, DrawTextCmd, FillBezPathCmd, PushLayerCmd, StrokeBezPathCmd};
use crate::render_list::RenderList;
pub use crate::screenshot::Screenshot;

use crate::text_renderer_data::{TextRender, TextScroll};
use crate::{Brush, RenderCommand, TargetItem};
use retgui_resource_manager::{ResourceId, ResourceManager};

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
