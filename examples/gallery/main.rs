#[cfg(feature = "audio")]
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

#[cfg(feature = "audio")]
use retgui::elements::Audio;
use retgui::elements::{
    Button, Calendar, Checkbox, CheckboxGroup, Container, Dropdown, DynElement, Element, Image, Radio, RadioGroup,
    Slider, SliderDirection, Text, TextInput, TinyVg, Window,
};
use retgui::events::Event;
use retgui::geometry::Point;
use retgui::style::{
    AlignItems, Animation, BoxShadow, Display, FlexDirection, FontFamily, FontStyle, FontWeight, JustifyContent,
    KeyFrame, Overflow, Position, Repeat, StyleVariant, TextAlign, TimingFunction,
};
use retgui::{
    App, Brush, Color, ColorStop, Gradient, ResourceId, ResourceType, RetGuiOptions, auto, pct, px, retgui_main, rgb,
    rgba,
};

use serde::Deserialize;

use util::setup_logging;

pub fn title(app: &mut App<GalleryState>, value: &str) -> Text {
    let text = Text::new(app, value);
    text.set_font_weight(app, FontWeight::BOLD);
    text.set_font_size(app, 20.0);
    text.set_margin(app, px(0.0), px(0.0), px(5.0), px(0.0));
    text
}

pub fn animations(app: &mut App<GalleryState>) -> Text {
    let gameboy_gradient = |start, end| {
        Gradient::new_linear(Point::new(start, 0.0), Point::new(end, 0.0)).color_stops(&[
            ColorStop::new(0.0, Color::from_rgb8(50, 50, 252)),
            ColorStop::new(0.2, Color::from_rgb8(133, 227, 103)),
            ColorStop::new(0.4, Color::from_rgb8(255, 82, 232)),
            ColorStop::new(0.6, Color::from_rgb8(255, 1, 81)),
            ColorStop::new(0.8, Color::from_rgb8(249, 229, 46)),
            ColorStop::new(1.0, Color::from_rgb8(240, 240, 240)),
        ])
    };
    let start = gameboy_gradient(-0.5, 1.0);
    let end = gameboy_gradient(0.0, 1.5);
    let animation = Animation::new(Duration::from_secs(3), Repeat::Forever, TimingFunction::EaseInOut)
        .push(KeyFrame::new(0.0).push(StyleVariant::TextBrush(Brush::Gradient(start.clone()))))
        .push(KeyFrame::new(50.0).push(StyleVariant::TextBrush(Brush::Gradient(end))))
        .push(KeyFrame::new(100.0).push(StyleVariant::TextBrush(Brush::Gradient(start))));
    let text = Text::new(app, "Animations");
    text.set_font_size(app, 64.0);
    text.set_font_weight(app, FontWeight::BOLD);
    text.set_animations(app, vec![animation]);
    text
}

pub fn text_input(app: &mut App<GalleryState>) -> Container {
    let input = TextInput::new(app, "An element for text input");
    input.set_width(app, px(200.0));
    input.set_height(app, px(200.0));
    let heading = title(app, "Text Input");
    let container = Container::new(app);
    container.set_display(app, Display::Block);
    container.push(app, heading);
    container.push(app, input);
    container
}

pub fn dropdown(app: &mut App<GalleryState>) -> Container {
    let cat = Text::new(app, "Cat");
    let dog = Text::new(app, "Dog");
    let dropdown = Dropdown::new(app);
    dropdown.set_width(app, px(100.0));
    dropdown.push(app, cat);
    dropdown.push(app, dog);
    dropdown.set_selected_item(app, 0);
    let heading = title(app, "Dropdown");
    let container = Container::new(app);
    container.set_min_width(app, px(200.0));
    container.set_display(app, Display::Block);
    container.push(app, heading);
    container.push(app, dropdown);
    container
}

pub fn text(app: &mut App<GalleryState>) -> Container {
    let normal = Text::new(app, "Normal Text with a Color");
    normal.set_color(app, Color::from_rgb8(0, 0, 255));
    let bold = Text::new(app, "Bold Text");
    bold.set_font_weight(app, FontWeight::BOLD);
    let italic = Text::new(app, "Italic Text");
    italic.set_font_style(app, FontStyle::Italic);
    let bold_italic = Text::new(app, "Bold & Italic Text");
    bold_italic.set_font_weight(app, FontWeight::BOLD);
    bold_italic.set_font_style(app, FontStyle::Italic);
    let underlined = Text::new(app, "Underlined Text");
    underlined.set_underline(app, Some(2.0), Color::from_rgb8(0, 255, 0), None);
    let left = Text::new(app, "Left");
    left.set_text_align(app, TextAlign::Left);
    let center = Text::new(app, "Center");
    center.set_text_align(app, TextAlign::Center);
    let right = Text::new(app, "Right");
    right.set_text_align(app, TextAlign::Right);
    let heading = title(app, "Text");
    let container = Container::new(app);
    container.set_display(app, Display::Block);
    container.push(app, heading);
    container.push(app, normal);
    container.push(app, bold);
    container.push(app, italic);
    container.push(app, bold_italic);
    container.push(app, underlined);
    container.push(app, left);
    container.push(app, center);
    container.push(app, right);
    container
}

pub fn variable_fonts(app: &mut App<GalleryState>) -> Container {
    let font = include_bytes!("../../assets/fonts/Roboto-VariableFont_wdth,wght.ttf");
    app.upload_resource(ResourceId::StaticBytes(font), ResourceType::Font, font.as_slice())
        .expect("gallery font must load");
    let heading = title(app, "Variable Fonts");
    let description = Text::new(app, "Roboto: drag the slider to explore font weights from 100 to 900.");
    let preview = Text::new(
        app,
        "The quick brown fox jumps over the lazy dog.\nABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz 0123456789",
    );
    preview.set_font_family(app, FontFamily::new("Roboto"));
    preview.set_font_size(app, 36.0);
    preview.set_font_weight(app, FontWeight::NORMAL);
    let weight_label = Text::new(app, "Weight: 400");
    let weight = Slider::new(app, 20.0);
    weight.set_min(app, 100.0);
    weight.set_max(app, 900.0);
    weight.set_step(app, 1.0);
    weight.set_value(app, 400.0);
    weight.set_width(app, px(300.0));
    weight.set_height(app, px(10.0));
    weight.set_margin_vertical(app, px(10.0));
    weight.add_slider_value_changed_listener(app, move |event, app, _| {
        let weight = event.value.round() as u16;
        preview.set_font_weight(app, FontWeight(weight));
        weight_label.set_text(app, &format!("Weight: {weight}"));
    });

    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_row_gap(app, px(12.0));
    container.push(app, heading);
    container.push(app, description);
    container.push(app, weight_label);
    container.push(app, weight);
    container.push(app, preview);
    container
}

pub fn tinyvg(app: &mut App<GalleryState>) -> Container {
    let tiger = include_bytes!("tiger.tvg");
    app.upload_resource(ResourceId::StaticBytes(tiger), ResourceType::TinyVg, tiger.as_slice())
        .expect("gallery image must load");
    let image = TinyVg::new(app, ResourceId::StaticBytes(include_bytes!("tiger.tvg")));
    image.set_width(app, px(250.0));
    image.set_height(app, px(250.0));
    let heading = title(app, "TinyVG");
    let container = Container::new(app);
    container.set_display(app, Display::Block);
    container.push(app, heading);
    container.push(app, image);
    container
}

pub fn images(app: &mut App<GalleryState>) -> Container {
    let image = Image::new(app, ResourceId::Url("https://picsum.photos/300/200".to_string()));
    image.set_width(app, px(300.0));
    image.set_height(app, px(200.0));
    let heading = title(app, "Image");
    let container = Container::new(app);
    container.set_display(app, Display::Block);
    container.push(app, heading);
    container.push(app, image);
    container
}

#[derive(Deserialize)]
struct WeatherResponse {
    current: CurrentWeather,
}

#[derive(Deserialize)]
struct CurrentWeather {
    time: String,
    temperature_2m: f64,
    apparent_temperature: f64,
    relative_humidity_2m: u8,
    weather_code: u8,
    wind_speed_10m: f64,
}

async fn fetch_amsterdam_weather() -> Result<CurrentWeather, String> {
    let response = reqwest::get(
        "https://api.open-meteo.com/v1/forecast?latitude=52.374&longitude=4.8897&current=temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,wind_speed_10m&timezone=Europe%2FAmsterdam",
    )
    .await
    .map_err(|error| error.to_string())?
    .error_for_status()
    .map_err(|error| error.to_string())?;
    let weather = response
        .json::<WeatherResponse>()
        .await
        .map_err(|error| error.to_string())?;
    Ok(weather.current)
}

fn weather_description(code: u8) -> &'static str {
    match code {
        0 => "Clear sky",
        1..=3 => "Partly cloudy",
        45 | 48 => "Fog",
        51..=57 => "Drizzle",
        61..=67 => "Rain",
        71..=77 => "Snow",
        80..=82 => "Rain showers",
        85 | 86 => "Snow showers",
        95..=99 => "Thunderstorm",
        _ => "Unknown conditions",
    }
}

pub fn async_weather(app: &mut App<GalleryState>) -> Container {
    let status = Text::new(app, "Click the button for the current conditions.");
    status.set_width(app, px(280.0));
    status.set_font_size(app, 14.0);
    let label = Text::new(app, "Refresh Weather");
    label.set_color(app, Color::WHITE);
    label.set_selectable(app, false);
    let button = Button::new(app);
    button.set_padding(app, px(5.0), px(15.0), px(5.0), px(15.0));
    button.set_border_radius_all(app, (4.0, 4.0));
    button.set_background_color(app, Color::from_rgb8(35, 127, 183));
    button.push(app, label);
    button.add_click_listener(app, move |event, app, _| {
        status.set_text(app, "Loading...");
        app.spawn_local(fetch_amsterdam_weather(), move |weather, app, _state| {
            let message = match weather {
                Ok(weather) => format!(
                    "{}\n{:.1} °C (feels like {:.1} °C)\nHumidity: {}%\nWind: {:.1} km/h\nUpdated: {}",
                    weather_description(weather.weather_code),
                    weather.temperature_2m,
                    weather.apparent_temperature,
                    weather.relative_humidity_2m,
                    weather.wind_speed_10m,
                    weather.time,
                ),
                Err(error) => format!("Request failed: {error}"),
            };
            status.set_text(app, &message);
        });
        event.stop_propagation();
    });
    let heading = title(app, "Amsterdam Weather");
    let attribution = Text::new(app, "Weather data by Open-Meteo");
    attribution.set_font_size(app, 12.0);
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_row_gap(app, px(8.0));
    container.push(app, heading);
    container.push(app, button);
    container.push(app, status);
    container.push(app, attribution);
    container
}

pub fn gradient(app: &mut App<GalleryState>) -> Container {
    let linear = Gradient::new_linear(Point::new(0.0, 0.0), Point::new(1.0, 0.0)).color_stops(&[
        ColorStop::new(0.0, Color::from_rgb8(120, 0, 200)),
        ColorStop::new(0.45, Color::from_rgb8(35, 127, 183)),
        ColorStop::new(1.0, Color::from_rgb8(255, 0, 0)),
    ]);
    let radial = Gradient::new_radial(Point::new(0.5, 0.5), 0.0, Point::new(0.5, 0.5), 0.75).color_stops(&[
        ColorStop::new(0.0, Color::from_rgb8(255, 245, 157)),
        ColorStop::new(0.55, Color::from_rgb8(255, 112, 67)),
        ColorStop::new(1.0, Color::from_rgb8(74, 20, 140)),
    ]);
    let sweep = Gradient::new_sweep(Point::new(0.5, 0.5), 0.0, std::f32::consts::TAU).color_stops(&[
        ColorStop::new(0.0, Color::from_rgb8(244, 67, 54)),
        ColorStop::new(0.33, Color::from_rgb8(76, 175, 80)),
        ColorStop::new(0.66, Color::from_rgb8(33, 150, 243)),
        ColorStop::new(1.0, Color::from_rgb8(244, 67, 54)),
    ]);
    let linear_box = Container::new(app);
    linear_box.set_width(app, px(140.0));
    linear_box.set_height(app, px(90.0));
    linear_box.set_border_radius_all(app, (8.0, 8.0));
    linear_box.set_background_gradient(app, linear.clone());
    let radial_box = Container::new(app);
    radial_box.set_width(app, px(140.0));
    radial_box.set_height(app, px(90.0));
    radial_box.set_border_radius_all(app, (8.0, 8.0));
    radial_box.set_background_gradient(app, radial);
    let sweep_box = Container::new(app);
    sweep_box.set_width(app, px(140.0));
    sweep_box.set_height(app, px(90.0));
    sweep_box.set_border_radius_all(app, (8.0, 8.0));
    sweep_box.set_background_gradient(app, sweep);
    let gradient_text = Text::new(app, "Gradient Text");
    gradient_text.set_font_weight(app, FontWeight::BOLD);
    gradient_text.set_text_gradient(app, linear.clone());
    let underline = Text::new(app, "Gradient Underline");
    underline.set_underline_gradient(app, Some(3.0), linear, None);
    let boxes = Container::new(app);
    boxes.set_display(app, Display::Flex);
    boxes.set_gap(app, px(10.0), px(10.0));
    boxes.push(app, linear_box);
    boxes.push(app, radial_box);
    boxes.push(app, sweep_box);
    let heading = title(app, "Gradients");
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_row_gap(app, px(10.0));
    container.push(app, heading);
    container.push(app, gradient_text);
    container.push(app, underline);
    container.push(app, boxes);
    container
}

pub fn box_shadows(app: &mut App<GalleryState>) -> Container {
    let border_color = rgb(0, 0, 0);
    let shadow = Container::new(app);
    shadow.set_box_shadows(
        app,
        vec![
            BoxShadow::new(false, 0.0, 5.0, 5.0, 0.0, rgba(0, 0, 0, 200)),
            BoxShadow::new(false, 0.0, 25.0, 35.0, 0.0, rgba(0, 0, 0, 150)),
            BoxShadow::new(true, 0.0, 4.0, 4.0, 0.0, rgba(255, 255, 255, 120)),
        ],
    );
    shadow.set_border_width(app, px(0), px(0), px(0), px(0));
    shadow.set_border_color(app, border_color, border_color, border_color, border_color);
    shadow.set_border_radius(app, (8.0, 8.0), (8.0, 8.0), (8.0, 8.0), (8.0, 8.0));
    shadow.set_padding(app, px(15), px(30), px(15), px(30));
    shadow.set_justify_content(app, JustifyContent::Center);
    shadow.set_background_color(app, Color::from_rgb8(255, 0, 0));
    let heading = title(app, "Box Shadows");
    let container = Container::new(app);
    container.set_display(app, Display::Block);
    container.push(app, heading);
    container.push(app, shadow);
    container
}

pub fn overlay(app: &mut App<GalleryState>) -> Container {
    let status = Text::new(app, "Click where the cards overlap");
    let overlay_label = Text::new(app, "Overlay");
    overlay_label.set_color(app, Color::WHITE);
    overlay_label.set_selectable(app, false);
    let floating = Container::new(app);
    floating.set_overlay(app, true);
    floating.set_position(app, Position::Absolute);
    floating.set_inset(app, px(20.0), auto(), auto(), px(20.0));
    floating.set_width(app, px(150.0));
    floating.set_height(app, px(100.0));
    floating.set_padding_all(app, px(10.0));
    floating.set_background_color(app, Color::from_rgb8(76, 175, 80));
    floating.push(app, overlay_label);
    floating.add_click_listener(app, move |event, app, _| {
        status.set_text(app, "The overlay received the click");
        event.stop_propagation();
    });
    let normal_label = Text::new(app, "Normal sibling");
    normal_label.set_color(app, Color::WHITE);
    normal_label.set_selectable(app, false);
    let normal = Container::new(app);
    normal.set_position(app, Position::Absolute);
    normal.set_inset(app, px(65.0), auto(), auto(), px(90.0));
    normal.set_width(app, px(120.0));
    normal.set_height(app, px(70.0));
    normal.set_padding_all(app, px(10.0));
    normal.set_background_color(app, Color::from_rgb8(33, 150, 243));
    normal.push(app, normal_label);
    normal.add_click_listener(app, move |event, app, _| {
        status.set_text(app, "The normal sibling received the click");
        event.stop_propagation();
    });
    let cards = Container::new(app);
    cards.set_position(app, Position::Relative);
    cards.set_width(app, px(230.0));
    cards.set_height(app, px(155.0));
    cards.set_background_color(app, Color::from_rgb8(238, 238, 238));
    cards.push(app, floating);
    cards.push(app, normal);
    let heading = title(app, "Overlay");
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_row_gap(app, px(8.0));
    container.set_width(app, px(280.0));
    container.set_min_width(app, auto());
    container.push(app, heading);
    container.set_margin_horizontal(app, auto());
    container.push(app, cards);
    container.push(app, status);
    container
}

pub fn multiple_windows(app: &mut App<GalleryState>) -> Container {
    let radius = (1.0, 1.0);
    let border = Color::BLACK;
    let width = px(1.0);
    let label = Text::new(app, "Open a new window");
    let button = Button::new(app);
    button.push(app, label);
    button.set_padding(app, px(5.0), px(15.0), px(5.0), px(15.0));
    button.set_border_radius(app, radius, radius, radius, radius);
    button.set_border_color(app, border, border, border, border);
    button.set_border_width(app, width, width, width, width);
    button.add_click_listener(app, |_event, app, _| {
        let greeting = Text::new(app, "Hi!");
        greeting.set_font_size(app, 32.0);
        greeting.set_font_weight(app, FontWeight::BOLD);
        let window = Window::new(app, "A new window!");
        window.push(app, greeting);
    });
    let heading = title(app, "Multiple Windows");
    let container = Container::new(app);
    container.set_display(app, Display::Block);
    container.push(app, heading);
    container.push(app, button);
    container
}

pub fn sliders(app: &mut App<GalleryState>) -> Container {
    let first = Slider::new(app, 20.0);
    first.set_value(app, 70.0);
    first.set_width(app, px(100.0));
    first.set_height(app, px(10.0));
    let br = (0.0, 0.0);
    let second = Slider::new(app, 14.0);
    second.set_value(app, 20.0);
    second.set_width(app, px(100.0));
    second.set_height(app, px(10.0));
    second.set_track_color(app, Color::from_rgb8(120, 150, 0));
    second.set_border_radius(app, br, br, br, br);
    second.set_thumb_border_radius(app, br, br, br, br);
    let third = Slider::new(app, 20.0);
    third.set_value(app, 70.0);
    third.set_width(app, px(10.0));
    third.set_height(app, px(100.0));
    third.set_direction(app, SliderDirection::Vertical);
    let heading = title(app, "Sliders");
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_row_gap(app, px(15.0));
    container.push(app, heading);
    container.push(app, first);
    container.push(app, second);
    container.push(app, third);
    container
}

pub fn scrollable(app: &mut App<GalleryState>) -> Container {
    let start = Text::new(app, "The Start");
    let middle = Text::new(app, "The Middle");
    middle.set_margin(app, px(50.0), px(0.0), px(250.0), px(0.0));
    let end = Text::new(app, "The End");
    end.set_padding(app, px(0.0), px(0.0), px(10.0), px(0.0));
    let scrollable = Container::new(app);
    scrollable.set_display(app, Display::Block);
    scrollable.set_overflow_y(app, Overflow::Scroll);
    scrollable.set_width(app, px(200.0));
    scrollable.set_max_height(app, px(150.0));
    scrollable.set_padding(app, px(5.0), px(15.0), px(5.0), px(15.0));
    scrollable.set_border_radius_all(app, (1.0, 1.0));
    scrollable.set_border_color_all(app, Color::BLACK);
    scrollable.set_border_width_all(app, px(1.0));
    scrollable.push(app, start);
    scrollable.push(app, middle);
    scrollable.push(app, end);
    let label = Text::new(app, "Scroll to the top");
    label.set_color(app, Color::WHITE);
    label.set_font_size(app, 14.0);
    label.set_padding(app, px(3.0), px(5.0), px(3.0), px(5.0));
    let button = Button::new(app);
    button.set_width(app, px(120.0));
    button.set_background_color(app, Color::from_rgb8(35, 127, 183));
    button.add_click_listener(app, move |_event, app, _| {
        scrollable.scroll_to_top(app);
    });
    button.push(app, label);
    let heading = title(app, "Scrollable");
    let container = Container::new(app);
    container.set_display(app, Display::Block);
    container.push(app, heading);
    container.push(app, scrollable);
    container.push(app, button);
    container
}

pub fn radio_buttons(app: &mut App<GalleryState>) -> Container {
    let group = RadioGroup::new(app, "Pick a color");
    let green = Image::new(
        app,
        ResourceId::Url("https://www.iconsdb.com/icons/preview/green/square-xxl.png".to_string()),
    );
    green.set_border_width_all(app, px(1));
    green.set_border_color_all(app, rgba(0, 0, 0, 0));
    let red_label = Text::new(app, "red");
    let red = Radio::new(app, group, "red", "red", true);
    red.push(app, red_label);
    let green_radio = Radio::new(app, group, "green", "green", false);
    green_radio.push(app, green);
    green_radio.hide_radio(app);
    let blue_label = Text::new(app, "blue");
    let blue = Radio::new(app, group, "blue", "blue", false);
    blue.push(app, blue_label);
    group.set_display(app, Display::Flex);
    group.set_flex_direction(app, FlexDirection::Column);
    group.set_justify_content(app, JustifyContent::Center);
    group.push(app, red);
    group.push(app, green_radio);
    group.push(app, blue);
    group.add_radio_value_changed_listener(app, move |event, app, _| {
        green.set_border_color_all(
            app,
            if event.value.as_str() == "green" {
                rgb(0, 100, 255)
            } else {
                rgba(0, 0, 0, 0)
            },
        );
    });
    let heading = title(app, "Radio Button");
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.push(app, heading);
    container.push(app, group);
    container
}

pub fn checkbox(app: &mut App<GalleryState>) -> Container {
    let coffee_label = Text::new(app, "Coffee");
    coffee_label.set_selectable(app, false);
    let coffee = Checkbox::new(app, "coffee", true);
    coffee.push(app, coffee_label);
    let tea_label = Text::new(app, "Tea");
    tea_label.set_selectable(app, false);
    let tea = Checkbox::new(app, "tea", false);
    tea.push(app, tea_label);
    let pork_label = Text::new(app, "红烧肉");
    pork_label.set_selectable(app, false);
    let pork = Checkbox::new(app, "红烧肉", false);
    pork.push(app, pork_label);
    let curry_label = Text::new(app, "カツカレー");
    curry_label.set_selectable(app, false);
    let curry = Checkbox::new(app, "カツカレー", false);
    curry.push(app, curry_label);
    let group = CheckboxGroup::new(app, "Select your favorite foods");
    group.add_checkbox_toggled_listener(app, move |event, _, _| {
        println!("checkbox toggled: {} - {}", event.label, event.status);
    });
    group.set_flex_direction(app, FlexDirection::Column);
    group.set_gap(app, px(15.0), px(15.0));
    group.push(app, coffee);
    group.push(app, tea);
    group.push(app, pork);
    group.push(app, curry);
    let heading = title(app, "Checkbox");
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.push(app, heading);
    container.push(app, group);
    container
}

#[cfg(feature = "audio")]
pub fn audio(app: &mut App<GalleryState>) -> Audio {
    let mut asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    asset_path.push("assets");
    asset_path.push("1-11. Mice on Venus.mp3");
    Audio::new(app, Path::new(asset_path.as_path()))
}

#[cfg(not(feature = "audio"))]
pub fn audio(app: &mut App<GalleryState>) -> Container {
    Container::new(app)
}

struct GalleryExample {
    label: &'static str,
    section: Container,
}

impl GalleryExample {
    fn new(app: &mut App<GalleryState>, label: &'static str, child: impl Element) -> Self {
        let section = Container::new(app);
        section.set_display(app, Display::Flex);
        section.set_flex_direction(app, FlexDirection::Column);
        section.set_flex_grow(app, 1.0);
        section.set_width(app, pct(100));
        section.set_height(app, pct(100));
        section.set_padding_all(app, px(32.0));
        section.set_overflow(app, Overflow::Clip, Overflow::Scroll);
        section.push(app, child);
        Self { label, section }
    }

    fn titled(app: &mut App<GalleryState>, label: &'static str, child: impl Element) -> Self {
        let heading = title(app, label);
        let content = Container::new(app);
        content.set_display(app, Display::Flex);
        content.set_flex_direction(app, FlexDirection::Column);
        content.set_row_gap(app, px(12.0));
        content.push(app, heading);
        content.push(app, child);
        Self::new(app, label, content)
    }
}

pub struct GalleryState {
    active: DynElement,
}

impl GalleryState {
    fn select(&mut self, app: &mut App<GalleryState>, target: DynElement) {
        style_navigation_button(app, self.active, false);
        style_navigation_button(app, target, true);
        self.active = target;
    }
}

fn gallery_examples(app: &mut App<GalleryState>) -> Vec<GalleryExample> {
    let animations = animations(app);
    let audio = audio(app);
    let calendar = Calendar::new(app);
    calendar.set_start_year(app, 1950);
    let text_input = text_input(app);
    let dropdown = dropdown(app);
    let text = text(app);
    let variable_fonts = variable_fonts(app);
    let tinyvg = tinyvg(app);
    let images = images(app);
    let gradient = gradient(app);
    let shadows = box_shadows(app);
    let weather = async_weather(app);
    let overlay = overlay(app);
    let sliders = sliders(app);
    let radios = radio_buttons(app);
    let checkboxes = checkbox(app);
    let scrollable = scrollable(app);
    let windows = multiple_windows(app);

    vec![
        GalleryExample::new(app, "Animations", animations),
        GalleryExample::titled(app, "Audio", audio),
        GalleryExample::titled(app, "Calendar", calendar),
        GalleryExample::new(app, "Text Input", text_input),
        GalleryExample::new(app, "Dropdown", dropdown),
        GalleryExample::new(app, "Text", text),
        GalleryExample::new(app, "Variable Fonts", variable_fonts),
        GalleryExample::new(app, "TinyVG", tinyvg),
        GalleryExample::new(app, "Image", images),
        GalleryExample::new(app, "Gradients", gradient),
        GalleryExample::new(app, "Box Shadows", shadows),
        GalleryExample::new(app, "Async", weather),
        GalleryExample::new(app, "Overlay", overlay),
        GalleryExample::new(app, "Sliders", sliders),
        GalleryExample::new(app, "Radio Buttons", radios),
        GalleryExample::new(app, "Checkboxes", checkboxes),
        GalleryExample::new(app, "Scrollable", scrollable),
        GalleryExample::new(app, "Multiple Windows", windows),
    ]
}

fn navigation_background(selected: bool) -> Color {
    if selected {
        Color::from_rgb8(214, 232, 250)
    } else {
        Color::from_rgb8(247, 248, 250)
    }
}

fn style_navigation_button(app: &mut App<GalleryState>, button: impl Element, selected: bool) {
    button.set_background_color(app, navigation_background(selected));
    button.set_outline_color_all(app, retgui::palette::css::DODGER_BLUE);
    button.set_outline_width_all(app, px(if selected { 2.0 } else { 0.0 }));
}

fn navigation_button(app: &mut App<GalleryState>, label: &str, selected: bool) -> Button {
    let label = Text::new(app, label);
    label.set_font_size(app, 15.0);
    label.set_selectable(app, false);
    let button = Button::new(app);
    button.set_display(app, Display::Flex);
    button.set_align_items(app, AlignItems::Center);
    button.set_width(app, pct(100));
    button.set_min_height(app, px(38.0));
    button.set_padding_horizontal(app, px(14.0));
    button.set_border_width_all(app, px(0.0));
    button.set_border_radius_all(app, (5.0, 5.0));
    button.set_background_color(app, navigation_background(selected));
    button.set_outline_color_all(app, retgui::palette::css::DODGER_BLUE);
    button.set_outline_width_all(app, px(if selected { 2.0 } else { 0.0 }));
    button.push(app, label);
    button
}

fn sidebar(app: &mut App<GalleryState>) -> Container {
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_flex_shrink(app, 0.0);
    container.set_width(app, px(220.0));
    container.set_height(app, pct(100));
    container.set_padding(app, px(12.0), px(8.0), px(12.0), px(8.0));
    container.set_row_gap(app, px(3.0));
    container.set_border_width(app, px(0.0), px(1.0), px(0.0), px(0.0));
    container.set_border_color_all(app, Color::from_rgb8(210, 214, 220));
    container.set_background_color(app, navigation_background(false));
    container.set_overflow(app, Overflow::Clip, Overflow::Scroll);
    container
}

fn content_pane(app: &mut App<GalleryState>) -> Container {
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_flex_grow(app, 1.0);
    container.set_width(app, pct(100));
    container.set_height(app, pct(100));
    container.set_overflow(app, Overflow::Clip, Overflow::Clip);
    container
}

fn select_example(app: &mut App<GalleryState>, examples: &[GalleryExample], selected: usize) {
    for (index, example) in examples.iter().enumerate() {
        example.section.set_display(
            app,
            if index == selected {
                Display::Flex
            } else {
                Display::None
            },
        );
    }
}

fn gallery(app: &mut App<GalleryState>) -> (Container, GalleryState) {
    let examples = Rc::new(gallery_examples(app));
    let sidebar = sidebar(app);
    let content = content_pane(app);
    let buttons = examples
        .iter()
        .enumerate()
        .map(|(index, example)| navigation_button(app, example.label, index == 0))
        .collect::<Vec<_>>();
    let state = GalleryState {
        active: buttons
            .first()
            .expect("the gallery must contain at least one example")
            .as_dyn_element(),
    };
    select_example(app, &examples, 0);

    for (index, (example, button)) in examples.iter().zip(buttons).enumerate() {
        let examples = examples.clone();
        button.add_click_listener(app, move |event, app, state| {
            select_example(app, &examples, index);
            state.select(app, event.current_target());
            event.stop_propagation();
        });
        sidebar.push(app, button);
        content.push(app, example.section);
    }

    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_width(app, pct(100));
    container.set_height(app, pct(100));
    container.push(app, sidebar);
    container.push(app, content);
    (container, state)
}

pub fn main() {
    setup_logging();
    let mut app = App::new();
    let (gallery, state) = gallery(&mut app);
    let window = Window::new(&mut app, "Gallery");
    window.set_display(&mut app, Display::Flex);
    window.set_overflow(&mut app, Overflow::Clip, Overflow::Clip);
    window.set_width(&mut app, pct(100));
    window.set_height(&mut app, pct(100));
    window.push(&mut app, gallery);
    retgui_main(app, state, RetGuiOptions::basic("Gallery"));
}
