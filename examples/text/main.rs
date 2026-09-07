use retgui::elements::{Container, Element, TextInput, Window};
use retgui::style::{AlignItems, Display, FlexDirection, JustifyContent, Overflow, Unit};
use retgui::{App, rgb};

const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In dolor tortor, congue eu lacus eget, faucibus aliquet nunc. Suspendisse aliquet ullamcorper fermentum. Pellentesque eu nibh sit amet nisi maximus pulvinar quis eget lectus. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Curabitur efficitur metus maximus libero maximus pharetra. Sed fringilla ac velit nec hendrerit. Mauris vulputate ante non tempor iaculis. In a diam eros. Mauris vehicula, lacus rhoncus consequat laoreet, lectus ligula venenatis leo, a aliquam odio ex ut lorem. Phasellus quis fermentum erat, ut pellentesque diam. Suspendisse pulvinar eros magna, vel ultricies metus luctus at. Pellentesque consequat a magna ut cursus. Nunc nisl velit, maximus blandit mauris quis, convallis cursus leo. Donec ut ultrices dui, a efficitur sem. Cras ac diam non orci sagittis tincidunt. Etiam auctor ultrices leo vitae vestibulum.

Nunc malesuada eleifend magna eget sollicitudin. Phasellus non posuere justo. Ut viverra posuere molestie. Aenean diam orci, dignissim eu diam vel, viverra suscipit libero. Nam convallis sed arcu porttitor aliquet. Pellentesque urna risus, consectetur bibendum metus vel, laoreet fringilla sapien. Curabitur justo turpis, auctor vel quam quis, volutpat scelerisque magna. Proin ornare, turpis ac eleifend consequat, augue nulla interdum nulla, a scelerisque mi tortor a nulla. Aenean ut sollicitudin quam.

Ut ac magna dolor. Etiam tempor varius arcu. Sed sit amet convallis quam. Phasellus varius vestibulum condimentum. Sed felis nulla, vehicula tempor fringilla eget, cursus sit amet metus. Nunc imperdiet metus nec ante porttitor tristique. In venenatis tortor sed aliquam dignissim. Donec tempus mollis enim in volutpat. Nam tincidunt sed mauris ut facilisis. Pellentesque tempus dolor at maximus vehicula. Sed nec facilisis nibh. Praesent molestie porttitor sem scelerisque malesuada. In pulvinar malesuada elit, ut ultricies nunc faucibus vel.

Morbi tincidunt porta scelerisque. Etiam sodales, leo eget molestie imperdiet, libero enim ullamcorper erat, eget iaculis nisi purus a metus. Proin ante elit, eleifend at gravida vel, accumsan vitae felis. Nullam id dolor vel felis faucibus aliquam vitae et purus. Curabitur malesuada, sapien vitae rutrum imperdiet, orci lorem imperdiet metus, at consequat justo libero non dui. Nam ac orci turpis. Aenean tristique urna velit. Vivamus laoreet ex sed dapibus mollis. Integer ante lorem, tincidunt nec pulvinar eu, auctor vel sem. Nulla non ex vitae dui viverra ultrices. In quis est enim. Duis consectetur mauris tortor, non consectetur metus aliquam sit amet. Vestibulum auctor tincidunt luctus. Aliquam aliquet dictum diam, in volutpat nunc egestas eget. Praesent dui leo, posuere in fringilla quis, congue nec nunc.

Nunc tellus magna, varius eu ornare et, sodales hendrerit quam. Praesent nec magna finibus, elementum orci nec, facilisis nisi. Duis ligula mi, dapibus eget nibh a, posuere viverra ante. Aliquam efficitur mauris id quam faucibus, eget posuere turpis imperdiet. Nam vulputate sed urna vitae tincidunt. Nulla ligula urna, iaculis id urna sit amet, porta iaculis ligula. Maecenas volutpat odio at pretium commodo. Nullam faucibus efficitur neque, vitae elementum sem sollicitudin eu. Nullam rutrum nulla eu erat dignissim varius. ";

pub fn text(app: &mut App) -> Container {
    let input = TextInput::new(app, LOREM_IPSUM);
    input.set_overflow_y(app, Overflow::Scroll);
    input.set_width(app, Unit::Px(600.0));
    input.set_height(app, Unit::Px(600.0));
    input.set_display(app, Display::Block);
    let container = Container::new(app);
    container.set_display(app, Display::Flex);
    container.set_flex_direction(app, FlexDirection::Column);
    container.set_justify_content(app, JustifyContent::Center);
    container.set_align_items(app, AlignItems::Center);
    container.set_width(app, Unit::Percentage(100.0));
    container.set_height(app, Unit::Percentage(100.0));
    container.set_row_gap(app, Unit::Px(20.0));
    container.set_font_size(app, 72.0);
    container.set_color(app, rgb(50, 50, 50));
    container.push(app, input);
    container
}

pub fn main() {
    let mut app = App::new();
    let content = text(&mut app);
    let window = Window::new(&mut app, "Text");
    window.push(&mut app, content);
    use retgui::RetGuiOptions;

    util::setup_logging();
    retgui::retgui_main(app, RetGuiOptions::basic("text"));
}
