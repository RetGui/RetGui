use retgui::elements::Element;
use retgui::events::PointerId;
use retgui::style::{AlignContent, AlignItems, AlignSelf, Animation, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, TextAlign, Unit};
use retgui::{App, Color, Gradient};

pub trait ElementBuilder: Sized {
    type Element: Element;

    fn element(&self) -> Self::Element;

    fn push<S: 'static>(self, child: impl Element, app: &mut App<S>) -> Self {
        self.element().push(app, child);
        self
    }

    fn id<S: 'static>(self, app: &mut App<S>, id: &str) -> Self {
        self.element().set_id(app, id);
        self
    }

    fn accessibility_name<S: 'static>(self, app: &mut App<S>, name: &str) -> Self {
        self.element().set_accessibility_name(app, name);
        self
    }

    fn display<S: 'static>(self, app: &mut App<S>, display: Display) -> Self {
        self.element().set_display(app, display);
        self
    }

    fn box_sizing<S: 'static>(self, app: &mut App<S>, box_sizing: BoxSizing) -> Self {
        self.element().set_box_sizing(app, box_sizing);
        self
    }

    fn position<S: 'static>(self, app: &mut App<S>, position: Position) -> Self {
        self.element().set_position(app, position);
        self
    }

    fn overlay<S: 'static>(self, app: &mut App<S>, overlay: bool) -> Self {
        self.element().set_overlay(app, overlay);
        self
    }

    fn margin<S: 'static>(self, app: &mut App<S>, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_margin(app, top, right, bottom, left);
        self
    }

    fn margin_all<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_margin_all(app, value);
        self
    }

    fn margin_vertical<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_margin_vertical(app, value);
        self
    }

    fn margin_horizontal<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_margin_horizontal(app, value);
        self
    }

    fn padding<S: 'static>(self, app: &mut App<S>, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_padding(app, top, right, bottom, left);
        self
    }

    fn padding_all<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_padding_all(app, value);
        self
    }

    fn padding_vertical<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_padding_vertical(app, value);
        self
    }

    fn padding_horizontal<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_padding_horizontal(app, value);
        self
    }

    fn gap<S: 'static>(self, app: &mut App<S>, row_gap: Unit, column_gap: Unit) -> Self {
        self.element().set_gap(app, row_gap, column_gap);
        self
    }

    fn row_gap<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_row_gap(app, value);
        self
    }

    fn column_gap<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_column_gap(app, value);
        self
    }

    fn inset<S: 'static>(self, app: &mut App<S>, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_inset(app, top, right, bottom, left);
        self
    }

    fn min_width<S: 'static>(self, app: &mut App<S>, min_width: Unit) -> Self {
        self.element().set_min_width(app, min_width);
        self
    }

    fn min_height<S: 'static>(self, app: &mut App<S>, min_height: Unit) -> Self {
        self.element().set_min_height(app, min_height);
        self
    }

    fn width<S: 'static>(self, app: &mut App<S>, width: Unit) -> Self {
        self.element().set_width(app, width);
        self
    }

    fn height<S: 'static>(self, app: &mut App<S>, height: Unit) -> Self {
        self.element().set_height(app, height);
        self
    }

    fn max_width<S: 'static>(self, app: &mut App<S>, max_width: Unit) -> Self {
        self.element().set_max_width(app, max_width);
        self
    }

    fn max_height<S: 'static>(self, app: &mut App<S>, max_height: Unit) -> Self {
        self.element().set_max_height(app, max_height);
        self
    }

    fn wrap<S: 'static>(self, app: &mut App<S>, wrap: FlexWrap) -> Self {
        self.element().set_wrap(app, wrap);
        self
    }

    fn align_items<S: 'static>(self, app: &mut App<S>, align_items: AlignItems) -> Self {
        self.element().set_align_items(app, align_items);
        self
    }

    fn align_self<S: 'static>(self, app: &mut App<S>, align_self: AlignSelf) -> Self {
        self.element().set_align_self(app, align_self);
        self
    }

    fn align_content<S: 'static>(self, app: &mut App<S>, align_content: AlignContent) -> Self {
        self.element().set_align_content(app, align_content);
        self
    }

    fn justify_content<S: 'static>(self, app: &mut App<S>, justify_content: JustifyContent) -> Self {
        self.element().set_justify_content(app, justify_content);
        self
    }

    fn flex_direction<S: 'static>(self, app: &mut App<S>, flex_direction: FlexDirection) -> Self {
        self.element().set_flex_direction(app, flex_direction);
        self
    }

    fn flex_grow<S: 'static>(self, app: &mut App<S>, flex_grow: f32) -> Self {
        self.element().set_flex_grow(app, flex_grow);
        self
    }

    fn flex_shrink<S: 'static>(self, app: &mut App<S>, flex_shrink: f32) -> Self {
        self.element().set_flex_shrink(app, flex_shrink);
        self
    }

    fn flex_basis<S: 'static>(self, app: &mut App<S>, flex_basis: Unit) -> Self {
        self.element().set_flex_basis(app, flex_basis);
        self
    }

    fn order<S: 'static>(self, app: &mut App<S>, order: i32) -> Self {
        self.element().set_order(app, order);
        self
    }

    fn font_family<S: 'static>(self, app: &mut App<S>, font_family: FontFamily) -> Self {
        self.element().set_font_family(app, font_family);
        self
    }

    fn color<S: 'static>(self, app: &mut App<S>, color: Color) -> Self {
        self.element().set_color(app, color);
        self
    }

    fn text_gradient<S: 'static>(self, app: &mut App<S>, gradient: Gradient) -> Self {
        self.element().set_text_gradient(app, gradient);
        self
    }

    fn background_color<S: 'static>(self, app: &mut App<S>, background_color: Color) -> Self {
        self.element().set_background_color(app, background_color);
        self
    }

    fn background_gradient<S: 'static>(self, app: &mut App<S>, gradient: Gradient) -> Self {
        self.element().set_background_gradient(app, gradient);
        self
    }

    fn font_size<S: 'static>(self, app: &mut App<S>, font_size: f32) -> Self {
        self.element().set_font_size(app, font_size);
        self
    }

    fn line_height<S: 'static>(self, app: &mut App<S>, line_height: f32) -> Self {
        self.element().set_line_height(app, line_height);
        self
    }

    fn font_weight<S: 'static>(self, app: &mut App<S>, font_weight: FontWeight) -> Self {
        self.element().set_font_weight(app, font_weight);
        self
    }

    fn font_style<S: 'static>(self, app: &mut App<S>, font_style: FontStyle) -> Self {
        self.element().set_font_style(app, font_style);
        self
    }

    fn text_align<S: 'static>(self, app: &mut App<S>, text_align: TextAlign) -> Self {
        self.element().set_text_align(app, text_align);
        self
    }

    fn underline<S: 'static>(
        self,
        app: &mut App<S>,
        thickness: Option<f32>,
        color: Color,
        offset: Option<f32>,
    ) -> Self {
        self.element().set_underline(app, thickness, color, offset);
        self
    }

    fn underline_gradient<S: 'static>(
        self,
        app: &mut App<S>,
        thickness: Option<f32>,
        gradient: Gradient,
        offset: Option<f32>,
    ) -> Self {
        self.element().set_underline_gradient(app, thickness, gradient, offset);
        self
    }

    fn overflow<S: 'static>(self, app: &mut App<S>, overflow_x: Overflow, overflow_y: Overflow) -> Self {
        self.element().set_overflow(app, overflow_x, overflow_y);
        self
    }

    fn overflow_x<S: 'static>(self, app: &mut App<S>, overflow_x: Overflow) -> Self {
        self.element().set_overflow_x(app, overflow_x);
        self
    }

    fn overflow_y<S: 'static>(self, app: &mut App<S>, overflow_y: Overflow) -> Self {
        self.element().set_overflow_y(app, overflow_y);
        self
    }

    fn border_color<S: 'static>(self, app: &mut App<S>, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.element().set_border_color(app, top, right, bottom, left);
        self
    }

    fn border_color_all<S: 'static>(self, app: &mut App<S>, value: Color) -> Self {
        self.element().set_border_color_all(app, value);
        self
    }

    fn border_color_vertical<S: 'static>(self, app: &mut App<S>, value: Color) -> Self {
        self.element().set_border_color_vertical(app, value);
        self
    }

    fn border_color_horizontal<S: 'static>(self, app: &mut App<S>, value: Color) -> Self {
        self.element().set_border_color_horizontal(app, value);
        self
    }

    fn border_width<S: 'static>(self, app: &mut App<S>, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_border_width(app, top, right, bottom, left);
        self
    }

    fn border_width_all<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_border_width_all(app, value);
        self
    }

    fn border_width_vertical<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_border_width_vertical(app, value);
        self
    }

    fn border_width_horizontal<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_border_width_horizontal(app, value);
        self
    }

    fn outline_color<S: 'static>(self, app: &mut App<S>, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.element().set_outline_color(app, top, right, bottom, left);
        self
    }

    fn outline_color_all<S: 'static>(self, app: &mut App<S>, value: Color) -> Self {
        self.element().set_outline_color_all(app, value);
        self
    }

    fn outline_color_vertical<S: 'static>(self, app: &mut App<S>, value: Color) -> Self {
        self.element().set_outline_color_vertical(app, value);
        self
    }

    fn outline_color_horizontal<S: 'static>(self, app: &mut App<S>, value: Color) -> Self {
        self.element().set_outline_color_horizontal(app, value);
        self
    }

    fn outline_width<S: 'static>(self, app: &mut App<S>, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_outline_width(app, top, right, bottom, left);
        self
    }

    fn outline_width_all<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_outline_width_all(app, value);
        self
    }

    fn outline_width_vertical<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_outline_width_vertical(app, value);
        self
    }

    fn outline_width_horizontal<S: 'static>(self, app: &mut App<S>, value: Unit) -> Self {
        self.element().set_outline_width_horizontal(app, value);
        self
    }

    fn border_radius<S: 'static>(
        self,
        app: &mut App<S>,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) -> Self {
        self.element().set_border_radius(app, top, right, bottom, left);
        self
    }

    fn border_radius_all<S: 'static>(self, app: &mut App<S>, value: (f32, f32)) -> Self {
        self.element().set_border_radius_all(app, value);
        self
    }

    fn border_radius_vertical<S: 'static>(self, app: &mut App<S>, value: (f32, f32)) -> Self {
        self.element().set_border_radius_vertical(app, value);
        self
    }

    fn border_radius_horizontal<S: 'static>(self, app: &mut App<S>, value: (f32, f32)) -> Self {
        self.element().set_border_radius_horizontal(app, value);
        self
    }

    fn scrollbar_color<S: 'static>(self, app: &mut App<S>, scrollbar_color: ScrollbarColor) -> Self {
        self.element().set_scrollbar_color(app, scrollbar_color);
        self
    }

    fn scrollbar_thumb_margin<S: 'static>(
        self,
        app: &mut App<S>,
        top: f32,
        right: f32,
        bottom: f32,
        left: f32,
    ) -> Self {
        self.element().set_scrollbar_thumb_margin(app, top, right, bottom, left);
        self
    }

    fn scrollbar_thumb_radius<S: 'static>(
        self,
        app: &mut App<S>,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) -> Self {
        self.element().set_scrollbar_thumb_radius(app, top, right, bottom, left);
        self
    }

    fn scrollbar_width<S: 'static>(self, app: &mut App<S>, scrollbar_width: f32) -> Self {
        self.element().set_scrollbar_width(app, scrollbar_width);
        self
    }

    fn animations<S: 'static>(self, app: &mut App<S>, animations: Vec<Animation>) -> Self {
        self.element().set_animations(app, animations);
        self
    }

    fn box_shadows<S: 'static>(self, app: &mut App<S>, box_shadows: Vec<BoxShadow>) -> Self {
        self.element().set_box_shadows(app, box_shadows);
        self
    }

    fn pointer_capture<S: 'static>(self, app: &mut App<S>, pointer_id: PointerId) -> Self {
        self.element().set_pointer_capture(app, pointer_id);
        self
    }
}
