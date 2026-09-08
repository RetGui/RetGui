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
use retgui::elements::{Button, Container, Element, Text};
use retgui::events::Event;
use retgui::style::{AlignItems, FlexDirection, JustifyContent};
use retgui::{App, Color, RetGuiOptions, pct, px, retgui_main, rgb};

use retgui_builder::{ElementBuilder, TextBuilder, button, container, text, window};

fn create_button(
    app: &mut App<i64>,
    label: &str,
    accessibility_name: &str,
    base_color: Color,
    delta: i64,
    count_text: Text,
) -> Button {
    let button = button(app)
        .accessibility_name(app, accessibility_name)
        .border_radius_all(app, (8.0, 8.0))
        .padding(app, px(12), px(30), px(12), px(30))
        .justify_content(app, JustifyContent::Center)
        .background_color(app, base_color)
        .push(
            text(app, label)
                .font_size(app, 24.0)
                .color(app, Color::WHITE)
                .selectable(app, false),
            app,
        )
        .build();
    button.add_click_listener(app, move |event, app, count| {
        *count += delta;
        count_text.set_text(app, &format!("Count: {count}"));
        event.stop_propagation();
    });
    button
}

fn counter(app: &mut App<i64>) -> Container {
    let count_text = text(app, "Count: 0").build();
    container(app)
        .flex_direction(app, FlexDirection::Column)
        .justify_content(app, JustifyContent::Center)
        .align_items(app, AlignItems::Center)
        .width(app, pct(100))
        .height(app, pct(100))
        .row_gap(app, px(24))
        .font_size(app, 28.0)
        .color(app, rgb(63, 63, 70))
        .background_color(app, Color::WHITE)
        .push(count_text, app)
        .push(
            container(app)
                .column_gap(app, px(16))
                .push(
                    create_button(app, "−", "Decrement", rgb(244, 63, 94), -1, count_text),
                    app,
                )
                .push(
                    create_button(app, "+", "Increment", rgb(16, 185, 129), 1, count_text),
                    app,
                ),
            app,
        )
        .build()
}

pub fn main() {
    let mut app = App::new();
    window(&mut app, "Counter")
        .width(&mut app, pct(100))
        .height(&mut app, pct(100))
        .push(counter(&mut app), &mut app);
    retgui_main(app, 0_i64, RetGuiOptions::basic("Counter"));
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
