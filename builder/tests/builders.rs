use retgui::drivers::headless::{self, HeadlessApp};
use retgui::elements::{Button, Container, DynElement, Element, Image, Radio, RadioGroup, Slider, SliderDirection, Text, TextInput, TinyVg, Window};
use retgui::events::{ClickEvent, Event, SliderValueChangedEvent, TextInputChangedEvent};
use retgui::geometry::Size;
use retgui::style::{FlexDirection, TextStyleProperty};
use retgui::text::RangedStyles;
use retgui::{App, RendererType, ResourceId, px};
use retgui_builder::Builder;
use retgui_builder::prelude::*;
use smol_str::SmolStr;

type TestResult = Result<(), &'static str>;

fn check(condition: bool, message: &'static str) -> TestResult {
    if condition { Ok(()) } else { Err(message) }
}

#[test]
fn mixed_text_chain_preserves_the_handle() -> TestResult {
    struct State;
    let mut app = App::<State>::new();
    let original = Text::new(&mut app, "Before");
    let text = original
        .builder()
        .id(&mut app, "label")
        .text(&mut app, "Intermediate")
        .padding(&mut app, px(1), px(2), px(3), px(4))
        .text_smol_str(&mut app, SmolStr::new("After"))
        .selectable(&mut app, false)
        .font_size(&mut app, 18.0)
        .build();

    check(
        text.as_dyn_element() == original.as_dyn_element(),
        "element handle changed",
    )?;
    check(text.text(&app) == "After", "text was not updated")?;
    check(text.id(&app).as_deref() == Some("label"), "element ID was not updated")?;
    check(!text.is_selectable(&app), "text is still selectable")?;

    text.set_text(&mut app, "Updated");
    check(
        original.text(&app) == "Updated",
        "original handle does not see the update",
    )
}

#[test]
fn builder_does_not_retain_argument_or_app_borrows() -> TestResult {
    let mut app = App::<()>::new();
    let styles = RangedStyles::new(Vec::from([(0..5, TextStyleProperty::FontSize(24.0))]));
    let input = TextInput::new(&mut app, "");
    let temporary = String::from("Hello\nworld");
    let builder = input.builder().multiline(&mut app, true).text(&mut app, &temporary);
    drop(temporary);
    input.set_disabled(&mut app, true);
    let input = builder.width(&mut app, px(200)).ranged_styles(&mut app, styles).build();

    check(input.text(&app) == "Hello\nworld", "borrowed text was not retained")?;
    check(input.is_multiline(&app), "input is not multiline")?;
    check(input.is_disabled(&app), "direct update during construction was lost")
}

#[test]
fn slider_chain_preserves_setter_order_and_clamping() -> TestResult {
    let mut app = App::<()>::new();
    let slider = Slider::new(&mut app, 10.0)
        .builder()
        .max(&mut app, 200.0)
        .min(&mut app, 150.0)
        .width(&mut app, px(300))
        .value(&mut app, 250.0)
        .direction(&mut app, SliderDirection::Vertical)
        .thumb_border_radius(&mut app, (1.0, 2.0), (3.0, 4.0), (5.0, 6.0), (7.0, 8.0))
        .build();

    check(
        slider.min(&app) == 150.0 && slider.max(&app) == 200.0,
        "slider range is incorrect",
    )?;
    check(slider.value(&app) == 200.0, "slider value was not clamped")?;
    check(
        slider.direction(&app) == SliderDirection::Vertical,
        "slider direction is incorrect",
    )?;
    check(
        slider.thumb_border_radius(&app) == Some([(1.0, 2.0), (3.0, 4.0), (5.0, 6.0), (7.0, 8.0)]),
        "border corners are out of order",
    )
}

#[test]
fn radio_group_keeps_selection_when_value_is_unknown() -> TestResult {
    let mut app = App::<()>::new();
    let group = RadioGroup::new(&mut app, "Color");
    let red = Radio::new(&mut app, group, "red", "Red", true);
    let green = Radio::new(&mut app, group, "green", "Green", false);

    let group = group
        .builder()
        .value(&mut app, "green")
        .id(&mut app, "colors")
        .value(&mut app, "unknown")
        .build();

    check(
        group.value(&app).as_deref() == Some("green"),
        "unknown value changed selection",
    )?;
    check(
        !red.is_selected(&app) && green.is_selected(&app),
        "radio selection is inconsistent",
    )?;
    check(group.set_value(&mut app, "red"), "direct setter failed")?;
    check(red.is_selected(&app), "direct setter did not select red")
}

#[test]
fn prelude_resolves_resource_builders_by_element_type() -> TestResult {
    let mut app = App::<()>::new();
    let image_id = ResourceId::StaticBytes(b"image");
    let vector_id = ResourceId::StaticBytes(b"vector");
    let image = Image::dummy(&mut app)
        .builder()
        .resource_id(&mut app, image_id.clone())
        .width(&mut app, px(32))
        .build();
    let vector = TinyVg::dummy(&mut app)
        .builder()
        .width(&mut app, px(32))
        .resource_id(&mut app, vector_id.clone())
        .build();

    check(image.resource_id(&app) == image_id, "image resource is incorrect")?;
    check(vector.resource_id(&app) == vector_id, "vector resource is incorrect")
}

#[test]
fn custom_elements_support_generic_and_downstream_extension_traits() -> TestResult {
    struct CustomContainer(Container);

    impl Copy for CustomContainer {}

    impl Clone for CustomContainer {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl Element for CustomContainer {
        fn as_dyn_element(&self) -> DynElement {
            self.0.as_dyn_element()
        }
    }

    trait CustomContainerBuilder: ElementBuilder<Element = CustomContainer> {
        fn child<S: 'static>(self, app: &mut App<S>, label: &str) -> Self {
            self.push(Text::new(app, label).builder().selectable(app, false).build(), app)
        }
    }

    impl<B: ElementBuilder<Element = CustomContainer>> CustomContainerBuilder for B {}

    fn common_style<S: 'static, B: ElementBuilder>(builder: B, app: &mut App<S>) -> B {
        builder.padding_all(app, px(8)).id(app, "custom")
    }

    let mut app = App::<usize>::new();
    let original = CustomContainer(Container::new(&mut app));
    let container = common_style(Builder::new(original), &mut app)
        .child(&mut app, "Nested builder")
        .build();

    check(
        container.as_dyn_element() == original.as_dyn_element(),
        "custom handle changed",
    )?;
    check(container.id(&app).as_deref() == Some("custom"), "generic setter failed")?;
    check(
        container.children(&app).len() == 1,
        "custom extension did not attach its child",
    )?;

    let erased = container.as_dyn_element().builder().id(&mut app, "erased").build();
    check(erased == original.as_dyn_element(), "erased handle changed")?;
    check(
        container.id(&app).as_deref() == Some("erased"),
        "erased builder did not update the element",
    )
}

#[test]
fn nested_pushes_preserve_child_order_and_parent_relationships() -> TestResult {
    let mut app = App::<usize>::new();
    let mut title = None;
    let root = container(&mut app)
        .id(&mut app, "root")
        .push(
            container(&mut app)
                .id(&mut app, "section")
                .push(
                    text(&mut app, "Title").id(&mut app, "title").capture(&mut title),
                    &mut app,
                )
                .push(
                    text_input(&mut app, "Input")
                        .multiline(&mut app, true)
                        .id(&mut app, "input"),
                    &mut app,
                ),
            &mut app,
        )
        .push(
            Text::new(&mut app, "Footer").builder().id(&mut app, "footer").build(),
            &mut app,
        )
        .font_size(&mut app, 16.0)
        .build();

    let children = root.children(&app);
    check(children.len() == 2, "root child count is incorrect")?;
    check(
        children[0].id(&app).as_deref() == Some("section"),
        "first child is incorrect",
    )?;
    check(
        children[1].id(&app).as_deref() == Some("footer"),
        "second child is incorrect",
    )?;
    for child in &children {
        check(
            child.parent(&app).ok() == Some(root.as_dyn_element()),
            "root parent link is incorrect",
        )?;
    }

    let grandchildren = children[0].children(&app);
    check(grandchildren.len() == 2, "nested child count is incorrect")?;
    check(
        title.ok_or("title was not captured")?.as_dyn_element() == grandchildren[0],
        "captured handle differs from attached child",
    )?;
    check(
        grandchildren[0].id(&app).as_deref() == Some("title"),
        "nested first child is incorrect",
    )?;
    check(
        grandchildren[1].id(&app).as_deref() == Some("input"),
        "nested second child is incorrect",
    )?;
    for child in grandchildren {
        check(
            child.parent(&app).ok() == Some(children[0]),
            "nested parent link is incorrect",
        )?;
    }
    Ok(())
}

#[test]
fn capture_stores_the_handle_immediately_without_retaining_a_borrow() -> TestResult {
    let mut app = App::<()>::new();
    let mut slot = Some(Text::new(&mut app, "Old"));
    let builder = text(&mut app, "New").capture(&mut slot);
    let captured = slot.take().ok_or("capture did not store a handle")?;
    check(captured.text(&app) == "New", "capture did not replace the old handle")?;
    let built = builder.text(&mut app, "Updated").build();
    check(
        captured.as_dyn_element() == built.as_dyn_element(),
        "capture changed the handle",
    )?;
    check(
        captured.text(&app) == "Updated",
        "captured handle does not see later setters",
    )
}

struct ListenerState {
    input: Option<TextInput>,
    slider: Option<Slider>,
    button: Option<Button>,
    text: String,
    value: f64,
    clicks: usize,
    targets: Vec<DynElement>,
}

fn input_changed(event: &mut TextInputChangedEvent, _app: &mut App<ListenerState>, state: &mut ListenerState) {
    state.text = event.value.clone();
    state.targets.push(event.target());
}

fn slider_changed(event: &mut SliderValueChangedEvent, _app: &mut App<ListenerState>, state: &mut ListenerState) {
    state.value = event.value;
    state.targets.push(event.target());
}

fn button_clicked(event: &mut ClickEvent, _app: &mut App<ListenerState>, state: &mut ListenerState) {
    state.clicks += 1;
    state.targets.push(event.target());
}

fn build_listeners(app: &mut App<ListenerState>, state: &mut ListenerState) -> Window {
    Window::new_with_renderer(app, "Listeners", RendererType::Blank)
        .builder()
        .width(app, px(320))
        .height(app, px(240))
        .push(
            container(app)
                .flex_direction(app, FlexDirection::Column)
                .row_gap(app, px(16))
                .push(
                    text_input(app, "Before")
                        .width(app, px(200))
                        .height(app, px(40))
                        .font_size(app, 18.0)
                        .on_text_input_changed(app, input_changed)
                        .capture(&mut state.input),
                    app,
                )
                .push(
                    slider(app, 20.0)
                        .width(app, px(200))
                        .height(app, px(30))
                        .value(app, 0.0)
                        .on_slider_value_changed(app, slider_changed)
                        .capture(&mut state.slider),
                    app,
                )
                .push(
                    button(app)
                        .width(app, px(200))
                        .height(app, px(40))
                        .on_click(app, button_clicked)
                        .capture(&mut state.button)
                        .push(text(app, "Apply"), app),
                    app,
                ),
            app,
        )
        .build()
}

fn check_listeners(test: &mut HeadlessApp<ListenerState>, window: Window) {
    test.open(&window, Size::new(320.0, 240.0));
    let input = test.state().input.expect("input was not captured");
    let slider = test.state().slider.expect("slider was not captured");
    let button = test.state().button.expect("button was not captured");

    input.focus(test.app_mut());
    test.type_text(&window, "After");
    check(test.state().text.contains("After"), "text listener was not called").expect("text input event failed");
    check(
        test.state().text == input.text(test.app()),
        "text event value differs from input",
    )
    .expect("text value failed");
    check(
        test.state().targets.contains(&input.as_dyn_element()),
        "input target is incorrect",
    )
    .expect("input target failed");

    test.click(&slider);
    check(test.state().value > 0.0, "slider listener was not called").expect("slider event failed");
    check(
        test.state().value == slider.value(test.app()),
        "slider event value differs from slider",
    )
    .expect("slider value failed");
    check(
        test.state().targets.contains(&slider.as_dyn_element()),
        "slider target is incorrect",
    )
    .expect("slider target failed");

    test.click(&button);
    check(test.state().clicks == 1, "click listener was not called once").expect("click event failed");
    check(
        test.state().targets.contains(&button.as_dyn_element()),
        "button target is incorrect",
    )
    .expect("button target failed");
}

#[test]
fn named_listeners_receive_events_from_nested_elements() {
    headless::run(
        "builder_listeners",
        ListenerState {
            input: None,
            slider: None,
            button: None,
            text: String::new(),
            value: 0.0,
            clicks: 0,
            targets: Vec::new(),
        },
        build_listeners,
        check_listeners,
    );
}
