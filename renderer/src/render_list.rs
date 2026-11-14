use crate::render_command::{BoxShadowCmd, DrawBoxShadow, DrawCircleCmd, DrawCircleOutlineCmd, DrawImageCmd, DrawRectCmd, DrawRectOutlineCmd, DrawTextCmd, FillBezPathCmd, PushLayerCmd, StrokeBezPathCmd};
use crate::sort_commands::{SortedCommands, SortedItem};
use crate::text_renderer_data::{TextRender, TextScroll};
use crate::{RenderCommand, TargetItem};
use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Affine, BezPath, Circle, Rectangle, Shape};
use retgui_resource_manager::ResourceId;
use std::sync::Arc;

pub struct RenderList {
    pub current_overlay_depth: u64,
    pub targets: Vec<TargetItem>,
    pub commands: Vec<RenderCommand>,
    /// Stores a sorted list of render command handles. This gets set in `Renderer::sort_render_list`.
    pub overlay: SortedCommands,
    pub cull: Option<Rectangle>,
    pub transform: Affine,
    /// Effective world-space clip while the render list is being built.
    pub(crate) current_clip: Option<Rectangle>,
    pub(crate) clip_stack: Vec<Option<Rectangle>>,
    pub(crate) overlay_clip_stack: Vec<Option<Rectangle>>,
}

impl Default for RenderList {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderList {
    pub fn new() -> Self {
        Self {
            current_overlay_depth: 0,
            targets: Vec::new(),
            commands: Vec::new(),
            overlay: SortedCommands { children: vec![] },
            cull: None,
            transform: Affine::IDENTITY,
            current_clip: None,
            clip_stack: Vec::new(),
            overlay_clip_stack: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.targets.clear();
        self.commands.clear();
        self.overlay.children.clear();
        self.current_overlay_depth = 0;
        self.transform = Affine::IDENTITY;
        self.clip_stack.clear();
        self.overlay_clip_stack.clear();
        self.current_clip = self.cull;
    }

    #[inline(always)]
    pub fn set_transform(&mut self, transform: Affine) {
        self.transform = transform;
    }

    #[inline(always)]
    pub fn get_transform(&self) -> Affine {
        self.transform
    }

    /// Returns the effective world-space clip while building the render list.
    #[inline(always)]
    pub fn get_clip(&self) -> Option<Rectangle> {
        self.current_clip
    }

    pub fn draw_circle(&mut self, circle: Circle, brush: Brush) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &circle.bounding_box(), self.cull.as_ref()) {
            return;
        }

        self.commands.push(RenderCommand::DrawCircle(DrawCircleCmd {
            circle,
            brush,
            transform,
        }));
    }

    pub fn draw_circle_outline(&mut self, circle: Circle, outline_brush: Brush, thickness: f32) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &circle.bounding_box(), self.cull.as_ref()) {
            return;
        }

        self.commands
            .push(RenderCommand::DrawCircleOutline(DrawCircleOutlineCmd {
                circle,
                outline_brush,
                thickness,
                transform,
            }));
    }

    #[inline(always)]
    pub fn draw_rect(&mut self, rect: Rectangle, brush: Brush) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.cull.as_ref()) {
            return;
        }

        self.commands.push(RenderCommand::DrawRect(DrawRectCmd {
            rect,
            brush,
            transform,
        }));
    }

    #[inline(always)]
    pub fn push_hit_testable(&mut self, id: u64, bounding_box: Rectangle) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &bounding_box, self.cull.as_ref()) {
            return;
        }

        let mut bounding_box = Rectangle::from_kurbo(transform.transform_rect_bbox(bounding_box.to_kurbo()));
        if let Some(clip) = self.get_clip() {
            let Some(clipped) = bounding_box.intersection(&clip) else {
                return;
            };
            bounding_box = clipped;
        }

        let overlay_depth = self.current_overlay_depth;
        self.targets.push(TargetItem::new(id, bounding_box, overlay_depth));
    }

    #[inline(always)]
    pub fn draw_rect_outline(&mut self, rect: Rectangle, outline_brush: Brush, thickness: f64) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.cull.as_ref()) {
            return;
        }

        self.commands.push(RenderCommand::DrawRectOutline(DrawRectOutlineCmd {
            rect,
            outline_brush,
            thickness,
            transform,
        }));
    }

    #[inline(always)]
    pub fn fill_bez_path(&mut self, path: BezPath, brush: Brush) {
        let transform = self.get_transform();
        if should_cull_bez_path(&transform, &path, self.cull.as_ref()) {
            return;
        }

        self.commands.push(RenderCommand::FillBezPath(FillBezPathCmd {
            path,
            brush,
            transform,
        }));
    }

    #[inline(always)]
    pub fn stroke_bez_path(&mut self, path: BezPath, brush: Brush) {
        let transform = self.get_transform();
        if should_cull_bez_path(&transform, &path, self.cull.as_ref()) {
            return;
        }

        self.commands.push(RenderCommand::StrokeBezPath(StrokeBezPathCmd {
            path,
            brush,
            transform,
        }));
    }

    #[inline(always)]
    pub fn draw_text(
        &mut self,
        data: Arc<TextRender>,
        rect: Rectangle,
        text_scroll: Option<TextScroll>,
        show_cursor: bool,
        use_glyph_cache: bool,
    ) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.cull.as_ref()) {
            return;
        }

        self.commands.push(RenderCommand::DrawText(DrawTextCmd {
            rect,
            data,
            text_scroll,
            show_cursor,
            use_glyph_cache,
            transform,
        }));
    }

    #[inline(always)]
    pub fn draw_image(&mut self, rect: Rectangle, resource_id: ResourceId) {
        let transform = self.get_transform();
        if should_cull_rect(&transform, &rect, self.cull.as_ref()) {
            return;
        }

        self.commands.push(RenderCommand::DrawImage(DrawImageCmd {
            rect,
            resource_id,
            transform,
        }));
    }

    #[inline(always)]
    pub fn push_layer(&mut self, rect: Rectangle) {
        let transform = self.get_transform();
        let world_rect = Rectangle::from_kurbo(transform.transform_rect_bbox(rect.to_kurbo()));
        let previous_clip = self.current_clip;
        let next_clip = match previous_clip {
            Some(clip) => Some(
                clip.intersection(&world_rect)
                    .unwrap_or_else(|| Rectangle::new(0.0, 0.0, -1.0, -1.0)),
            ),
            None => Some(world_rect),
        };
        self.clip_stack.push(previous_clip);
        self.current_clip = next_clip;

        self.commands
            .push(RenderCommand::PushLayer(PushLayerCmd::Rect(rect, transform)));
    }

    pub fn push_layer_with_bez_path(&mut self, path: BezPath) {
        let transform = self.get_transform();
        let world_rect = Rectangle::from_kurbo(transform.transform_rect_bbox(path.bounding_box()));
        let previous_clip = self.current_clip;
        let next_clip = match previous_clip {
            Some(clip) => Some(
                clip.intersection(&world_rect)
                    .unwrap_or_else(|| Rectangle::new(0.0, 0.0, -1.0, -1.0)),
            ),
            None => Some(world_rect),
        };
        self.clip_stack.push(previous_clip);
        self.current_clip = next_clip;

        self.commands
            .push(RenderCommand::PushLayer(PushLayerCmd::BezPath(path, transform)));
    }

    #[inline(always)]
    pub fn pop_layer(&mut self) {
        self.current_clip = self.clip_stack.pop().expect("renderer layer stack underflow");
        self.commands.push(RenderCommand::PopLayer);
    }

    pub fn start_overlay(&mut self) {
        let previous_clip = self.current_clip;
        self.overlay_clip_stack.push(previous_clip);
        self.current_clip = self.cull;
        self.commands.push(RenderCommand::StartOverlay);
        self.current_overlay_depth += 1;
    }

    pub fn end_overlay(&mut self) {
        self.commands.push(RenderCommand::EndOverlay);
        self.current_overlay_depth -= 1;
        self.current_clip = self.overlay_clip_stack.pop().expect("renderer overlay stack underflow");
    }

    #[inline(always)]
    pub fn draw_outset_box_shadow(&mut self, box_shadow: DrawBoxShadow) {
        let transform = self.get_transform();

        self.commands.push(RenderCommand::BoxShadowCmd(BoxShadowCmd {
            box_shadow,
            transform,
        }));
    }

    pub fn set_cull(&mut self, cull: Option<Rectangle>) {
        self.cull = cull;
        if self.clip_stack.is_empty() {
            self.current_clip = cull;
        }
    }

    pub fn sort_render_list(&mut self) {
        let mut current: *mut SortedCommands = &mut self.overlay;
        let mut stack: Vec<*mut SortedCommands> = vec![current];

        for (index, command) in self.commands.iter().enumerate() {
            match &command {
                RenderCommand::StartOverlay => {
                    // Overlay Start
                    unsafe {
                        (*current)
                            .children
                            .push(SortedItem::Overlay(SortedCommands { children: vec![] }));
                        match (*current).children.last_mut() {
                            Some(SortedItem::Overlay(overlay)) => {
                                stack.push(overlay);
                            }
                            _ => {
                                panic!("OverlayRender stack corrupted");
                            }
                        }
                        current = *stack.last_mut().unwrap();
                    }
                }
                RenderCommand::EndOverlay => {
                    // Overlay End
                    stack.pop();
                    current = *stack.last_mut().unwrap();
                }

                RenderCommand::PushLayer(_) | RenderCommand::PopLayer => {
                    // Normal Draw Command
                    unsafe {
                        (*current).children.push(SortedItem::Other(index as u32));
                    }
                }

                _ => unsafe {
                    (*current).children.push(SortedItem::Other(index as u32));
                },
            }
        }
    }
}

#[inline(always)]
fn should_cull_bez_path(transform: &Affine, path: &BezPath, cull: Option<&Rectangle>) -> bool {
    if let Some(cull) = cull {
        let bb = path.bounding_box();
        let bb_transformed = transform.transform_rect_bbox(bb);

        if !bb_transformed.overlaps(cull.to_kurbo()) {
            return true;
        }
    }

    false
}

#[inline(always)]
fn should_cull_rect(transform: &Affine, rect: &Rectangle, cull: Option<&Rectangle>) -> bool {
    if let Some(cull) = cull {
        let bb = rect.to_kurbo();
        let bb_transformed = transform.transform_rect_bbox(bb);

        if !bb_transformed.overlaps(cull.to_kurbo()) {
            return true;
        }
    }

    false
}
