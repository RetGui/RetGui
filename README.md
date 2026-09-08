<p align="center"><img src="./images/retgui_logo.svg" alt="The RetGui logo" width="40%"></p>

<p align="center">
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-Unlicense-blue.svg" alt="License: Unlicense"></a>
  <a href="https://discord.gg/Atb8nuAub2"><img src="https://img.shields.io/discord/1382383100562243746?logo=discord&logoColor=%23ffffff&labelColor=%236A7EC2&color=%237389D8" alt="Discord"></a>
</p>

## Introduction
RetGui is a Rust library for creating desktop user interfaces.

## Installation
Add the following to your `Cargo.toml`:
```toml
[dependencies.retgui]
git = "https://github.com/RetGui/RetGui"
default-features = false
features = ["system_fonts", "vello_hybrid_renderer"]
```

## Example

```rust
use retgui::elements::{Container, Element, Text, Window};
use retgui::events::Event;
use retgui::style::{AlignItems, FlexDirection, JustifyContent};
use retgui::{App, Color, RetGuiOptions, pct, px, rgb};

fn create_button(
    app: &mut App<i64>,
    label: &str,
    base_color: Color,
    delta: i64,
    count_text: Text,
) -> Container {
    let label = Text::new(app, label);
    label.set_font_size(app, 24.0);
    label.set_color(app, Color::WHITE);
    label.set_selectable(app, false);

    let button = Container::new(app);
    button.set_border_width(app, px(1), px(2), px(3), px(4));
    button.set_border_color_all(app, rgb(0, 0, 0));
    button.set_border_radius_all(app, (10.0, 10.0));
    button.set_padding(app, px(15), px(30), px(15), px(30));
    button.set_justify_content(app, JustifyContent::Center);
    button.set_background_color(app, base_color);
    button.add_click_listener(app, move |event, app, count| {
        *count += delta;
        count_text.set_text(app, &format!("Count: {count}"));
        event.stop_propagation();
    });
    button.push(app, label);
    button
}

fn main() {
    let mut app = App::new();
    let count_text = Text::new(&mut app, "Count: 0");
    let subtract = create_button(
        &mut app, "-", rgb(244, 67, 54), -1, count_text,
    );
    let add = create_button(
        &mut app, "+", rgb(76, 175, 80), 1, count_text,
    );
    let buttons = Container::new(&mut app);
    buttons.set_gap(&mut app, px(20), px(20));
    buttons.push(&mut app, subtract);
    buttons.push(&mut app, add);

    let window = Window::new(&mut app, "Counter");
    window.set_flex_direction(&mut app, FlexDirection::Column);
    window.set_justify_content(&mut app, JustifyContent::Center);
    window.set_align_items(&mut app, AlignItems::Center);
    window.set_width(&mut app, pct(100));
    window.set_height(&mut app, pct(100));
    window.set_gap(&mut app, px(20), px(20));
    window.push(&mut app, count_text);
    window.push(&mut app, buttons);

    retgui::retgui_main(app, 0_i64, RetGuiOptions::basic("Counter"));
}
```

## Supported Platforms
| Platform | Status                             | Accessibility Status |
|----------|------------------------------------|----------------------|
| Windows  | Officially supported               | In Progress          |
| macOS    | Officially supported               | Planned              |
| Linux    | Officially supported               | Planned              |
| Web      | Runs, but not officially supported | TBD                  |
| Android  | Runs, but not officially supported | TBD                  |
| iOS      | Runs, but not officially supported | TBD                  |

## Features
| Feature               | Description                                                                 | Platforms Not Supported |
|-----------------------|-----------------------------------------------------------------------------|-------------------------|
| audio                 | Enables playing audio via MiniAudio.                                        | Web, Android, and iOS   |
| clipboard             | Enables clipboard support in text elements.                                 |                         |
| vello_cpu_renderer    | Enables the Vello CPU renderer.                                             |                         |
| vello_hybrid_renderer | Enables the Vello Hybrid renderer.                                          |                         |
| http_client           | Enables the HTTP client, which allows loading resources from URLs and more. |                         |
| system_fonts          | Tells the font engine to load system fonts automatically.                   |                         |
| png                   | Enables decoding PNG images.                                                |                         |
| jpeg                  | Enables decoding JPEG images                                                |                         |
| markdown              | Provides the ability to render markdown via an element.                     |                         |
| link                  | Allows opening links with your default browser.                             |                         |

## Showcase
<p>
  <img src="./images/gallery.png" alt="The RetGui gallery example." width="40%">
  <img src="./images/counter.png" alt="The RetGui counter example." width="40%">
</p>

## FAQ
### 1. Are Android, iOS, and the Web Supported? 
RetGui can run on those platforms, but they are not officially supported. We would like to support those platforms, but it requires a lot of platform integration. Please use SwiftUI, Jetpack Compose, Flutter, and etc.

### 2. Why do mutations need `App`?
App owns RetGui elements, and each element is a handle to its data in App. Consumer state lives in a separate States store, so event handlers can borrow their state and mutate the UI at the same time using ordinary Rust references. Built-in widgets own their data directly: radio selection and audio playback do not use the consumer state store.

## License
Distributed under the Unlicense License. See the [LICENSE](./LICENSE) for more information.
