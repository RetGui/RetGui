//! A calendar.

use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

use retgui_calendar::sys_locale::get_locale_or_default;
use retgui_calendar::{
    DateAddOptions, DateDuration, Locale, Month, Weekday, current_calendar_start, current_month, day_abbreviation,
    first_day_of_week, format_date_day_number, month_name, year_name,
};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use rustc_hash::FxHashMap;

use crate::elements::button::ButtonElement;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{
    apply_generic_container_layout, draw_generic_container, push_child_to_element,
};
use crate::elements::traits::{clone_element, subtree_clone_pairs};
use crate::elements::{
    Button, Container, ContainerElement, Dropdown, DropdownElement, DynElement, Element, ElementIds, ElementInternals,
    RetGuiAccessTree, RetainedElements, Text, TextElement,
};
use crate::events::{CalendarDateSelectedEvent, Event, EventCallbackKind, EventKind, EventListenerOptions};
use crate::layout::GummyTree;
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit};
use crate::text::text_context::TextContext;
use crate::{App, px, rgb};

#[derive(Clone, Copy)]
pub struct Calendar {
    pub(crate) inner: DynElement,
}

/// A calendar.
#[derive(Clone)]
pub(crate) struct CalendarElement {
    element_data: ElementData,
    pub first_day: Weekday,
    pub nav: Container,
    pub day_header: Container,
    pub week_grid: Container,
    pub days: Vec<Text>,
    pub day_cells: Vec<Button>,
    pub day_dates: Vec<jiff::civil::Date>,
    pub selected_date: Option<jiff::civil::Date>,
    pub year_dropdown: Dropdown,
    pub month_dropdown: Dropdown,
    pub focus_year: i32,
    pub focus_month: u8,
    pub start_year: i32,
    pub end_year: i32,
    pub locale: Locale,
}

const ROWS: usize = 6;
const COLUMNS: usize = 7;
const CELL_SIZE: Unit = Unit::Px(36.0);
const MIN_YEAR: i32 = 1900;
const MAX_YEAR: i32 = 3000;

impl Element for Calendar {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for CalendarElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for CalendarElement {
    fn deep_clone(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        let cloned = clone_element::<Self, _>(self, elements, gummy_tree, access_tree, by_internal_id, |_, _| None);
        let cloned_children = &elements.get(cloned).element_data().children;
        let cloned_by_source: FxHashMap<_, _> = self
            .element_data
            .children
            .iter()
            .zip(cloned_children)
            .flat_map(|(&source, &cloned)| subtree_clone_pairs(elements, source, cloned))
            .collect();
        let calendar = elements.get_as_mut::<Self>(cloned);
        calendar.nav.inner = cloned_by_source[&self.nav.inner];
        calendar.day_header.inner = cloned_by_source[&self.day_header.inner];
        calendar.week_grid.inner = cloned_by_source[&self.week_grid.inner];
        calendar.year_dropdown.inner = cloned_by_source[&self.year_dropdown.inner];
        calendar.month_dropdown.inner = cloned_by_source[&self.month_dropdown.inner];
        for day in &mut calendar.days {
            day.inner = cloned_by_source[&day.inner];
        }
        for cell in &mut calendar.day_cells {
            cell.inner = cloned_by_source[&cell.inner];
        }
        cloned
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(self, gummy_tree, z_index, scale_factor);
    }

    fn draw(
        &self,
        elements: &RetainedElements,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, elements, renderer, resource_manager, text_context, scale_factor);
    }

    fn on_event(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        focus_outline_visible: bool,
        _pending_animation_updates: &mut Vec<(DynElement, bool)>,
        event: &mut EventKind,
        _text_context: &mut TextContext,
    ) {
        if let EventKind::Click(click) = event {
            self.handle_day_click(
                elements,
                gummy_tree,
                access_tree,
                by_internal_id,
                event_queue,
                focus,
                focus_outline_visible,
                click.target(),
            );
        }
        let year_id = elements.get(self.year_dropdown.inner).element_data().internal_id;
        let month_id = elements.get(self.month_dropdown.inner).element_data().internal_id;
        if let EventKind::DropdownItemSelected(dropdown_event) = event {
            let target_id = elements.get(dropdown_event.target()).element_data().internal_id;
            if target_id == year_id {
                self.select_year(elements, gummy_tree, dropdown_event.index);
            } else if target_id == month_id {
                self.select_month(elements, gummy_tree, dropdown_event.index);
            }
        }
    }
}

impl Calendar {
    pub fn new<S: 'static>(app: &mut App<S>) -> Self {
        let App {
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            event_queue,
            focus,
            ..
        } = app;
        let locale = get_locale_or_default();
        let first_day = first_day_of_week(&locale);
        let start_of_month = current_month();
        let week_grid = Container {
            inner: ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id),
        };
        elements.get_mut(week_grid.inner).set_display(gummy_tree, Display::Flex);
        elements
            .get_mut(week_grid.inner)
            .set_flex_direction(gummy_tree, FlexDirection::Column);
        let day_header = Container {
            inner: ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id),
        };
        let nav = Container {
            inner: ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id),
        };
        let year_dropdown = Dropdown {
            inner: DropdownElement::insert(elements, gummy_tree, access_tree, by_internal_id),
        };
        elements.get_mut(year_dropdown.inner).set_width(gummy_tree, px(100));
        let month_dropdown = Dropdown {
            inner: DropdownElement::insert(elements, gummy_tree, access_tree, by_internal_id),
        };
        elements.get_mut(month_dropdown.inner).set_width(gummy_tree, px(100));
        let inner = elements.insert_with(access_tree, by_internal_id, |me, access_tree| {
            Box::new(CalendarElement {
                element_data: ElementData::new(me, true, access_tree),
                week_grid,
                days: Vec::new(),
                day_cells: Vec::new(),
                day_dates: Vec::new(),
                selected_date: None,
                focus_year: start_of_month.year().extended_year(),
                day_header,
                first_day,
                nav,
                focus_month: start_of_month.month().ordinal,
                year_dropdown,
                month_dropdown,
                start_year: MIN_YEAR,
                end_year: MAX_YEAR,
                locale,
            })
        });
        elements
            .get_mut(inner)
            .element_data_mut()
            .create_layout_node(gummy_tree, None);
        elements.dispatch_mut(inner, |inner_value, elements| {
            let inner = (inner_value as &mut dyn std::any::Any)
                .downcast_mut::<CalendarElement>()
                .unwrap();
            inner.setup_years(elements, gummy_tree, access_tree, by_internal_id, event_queue, focus);
            inner.setup_months(elements, gummy_tree, access_tree, by_internal_id, event_queue, focus);
            let mut current_header_day = inner.first_day;
            for _ in 0..COLUMNS {
                let label = day_abbreviation(&inner.locale, current_header_day);
                let text = Text {
                    inner: TextElement::insert(elements, gummy_tree, access_tree, by_internal_id, label.as_str()),
                };
                elements.get_as_mut::<TextElement>(text.inner).set_selectable(false);
                let day = ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id);
                elements.get_mut(day).set_display(gummy_tree, Display::Flex);
                elements
                    .get_mut(day)
                    .set_justify_content(gummy_tree, JustifyContent::Center);
                elements.get_mut(day).set_align_items(gummy_tree, AlignItems::Center);
                push_child_to_element(elements, gummy_tree, day, text.inner);
                elements.get_mut(day).set_width(gummy_tree, CELL_SIZE);
                elements.get_mut(day).set_height(gummy_tree, CELL_SIZE);
                push_child_to_element(elements, gummy_tree, inner.day_header.inner, day);
                current_header_day = Weekday::from_days_since_sunday(current_header_day as isize + 1);
            }
            for _ in 0..ROWS {
                let week = ContainerElement::create(elements, gummy_tree, access_tree, by_internal_id);
                elements.get_mut(week).set_display(gummy_tree, Display::Flex);
                elements
                    .get_mut(week)
                    .set_flex_direction(gummy_tree, FlexDirection::Row);
                for _ in 0..COLUMNS {
                    let text = Text {
                        inner: TextElement::insert(elements, gummy_tree, access_tree, by_internal_id, ""),
                    };
                    elements.get_as_mut::<TextElement>(text.inner).set_selectable(false);
                    let day = ButtonElement::insert(elements, gummy_tree, access_tree, by_internal_id);
                    elements.get_mut(day).set_display(gummy_tree, Display::Flex);
                    elements
                        .get_mut(day)
                        .set_justify_content(gummy_tree, JustifyContent::Center);
                    elements.get_mut(day).set_align_items(gummy_tree, AlignItems::Center);
                    elements.get_mut(day).set_width(gummy_tree, CELL_SIZE);
                    elements.get_mut(day).set_height(gummy_tree, CELL_SIZE);
                    push_child_to_element(elements, gummy_tree, day, text.inner);
                    push_child_to_element(elements, gummy_tree, week, day);
                    inner.days.push(text);
                    inner.day_cells.push(Button { inner: day });
                }
                push_child_to_element(elements, gummy_tree, inner.week_grid.inner, week);
            }
            inner.update_calendar(elements, gummy_tree);
            inner.set_display(gummy_tree, Display::Flex);
            inner.set_flex_direction(gummy_tree, FlexDirection::Column);
            let nav = elements.get_mut(inner.nav.inner);
            nav.set_display(gummy_tree, Display::Flex);
            nav.set_justify_content(gummy_tree, JustifyContent::SpaceAround);
            nav.set_align_items(gummy_tree, AlignItems::Center);
            nav.set_flex_direction(gummy_tree, FlexDirection::Row);
            nav.set_width(gummy_tree, px(CELL_SIZE.raw_value() * 7.0));
            push_child_to_element(elements, gummy_tree, inner.nav.inner, inner.year_dropdown.inner);
            push_child_to_element(elements, gummy_tree, inner.nav.inner, inner.month_dropdown.inner);
        });
        push_child_to_element(elements, gummy_tree, inner, nav.inner);
        push_child_to_element(elements, gummy_tree, inner, day_header.inner);
        push_child_to_element(elements, gummy_tree, inner, week_grid.inner);
        Self { inner }
    }

    pub fn selected_date<S: 'static>(&self, app: &App<S>) -> Option<jiff::civil::Date> {
        app.elements.get_as::<CalendarElement>(self.inner).selected_date
    }

    pub fn visible_dates<S: 'static>(&self, app: &App<S>) -> Vec<jiff::civil::Date> {
        app.elements.get_as::<CalendarElement>(self.inner).day_dates.clone()
    }

    pub fn set_selected_date<S: 'static>(&self, app: &mut App<S>, date: Option<jiff::civil::Date>) {
        if !app.contains(self.inner) {
            return;
        }
        let calendar = app.elements.get_as::<CalendarElement>(self.inner);
        if date.is_some_and(|date| !(calendar.start_year..=calendar.end_year).contains(&(date.year() as i32))) {
            return;
        }
        app.elements.dispatch_mut(self.inner, |element, elements| {
            let calendar = (element as &mut dyn std::any::Any)
                .downcast_mut::<CalendarElement>()
                .unwrap();
            calendar.select_date(
                elements,
                &mut app.gummy_tree,
                &app.access_tree,
                &mut app.by_internal_id,
                date,
            );
        });
    }

    pub fn add_date_selected_listener<S: 'static>(
        &self,
        app: &mut App<S>,
        handler: impl Fn(&mut CalendarDateSelectedEvent, &mut App<S>, &mut S) + 'static,
    ) {
        self.add_event_listener(
            app,
            EventCallbackKind::CalendarDateSelected(Rc::new(handler)),
            EventListenerOptions::default(),
        );
    }

    pub fn set_start_year<S: 'static>(&self, app: &mut App<S>, year: i32) {
        if !app.contains(self.inner) {
            return;
        }
        if year < MIN_YEAR {
            panic!("Dates below {MIN_YEAR} are not supported.");
        }
        if year > MAX_YEAR {
            panic!("Dates above {MAX_YEAR} are not supported.");
        }
        app.elements.try_dispatch_mut(self.inner, |inner, arena| {
            (inner as &mut dyn std::any::Any)
                .downcast_mut::<CalendarElement>()
                .unwrap()
                .set_start_year(
                    arena,
                    &mut app.gummy_tree,
                    &app.access_tree,
                    &mut app.by_internal_id,
                    &mut app.event_queue,
                    &mut app.focus,
                    year,
                )
        });
    }

    pub fn set_end_year<S: 'static>(&self, app: &mut App<S>, year: i32) {
        if !app.contains(self.inner) {
            return;
        }
        if year < MIN_YEAR {
            panic!("Dates below {MIN_YEAR} are not supported.");
        }
        if year > MAX_YEAR {
            panic!("Dates above {MAX_YEAR} are not supported.");
        }
        app.elements.try_dispatch_mut(self.inner, |inner, arena| {
            (inner as &mut dyn std::any::Any)
                .downcast_mut::<CalendarElement>()
                .unwrap()
                .set_end_year(
                    arena,
                    &mut app.gummy_tree,
                    &app.access_tree,
                    &mut app.by_internal_id,
                    &mut app.event_queue,
                    &mut app.focus,
                    year,
                )
        });
    }
}

impl CalendarElement {
    #[allow(clippy::too_many_arguments)]
    fn handle_day_click(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        focus_outline_visible: bool,
        target: DynElement,
    ) {
        let Some(index) = self
            .day_cells
            .iter()
            .zip(&self.days)
            .position(|(cell, text)| target == cell.inner || target == text.inner)
        else {
            return;
        };
        let date = self.day_dates[index];
        if !(self.start_year..=self.end_year).contains(&(date.year() as i32)) {
            elements
                .get_mut(self.day_cells[index].inner)
                .unfocus(event_queue, focus);
            return;
        }

        self.select_date(elements, gummy_tree, access_tree, by_internal_id, Some(date));
        let selected_index = self.day_dates.iter().position(|shown| *shown == date).unwrap();
        elements.dispatch_mut(self.day_cells[selected_index].inner, |cell, elements| {
            cell.focus(elements, event_queue, focus, focus_outline_visible);
        });
        event_queue.push_back(EventKind::CalendarDateSelected(CalendarDateSelectedEvent::new(
            self.element_data.me,
            date,
        )));
    }

    fn select_date(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        date: Option<jiff::civil::Date>,
    ) {
        self.selected_date = date;
        if let Some(date) = date {
            self.focus_year = date.year() as i32;
            self.focus_month = date.month() as u8;
            for (dropdown, index) in [
                (self.year_dropdown, (self.focus_year - self.start_year) as usize),
                (self.month_dropdown, (self.focus_month - 1) as usize),
            ] {
                elements.dispatch_mut(dropdown.inner, |element, elements| {
                    (element as &mut dyn std::any::Any)
                        .downcast_mut::<DropdownElement>()
                        .unwrap()
                        .set_selected_element(elements, gummy_tree, access_tree, by_internal_id, index);
                });
            }
        }
        self.update_calendar(elements, gummy_tree);
    }

    fn update_calendar(&mut self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree) {
        self.update_calendar_for_today(elements, gummy_tree, jiff::Zoned::now().date());
    }

    fn update_calendar_for_today(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        today: jiff::civil::Date,
    ) {
        let mut start_date = current_calendar_start(self.first_day, self.focus_year, Month::new(self.focus_month));
        self.day_dates.clear();
        for (day_element, cell) in self.days.iter().zip(&self.day_cells) {
            let date = jiff::civil::Date::new(
                start_date.year().extended_year() as i16,
                start_date.month().ordinal as i8,
                start_date.day_of_month().0 as i8,
            )
            .unwrap();
            self.day_dates.push(date);
            let selected = self.selected_date == Some(date);
            let is_today = date == today;
            let enabled = (self.start_year..=self.end_year).contains(&(date.year() as i32));
            let cell_element = elements.get_mut(cell.inner);
            cell_element.set_background_brush(crate::Brush::Color(if selected {
                rgb(38, 67, 101)
            } else {
                crate::Color::TRANSPARENT
            }));
            cell_element.set_border_radius_all((4.0, 4.0));
            // Reserve the border on every cell so today's marker never shifts the grid.
            cell_element.set_border_width_all(gummy_tree, px(2));
            cell_element.set_border_color_all(if is_today {
                if selected { rgb(100, 230, 210) } else { rgb(0, 130, 110) }
            } else {
                crate::Color::TRANSPARENT
            });
            let mut label = date.to_string();
            if is_today {
                label.push_str(", today");
            }
            if selected {
                label.push_str(", selected");
            }
            cell_element.element_data_mut().set_accessibility_name(label);
            cell_element.element_data_mut().set_accessibility_enabled(enabled);

            let is_in_current_month = start_date.month().ordinal == self.focus_month;
            let date_str = format_date_day_number(&self.locale, &start_date);
            let day = elements.get_as_mut::<TextElement>(day_element.inner);
            day.set_text(gummy_tree, date_str.as_str());
            day.set_text_brush(
                gummy_tree,
                crate::Brush::Color(if selected {
                    rgb(255, 255, 255)
                } else if !enabled {
                    rgb(170, 170, 170)
                } else if is_in_current_month {
                    rgb(0, 0, 0)
                } else {
                    rgb(120, 120, 120)
                }),
            );
            start_date
                .try_add_with_options(DateDuration::for_days(1), DateAddOptions::default())
                .unwrap()
        }
    }

    fn select_year(&mut self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree, year: usize) {
        self.focus_year = self.start_year + (year as i32);
        self.update_calendar(elements, gummy_tree);
    }

    fn select_month(&mut self, elements: &mut RetainedElements, gummy_tree: &mut GummyTree, month: usize) {
        self.focus_month = 1 + month as u8;
        self.update_calendar(elements, gummy_tree);
    }

    fn setup_years(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
    ) {
        let dropdown = self.year_dropdown.inner;
        elements.dispatch_mut(dropdown, |dropdown, elements| {
            dropdown.remove_all_children(elements, gummy_tree, event_queue, focus);
        });
        for year in self.start_year..=self.end_year {
            let text = TextElement::insert(
                elements,
                gummy_tree,
                access_tree,
                by_internal_id,
                &year_name(&self.locale, year),
            );
            elements.get_as_mut::<TextElement>(text).set_selectable(false);
            push_child_to_element(elements, gummy_tree, dropdown, text);
            elements.get_mut(dropdown).set_font_size(gummy_tree, 20.0);
            if year == self.focus_year {
                elements.dispatch_mut(dropdown, |dropdown, elements| {
                    (dropdown as &mut dyn std::any::Any)
                        .downcast_mut::<DropdownElement>()
                        .unwrap()
                        .set_selected_element(
                            elements,
                            gummy_tree,
                            access_tree,
                            by_internal_id,
                            (year - self.start_year) as usize,
                        );
                });
            }
        }
    }

    fn setup_months(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
    ) {
        let dropdown = self.month_dropdown.inner;
        elements.dispatch_mut(dropdown, |dropdown, elements| {
            dropdown.remove_all_children(elements, gummy_tree, event_queue, focus);
        });
        for month in 0..12 {
            let text = TextElement::insert(
                elements,
                gummy_tree,
                access_tree,
                by_internal_id,
                &month_name(&self.locale, Month::new(month + 1), self.focus_year),
            );
            elements.get_as_mut::<TextElement>(text).set_selectable(false);
            push_child_to_element(elements, gummy_tree, dropdown, text);
            elements.get_mut(dropdown).set_font_size(gummy_tree, 20.0);
            if month + 1 == self.focus_month {
                elements.dispatch_mut(dropdown, |dropdown, elements| {
                    (dropdown as &mut dyn std::any::Any)
                        .downcast_mut::<DropdownElement>()
                        .unwrap()
                        .set_selected_element(elements, gummy_tree, access_tree, by_internal_id, month as usize);
                });
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_start_year(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        year: i32,
    ) {
        if year > self.end_year {
            panic!("Invalid start year");
        }
        self.start_year = year;
        self.focus_year = self.focus_year.max(year);
        if self.selected_date.is_some_and(|date| (date.year() as i32) < year) {
            self.selected_date = None;
        }
        self.setup_years(elements, gummy_tree, access_tree, by_internal_id, event_queue, focus);
        self.update_calendar(elements, gummy_tree);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_end_year(
        &mut self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        year: i32,
    ) {
        if year < self.start_year {
            panic!("Invalid end year");
        }
        self.end_year = year;
        self.focus_year = self.focus_year.min(year);
        if self.selected_date.is_some_and(|date| (date.year() as i32) > year) {
            self.selected_date = None;
        }
        self.setup_years(elements, gummy_tree, access_tree, by_internal_id, event_queue, focus);
        self.update_calendar(elements, gummy_tree);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{DropdownItemSelectedEvent, EventDispatcher};

    #[test]
    fn calendar_rebuilds_years_and_updates_days_from_dropdown_events() {
        let mut app = App::new();
        let mut state = ();
        let calendar = Calendar::new(&mut app);
        let year_dropdown = app.get_as::<CalendarElement>(calendar.inner).year_dropdown;
        let month_dropdown = app.get_as::<CalendarElement>(calendar.inner).month_dropdown;
        let old_years = year_dropdown.children(&app);

        calendar.set_end_year(&mut app, 2030);
        calendar.set_start_year(&mut app, 2020);
        assert_eq!(year_dropdown.children(&app).len(), 11);
        assert!(
            old_years
                .iter()
                .all(|year| app.contains(*year) && year.parent(&app).is_err())
        );

        for (dropdown, index) in [(year_dropdown, 4), (month_dropdown, 0)] {
            app.event_queue
                .push_back(EventKind::DropdownItemSelected(DropdownItemSelectedEvent::new(
                    dropdown.inner,
                    index,
                )));
        }
        EventDispatcher::dispatch_queued_events(&mut app, &mut state);
        let calendar_element = app.get_as::<CalendarElement>(calendar.inner);
        assert_eq!((calendar_element.focus_year, calendar_element.focus_month), (2024, 1));
        let days = calendar_element.days.clone();
        assert_eq!(days.len(), ROWS * COLUMNS);
        let january: Vec<_> = days.iter().map(|day| day.text(&app)).collect();
        assert!(january.iter().all(|day| !day.is_empty()));

        app.event_queue
            .push_back(EventKind::DropdownItemSelected(DropdownItemSelectedEvent::new(
                month_dropdown.inner,
                1,
            )));
        EventDispatcher::dispatch_queued_events(&mut app, &mut state);
        assert_eq!(app.get_as::<CalendarElement>(calendar.inner).focus_month, 2);
        assert_ne!(january, days.iter().map(|day| day.text(&app)).collect::<Vec<_>>());
    }
}
