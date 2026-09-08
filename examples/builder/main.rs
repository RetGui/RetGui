use retgui::elements::{Container, Element, Slider, Text, TextInput, Window};
use retgui::events::{SliderValueChangedEvent, TextInputChangedEvent};
use retgui::style::{FlexDirection, FontWeight, JustifyContent, Overflow};
use retgui::{App, Brush, RetGuiOptions, pct, px, retgui_main, rgb};

use retgui_builder::prelude::*;

const INITIAL_TEXT: &str = "Hello, RetGui!";
const INITIAL_FONT_SIZE: f32 = 28.0;

struct PreviewState {
    preview: Option<Text>,
    size_label: Option<Text>,
}

fn font_size_label(size: f32) -> String {
    let mut label = String::from("Font size: ");
    label.push_str(&(size.round() as u32).to_string());
    label.push_str(" px");
    label
}

fn update_text(event: &mut TextInputChangedEvent, app: &mut App<PreviewState>, state: &mut PreviewState) {
    if let Some(preview) = state.preview {
        preview.set_text(app, &event.value);
    }
}

fn update_font_size(event: &mut SliderValueChangedEvent, app: &mut App<PreviewState>, state: &mut PreviewState) {
    if let Some(preview) = state.preview {
        preview.set_font_size(app, event.value as f32);
    }
    if let Some(label) = state.size_label {
        label.set_text(app, &font_size_label(event.value as f32));
    }
}

fn builder_example(app: &mut App<PreviewState>, state: &mut PreviewState) -> Container {
    Container::new(app)
        .builder()
        .flex_direction(app, FlexDirection::Column)
        .justify_content(app, JustifyContent::Center)
        .row_gap(app, px(24))
        .padding_all(app, px(32))
        .width(app, pct(100))
        .min_height(app, pct(100))
        .color(app, rgb(30, 41, 59))
        .push(
            Text::new(app, "Live text preview")
                .builder()
                .font_size(app, 24.0)
                .font_weight(app, FontWeight::BOLD)
                .selectable(app, false)
                .build(),
            app,
        )
        .push(
            Container::new(app)
                .builder()
                .flex_direction(app, FlexDirection::Column)
                .row_gap(app, px(16))
                .push(
                    {
                        let input = TextInput::new(app, INITIAL_TEXT)
                            .builder()
                            .accessibility_name(app, "Preview text")
                            .multiline(app, false)
                            .font_size(app, 18.0)
                            .height(app, px(48))
                            .padding_all(app, px(10))
                            .border_width_all(app, px(1))
                            .border_color_all(app, rgb(148, 163, 184))
                            .border_radius_all(app, (6.0, 6.0))
                            .build();
                        input.add_text_input_changed_listener(app, update_text);
                        input
                    },
                    app,
                )
                .push(
                    {
                        let label = Text::new(app, &font_size_label(INITIAL_FONT_SIZE))
                            .builder()
                            .font_size(app, 14.0)
                            .selectable(app, false)
                            .build();
                        state.size_label = Some(label);
                        label
                    },
                    app,
                )
                .push(
                    {
                        let slider = Slider::new(app, 20.0)
                            .builder()
                            .accessibility_name(app, "Preview font size")
                            .min(app, 12.0)
                            .max(app, 64.0)
                            .step(app, 1.0)
                            .value(app, f64::from(INITIAL_FONT_SIZE))
                            .height(app, px(28))
                            .thumb_color(app, Brush::Color(rgb(37, 99, 235)))
                            .track_color(app, rgb(203, 213, 225))
                            .build();
                        slider.add_slider_value_changed_listener(app, update_font_size);
                        slider
                    },
                    app,
                )
                .build(),
            app,
        )
        .push(
            {
                let preview = Text::new(app, INITIAL_TEXT)
                    .builder()
                    .id(app, "preview")
                    .font_size(app, INITIAL_FONT_SIZE)
                    .selectable(app, true)
                    .padding_all(app, px(20))
                    .min_height(app, px(120))
                    .background_color(app, rgb(239, 246, 255))
                    .border_radius_all(app, (8.0, 8.0))
                    .build();
                state.preview = Some(preview);
                preview
            },
            app,
        )
        .build()
}

fn main() {
    util::setup_logging();

    let mut app = App::new();
    let mut state = PreviewState {
        preview: None,
        size_label: None,
    };
    Window::new(&mut app, "Builder example")
        .builder()
        .width(&mut app, pct(100))
        .height(&mut app, pct(100))
        .overflow_y(&mut app, Overflow::Scroll)
        .background_color(&mut app, rgb(255, 255, 255))
        .push(builder_example(&mut app, &mut state), &mut app)
        .build();

    retgui_main(app, state, RetGuiOptions::basic("Builder example"));
}
