use retgui::drivers::headless::run;
use retgui::elements::{Element, Radio, RadioGroup, Text, Window};
use retgui::geometry::Size;
use retgui::style::FlexDirection;
use retgui::{RendererType, px};

#[cfg(test)]
mod test_utils;

#[test]
fn switches_from_red_to_green() {
    run(
        "radio_switches_from_red_to_green",
        |app, _states| {
            let group = RadioGroup::new(app, "Color");
            let red_label = Text::new(app, "Red");
            let red = Radio::new(app, group, "red", "red", true);
            red.push(app, red_label);
            let green_label = Text::new(app, "Green");
            let green = Radio::new(app, group, "green", "green", false);
            green.push(app, green_label);
            group.set_flex_direction(app, FlexDirection::Column);
            group.set_gap(app, px(8), px(8));
            group.push(app, red);
            group.push(app, green);
            let window = Window::new_with_renderer(app, "Radio buttons", RendererType::VelloCPU);
            window.set_width(app, px(240));
            window.set_height(app, px(120));
            window.push(app, group);
            (group, green, window)
        },
        |test, (group, green, window)| {
            test.open(&window, Size::new(240.0, 120.0));
            assert_eq!(group.value(test.app()).as_deref(), Some("red"));

            test.click(&green);

            assert_eq!(group.value(test.app()).as_deref(), Some("green"));
            test_utils::check_snapshot(
                test_utils::screenshot_rgb(test, &window),
                "switches_from_red_to_green.png",
            );
        },
    );
}
