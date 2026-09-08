use retgui::elements::{Container, DynElement, Element, Image, Radio, RadioGroup, Slider, SliderDirection, Text, TextInput, TinyVg};
use retgui::style::TextStyleProperty;
use retgui::text::RangedStyles;
use retgui::{App, ResourceId, px};
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
    let root = Container::new(&mut app)
        .builder()
        .id(&mut app, "root")
        .push(
            Container::new(&mut app)
                .builder()
                .id(&mut app, "section")
                .push(
                    Text::new(&mut app, "Title").builder().id(&mut app, "title").build(),
                    &mut app,
                )
                .push(
                    TextInput::new(&mut app, "Input")
                        .builder()
                        .multiline(&mut app, true)
                        .id(&mut app, "input")
                        .build(),
                    &mut app,
                )
                .build(),
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
