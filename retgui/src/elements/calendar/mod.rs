//! A calendar.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use jiff::civil::Date;

use retgui_calendar::sys_locale::get_locale_or_default;
use retgui_calendar::{DateAddOptions, DateDuration, Locale, Month, Weekday, current_calendar_start, current_month, day_abbreviation, first_day_of_week, format_date_day_number, month_name, year_name};

use retgui_renderer::RenderList;

use retgui_resource_manager::ResourceManager;

use crate::app::queue_event;
use crate::elements::dropdown::DropdownInner;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::builder::ElementBuilder;
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, Button, Container, Dropdown, DynElement, Element, ElementInternals, Text, TextInner};
use crate::events::{CalendarDateSelectedEvent, Event, EventCallbackKind, EventKind, EventListenerOptions};
use crate::layout::GummyTree;
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit};
use crate::text::text_context::TextContext;
use crate::{px, rgb};

#[derive(Clone)]
pub struct Calendar {
    pub(crate) inner: Rc<RefCell<CalendarInner>>,
}

/// A calendar.
#[derive(Clone)]
pub(crate) struct CalendarInner {
    element_data: ElementData,
    pub first_day: Weekday,
    pub nav: DynElement,
    pub day_header: DynElement,
    pub week_grid: DynElement,
    pub days: Vec<DynElement>,
    day_cells: Vec<DynElement>,
    day_dates: Vec<Date>,
    selected_date: Option<Date>,
    selected_index: Option<usize>,
    day_labels: Vec<String>,
    today_index: Option<usize>,
    pub year_dropdown: DynElement,
    pub month_dropdown: DynElement,
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

impl Default for Calendar {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Calendar {}

impl Drop for CalendarInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Calendar {
    fn with<R>(&self, callback: impl FnOnce(&dyn ElementInternals) -> R) -> R {
        callback(&*self.inner.borrow())
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut dyn ElementInternals) -> R) -> R {
        callback(&mut *self.inner.borrow_mut())
    }
}

impl crate::elements::ElementData for CalendarInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for CalendarInner {
    fn deep_clone(&self) -> DynElement {
        let cloned = clone_element::<Self, _>(self, |_, _| None);
        fn collect(source: &[DynElement], cloned: &[DynElement], map: &mut std::collections::HashMap<u64, DynElement>) {
            for (source, cloned) in source.iter().zip(cloned) {
                map.insert(source.borrow().element_data().internal_id, cloned.clone());
                collect(
                    &source.borrow().element_data().children,
                    &cloned.borrow().element_data().children,
                    map,
                );
            }
        }
        let mut map = std::collections::HashMap::new();
        collect(
            &self.element_data.children,
            &cloned.borrow().element_data.children,
            &mut map,
        );
        let remap = |child: &DynElement| map[&child.borrow().element_data().internal_id].clone();
        {
            let mut calendar = cloned.borrow_mut();
            calendar.nav = remap(&self.nav);
            calendar.day_header = remap(&self.day_header);
            calendar.week_grid = remap(&self.week_grid);
            calendar.year_dropdown = remap(&self.year_dropdown);
            calendar.month_dropdown = remap(&self.month_dropdown);
            calendar.days = self.days.iter().map(remap).collect();
            calendar.day_cells = self.day_cells.iter().map(remap).collect();
        }
        DynElement::new(cloned)
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
        &mut self,
        render_list: &mut RenderList,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, render_list, resource_manager, text_context, scale_factor);
    }

    fn on_event(&mut self, event: &mut EventKind, _text_context: &mut TextContext) {
        let year_id = self.year_dropdown.with(|element| element.element_data().internal_id);
        let month_id = self.month_dropdown.with(|element| element.element_data().internal_id);
        if let EventKind::Click(click) = event {
            let target = click.target();
            if let Some(index) = self.day_cells.iter().zip(&self.days).position(|(cell, text)| {
                let target_id = target.borrow().element_data().internal_id;
                cell.with(|element| element.element_data().internal_id) == target_id
                    || text.with(|element| element.element_data().internal_id) == target_id
            }) {
                let date = self.day_dates[index];
                if (self.start_year..=self.end_year).contains(&(date.year() as i32)) {
                    self.select_date(Some(date));
                    if let Some(index) = self.selected_index {
                        self.day_cells[index].clone().focus();
                    }
                    queue_event(EventKind::CalendarDateSelected(CalendarDateSelectedEvent::new(
                        DynElement::new(self.element_data.me.upgrade().unwrap()),
                        date,
                    )));
                } else {
                    self.day_cells[index].clone().unfocus();
                }
            }
        }
        if let EventKind::DropdownItemSelected(dropdown_event) = event {
            let target_id = dropdown_event.target().borrow().element_data().internal_id;
            if target_id == year_id {
                self.select_year(dropdown_event.index);
            } else if target_id == month_id {
                self.select_month(dropdown_event.index);
            }
        }
    }

    fn push(&mut self, child: DynElement) {
        push_child_to_element(self, child.inner);
    }
}

impl Calendar {
    pub fn new() -> Self {
        let locale = get_locale_or_default();
        let first_day = first_day_of_week(&locale);
        let start_of_month = current_month();
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<CalendarInner>>| {
            RefCell::new(CalendarInner {
                element_data: ElementData::new(me.clone(), true),
                week_grid: Container::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .with(|el| el.to_dyn_element()),
                days: Vec::new(),
                day_cells: Vec::new(),
                day_dates: Vec::new(),
                selected_date: None,
                selected_index: None,
                day_labels: Vec::new(),
                today_index: None,
                focus_year: start_of_month.year().extended_year(),
                day_header: Container::new().with(|el| el.to_dyn_element()),
                first_day,
                nav: Container::new().with(|el| el.to_dyn_element()),
                focus_month: start_of_month.month().ordinal,
                year_dropdown: Dropdown::new().width(px(100)).with(|el| el.to_dyn_element()),
                month_dropdown: Dropdown::new().width(px(100)).with(|el| el.to_dyn_element()),
                start_year: MIN_YEAR,
                end_year: start_of_month.year().extended_year() + 2,
                locale,
            })
        });
        let mut inner_mut = inner.borrow_mut();
        inner_mut.setup_years();
        inner_mut.setup_months();
        inner_mut.element_data.create_layout_node(None);

        let mut current_header_day = inner_mut.first_day;
        for _ in 0..COLUMNS {
            let day = day_abbreviation(&inner_mut.locale, current_header_day);
            inner_mut.day_header.clone().push(
                Container::new()
                    .display(Display::Flex)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .push(Text::new(day.as_str()).set_selectable(false))
                    .width(CELL_SIZE)
                    .height(CELL_SIZE),
            );
            current_header_day = Weekday::from_days_since_sunday(current_header_day as isize + 1)
        }
        for _ in 0..ROWS {
            let mut week = Container::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row);
            for _ in 0..COLUMNS {
                let day_text = Text::new("").set_selectable(false);
                let day = Button::new()
                    .display(Display::Flex)
                    .border_radius_all((4.0, 4.0))
                    .border_width_all(px(2))
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .width(CELL_SIZE)
                    .height(CELL_SIZE)
                    .push(day_text.clone());
                week = week.push(day.clone());
                inner_mut.days.push(day_text.with(|el| el.to_dyn_element()));
                inner_mut.day_cells.push(day.with(|el| el.to_dyn_element()));
            }
            inner_mut.week_grid.clone().push(week);
        }
        inner_mut.update_calendar();

        inner_mut.set_display(Display::Flex);
        inner_mut.set_flex_direction(FlexDirection::Column);

        let nav = inner_mut.nav.clone();
        nav.clone()
            .display(Display::Flex)
            .justify_content(JustifyContent::SpaceAround)
            .align_items(AlignItems::Center)
            .flex_direction(FlexDirection::Row)
            .width(px(CELL_SIZE.raw_value() * 7.0))
            .push(inner_mut.year_dropdown.clone())
            .push(inner_mut.month_dropdown.clone());
        inner_mut.push(DynElement::new(nav.inner));

        let day_header = inner_mut.day_header.clone();
        inner_mut.push(DynElement::new(day_header.inner));

        let week_grid = inner_mut.week_grid.clone();
        inner_mut.push(DynElement::new(week_grid.inner));

        drop(inner_mut);
        Self { inner }
    }

    pub fn selected_date(&self) -> Option<Date> {
        self.inner.borrow().selected_date
    }

    pub fn visible_dates(&self) -> Vec<Date> {
        self.inner.borrow().day_dates.clone()
    }

    pub fn set_selected_date(&self, date: Option<Date>) {
        self.inner.borrow_mut().select_date(date);
    }

    pub(crate) fn on_date_selected(self, handler: impl Fn(&mut CalendarDateSelectedEvent) + 'static) -> Self {
        self.add_event_listener(
            EventCallbackKind::CalendarDateSelected(Rc::new(handler)),
            EventListenerOptions::default(),
        )
    }

    pub fn set_start_year(&self, year: i32) {
        assert!((MIN_YEAR..=MAX_YEAR).contains(&year), "Unsupported year");
        self.inner.borrow_mut().set_start_year(year);
    }

    pub fn set_end_year(&self, year: i32) {
        assert!((MIN_YEAR..=MAX_YEAR).contains(&year), "Unsupported year");
        self.inner.borrow_mut().set_end_year(year);
    }

}

impl CalendarInner {
    fn select_dropdown(dropdown: &DynElement, index: usize) {
        dropdown.with_mut(|el| {
            (el as &mut dyn std::any::Any)
                .downcast_mut::<DropdownInner>()
                .unwrap()
                .set_selected_element(index);
        });
    }

    fn select_date(&mut self, date: Option<Date>) {
        if date.is_some_and(|date| !(self.start_year..=self.end_year).contains(&(date.year() as i32))) {
            return;
        }
        let month_changed =
            date.is_some_and(|date| date.year() as i32 != self.focus_year || date.month() as u8 != self.focus_month);
        if self.selected_date == date && !month_changed {
            return;
        }
        if !month_changed
            && let Some(previous) = self.selected_index.take()
        {
            self.set_day_selected(previous, false);
        }
        self.selected_date = date;
        if month_changed {
            let date = date.unwrap();
            self.focus_year = date.year() as i32;
            self.focus_month = date.month() as u8;
            Self::select_dropdown(&self.year_dropdown, (self.focus_year - self.start_year) as usize);
            Self::select_dropdown(&self.month_dropdown, (self.focus_month - 1) as usize);
            self.update_calendar();
        } else if let Some(index) = date.and_then(|date| self.day_dates.iter().position(|shown| *shown == date)) {
            self.selected_index = Some(index);
            self.set_day_selected(index, true);
        }
    }

    fn update_calendar(&mut self) {
        self.update_calendar_for_today(jiff::ZonedDateTime::now().date());
    }

    fn update_calendar_for_today(&mut self, today: Date) {
        let mut start_date = current_calendar_start(self.first_day, self.focus_year, Month::new(self.focus_month));
        self.day_dates.clear();
        self.day_labels.clear();
        self.selected_index = None;
        self.today_index = None;
        for (day_element, cell) in self.days.iter().zip(&self.day_cells) {
            let date = Date::new(
                start_date.year().extended_year() as i16,
                start_date.month().ordinal as i8,
                start_date.day_of_month().0 as i8,
            )
            .unwrap();
            self.day_dates.push(date);
            let selected = self.selected_date == Some(date);
            if selected {
                self.selected_index = Some(self.day_dates.len() - 1);
            }
            let is_in_current_month = start_date.month().ordinal == self.focus_month;
            let date_str = format_date_day_number(&self.locale, &start_date);
            day_element.with_mut(|el| {
                (el as &mut dyn std::any::Any)
                    .downcast_mut::<TextInner>()
                    .unwrap()
                    .set_text(&date_str);
            });
            day_element.clone().color(if selected {
                rgb(255, 255, 255)
            } else if !(self.start_year..=self.end_year).contains(&(date.year() as i32)) {
                rgb(170, 170, 170)
            } else if is_in_current_month {
                rgb(0, 0, 0)
            } else {
                rgb(120, 120, 120)
            });
            let mut label = format!(
                "{}-{:02}-{:02}",
                start_date.year().extended_year(),
                start_date.month().ordinal,
                start_date.day_of_month().0
            );
            if date == today {
                self.today_index = Some(self.day_dates.len() - 1);
                label.push_str(", today");
            }
            self.day_labels.push(label.clone());
            if selected {
                label.push_str(", selected");
            }
            cell.clone()
                .border_color_all(if date == today {
                    if selected { rgb(100, 230, 210) } else { rgb(0, 130, 110) }
                } else {
                    crate::Color::TRANSPARENT
                })
                .background_color(if selected {
                    rgb(38, 67, 101)
                } else {
                    crate::Color::TRANSPARENT
                });
            cell.with_mut(|element| {
                element.element_data_mut().set_accessibility_name(label);
                element.element_data_mut().set_accessibility_enabled(
                    (self.start_year..=self.end_year).contains(&start_date.year().extended_year()),
                );
            });
            start_date
                .try_add_with_options(DateDuration::for_days(1), DateAddOptions::default())
                .unwrap()
        }
    }

    fn set_day_selected(&self, index: usize, selected: bool) {
        self.days[index].clone().color(if selected {
            rgb(255, 255, 255)
        } else if self.day_dates[index].month() as u8 == self.focus_month {
            rgb(0, 0, 0)
        } else {
            rgb(120, 120, 120)
        });
        self.day_cells[index]
            .clone()
            .border_color_all(if self.today_index == Some(index) {
                if selected { rgb(100, 230, 210) } else { rgb(0, 130, 110) }
            } else {
                crate::Color::TRANSPARENT
            })
            .background_color(if selected {
                rgb(38, 67, 101)
            } else {
                crate::Color::TRANSPARENT
            });
        let mut label = self.day_labels[index].clone();
        if selected {
            label.push_str(", selected");
        }
        self.day_cells[index].with_mut(|element| {
            element.element_data_mut().set_accessibility_name(label);
        });
    }

    fn select_year(&mut self, year: usize) {
        let year = self.start_year + year as i32;
        if self.focus_year == year {
            return;
        }
        self.focus_year = year;
        self.update_calendar();
    }

    fn select_month(&mut self, month: usize) {
        let month = 1 + month as u8;
        if self.focus_month == month {
            return;
        }
        self.focus_month = month;
        self.update_calendar();
    }

    fn setup_years(&mut self) {
        let dropdown = self.year_dropdown.clone();
        dropdown.remove_all_children();
        for year in self.start_year..=self.end_year {
            dropdown
                .clone()
                .push(Text::new(&year_name(&self.locale, year)).set_selectable(false))
                .font_size(20.0);
            if year == self.focus_year {
                Self::select_dropdown(&dropdown, (year - self.start_year) as usize);
            }
        }
    }

    fn setup_months(&mut self) {
        let dropdown = self.month_dropdown.clone();
        dropdown.remove_all_children();
        for month in 0..12 {
            dropdown
                .clone()
                .push(Text::new(&month_name(&self.locale, Month::new(month + 1), self.focus_year)).set_selectable(false))
                .font_size(20.0);
            if month + 1 == self.focus_month {
                Self::select_dropdown(&dropdown, month as usize);
            }
        }
    }

    pub fn set_start_year(&mut self, year: i32) {
        if self.start_year == year {
            return;
        }
        if year > self.end_year {
            panic!("Invalid start year");
        }
        self.start_year = year;
        self.focus_year = self.focus_year.max(year);
        if self.selected_date.is_some_and(|date| (date.year() as i32) < year) {
            self.selected_date = None;
        }
        self.setup_years();
        self.update_calendar();
    }

    pub fn set_end_year(&mut self, year: i32) {
        if self.end_year == year {
            return;
        }
        if year < self.start_year {
            panic!("Invalid end year");
        }
        self.end_year = year;
        self.focus_year = self.focus_year.min(year);
        if self.selected_date.is_some_and(|date| (date.year() as i32) > year) {
            self.selected_date = None;
        }
        self.setup_years();
        self.update_calendar();
    }
}

impl Calendar {
    pub fn add_date_selected_listener(&self, handler: impl Fn(&mut CalendarDateSelectedEvent) + 'static) {
        self.clone().on_date_selected(handler);
    }
}

#[cfg(test)]
mod retained_tests {
    use crate::events::{ClickEvent, ClickTrigger};

    use super::*;

    #[test]
    fn selecting_day_does_not_rewrite_other_cells() {
        let calendar = Calendar::new();
        let cells = calendar.inner.borrow().day_cells.clone();
        let dates = calendar.inner.borrow().day_dates.clone();
        let _unrelated_borrow = cells[20].inner.borrow();
        let mut context = TextContext::default();
        for index in [10, 11, 11] {
            let mut event = EventKind::Click(ClickEvent::new(
                DynElement::new(cells[index].inner.clone()),
                ClickTrigger::Accessibility,
            ));
            calendar.inner.borrow_mut().on_event(&mut event, &mut context);
            assert_eq!(calendar.inner.borrow().selected_date, Some(dates[index]));
        }
        assert_eq!(calendar.inner.borrow().day_dates, dates);
    }
}
