use crate::prelude::*;
use bevy::prelude::*;

/// A plain calendar date -- deliberately self-contained (no wall-clock "today", no timezone,
/// no new date/time crate dependency) since this widget only needs to *display* and *compare*
/// dates a caller hands it, never read the system clock. See [`DatePicker`]'s doc comment for
/// what that means for callers wanting a "today" default.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct CalendarDate {
    /// The calendar year (astronomical numbering -- there is no year 0, but this type doesn't
    /// enforce that; it's a plain display/comparison value).
    pub year: i32,
    /// The month, `1..=12`.
    pub month: u8,
    /// The day of month, `1..=31` (validity against `month`/`year` isn't enforced by this
    /// type -- callers constructing one directly are expected to pass a real date).
    pub day: u8,
}

impl CalendarDate {
    /// A new date. Doesn't validate `day` against `days_in_month(year, month)`.
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// The standard Gregorian leap-year rule: divisible by 4, except centuries, except again
    /// every 400 years.
    pub fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    /// How many days are in `month` (`1..=12`) of `year`. An out-of-range `month` returns `30`
    /// as a defensive fallback rather than panicking.
    pub fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// The day of the week for `year`-`month`-`day`, via Sakamoto's algorithm -- `0` = Sunday
    /// .. `6` = Saturday. Valid across the whole proleptic Gregorian calendar; no library
    /// dependency needed for it.
    pub fn day_of_week(year: i32, month: u8, day: u8) -> u8 {
        const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let month = month.clamp(1, 12);
        let y = if month < 3 { year - 1 } else { year };
        let result = y + y / 4 - y / 100 + y / 400 + T[(month - 1) as usize] + day as i32;
        result.rem_euclid(7) as u8
    }

    /// This date's own day-of-week.
    pub fn weekday(self) -> u8 {
        Self::day_of_week(self.year, self.month, self.day)
    }

    /// How many days are in this date's own month.
    pub fn days_in_this_month(self) -> u8 {
        Self::days_in_month(self.year, self.month)
    }

    /// The same day-of-month one month later, clamped to the target month's own length (e.g.
    /// Jan 31 -> Feb 28/29, not Mar 2/3).
    pub fn next_month(self) -> Self {
        let (year, month) = if self.month == 12 {
            (self.year + 1, 1)
        } else {
            (self.year, self.month + 1)
        };
        Self {
            year,
            month,
            day: self.day.min(Self::days_in_month(year, month)),
        }
    }

    /// The same day-of-month one month earlier, with the same end-of-month clamping as
    /// [`Self::next_month`].
    pub fn prev_month(self) -> Self {
        let (year, month) = if self.month == 1 {
            (self.year - 1, 12)
        } else {
            (self.year, self.month - 1)
        };
        Self {
            year,
            month,
            day: self.day.min(Self::days_in_month(year, month)),
        }
    }

    /// This date with a different day of month.
    pub fn with_day(self, day: u8) -> Self {
        Self { day, ..self }
    }
}

impl std::fmt::Display for CalendarDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAY_LABELS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

/// Fired when a date is chosen -- a day-cell click, or Enter with a day keyboard-active.
#[derive(Debug, Clone, Reflect)]
pub struct DateChanged {
    /// The chosen date.
    pub date: CalendarDate,
}

/// A collection of styles for [`DatePicker`].
#[derive(Component, Clone, PartialEq, Reflect)]
pub struct DatePickerStyles {
    /// The trigger row's own background/border (`DatePicker`'s own `WoodpeckerStyle`).
    pub background: WoodpeckerStyle,
    /// The trigger's date text (or placeholder) styles.
    pub text: WoodpeckerStyle,
    /// The calendar icon's styles.
    pub icon: WoodpeckerStyle,
    /// The "Month YYYY" header label styles.
    pub header_label: WoodpeckerStyle,
    /// The prev/next month nav buttons' styles.
    pub nav_button: ButtonStyles,
    /// A weekday initial's styles (`Su`, `Mo`, ...).
    pub weekday_label: WoodpeckerStyle,
    /// A day cell's styles.
    pub day_cell: ButtonStyles,
    /// Background applied to the selected day, over `day_cell`'s own.
    pub selected_day_background: Color,
    /// Text color for a day outside `DatePicker::min_date..=DatePicker::max_date`.
    pub disabled_day_color: Color,
    /// Divider color for the grid lines between day cells.
    pub grid_line_color: Color,
}

impl Default for DatePickerStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for DatePickerStyles {
    fn from_theme(theme: &Theme) -> Self {
        let nav_normal = WoodpeckerStyle {
            width: 32.0.into(),
            height: 32.0.into(),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            background_color: Color::NONE,
            color: theme.text,
            font_size: theme.font_size * 1.4,
            border_radius: Corner::all(theme.control_radius),
            ..Default::default()
        };
        let grid_line_color = theme.border.with_alpha(0.5);
        // Only the right/bottom edges -- each internal divider is then drawn once (by the
        // cell to its left/above), not doubled-up between two adjacent cells. Square corners
        // (no `border_radius`) read as a continuous grid rather than a row of separate pills.
        let day_normal = WoodpeckerStyle {
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            background_color: Color::NONE,
            color: theme.text,
            font_size: theme.font_size,
            border: Edge::all(0.0).right(1.0).bottom(1.0),
            border_color: grid_line_color,
            ..Default::default()
        };
        Self {
            background: WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Row,
                align_items: Some(WidgetAlignItems::Center),
                background_color: theme.background,
                border: Edge::all(1.0),
                border_color: theme.border,
                border_radius: Corner::all(theme.control_radius),
                width: Units::Percentage(100.0),
                height: theme.control_height.into(),
                padding: Edge::all(0.0).left(12.0).right(8.0),
                ..Default::default()
            },
            text: WoodpeckerStyle {
                color: theme.text,
                font_size: theme.font_size,
                flex_grow: 1.0,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            icon: WoodpeckerStyle {
                color: theme.text,
                width: 16.0.into(),
                height: 16.0.into(),
                ..Default::default()
            },
            header_label: WoodpeckerStyle {
                color: theme.text,
                font_size: theme.font_size,
                // `width: 100%` of its own wrapper (see `render`'s "label_wrapper"), not
                // `flex_grow` on this entity directly -- the renderer computes `text_alignment`
                // against this entity's *parent's* resolved width, not its own, so the parent
                // must already be exactly as wide as this element for centering to land in the
                // right place (see `NumberInputStyles::label`'s doc comment for the same
                // gotcha, hit there first).
                width: Units::Percentage(100.0),
                text_alignment: Some(TextAlign::Center),
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            nav_button: ButtonStyles {
                normal: nav_normal,
                hovered: WoodpeckerStyle {
                    background_color: theme.background_light,
                    ..nav_normal
                },
            },
            weekday_label: WoodpeckerStyle {
                color: theme.text.with_alpha(0.6),
                font_size: theme.font_size * 0.8,
                justify_content: Some(WidgetAlignContent::Center),
                ..Default::default()
            },
            day_cell: ButtonStyles {
                normal: day_normal,
                hovered: WoodpeckerStyle {
                    background_color: theme.background_light,
                    ..day_normal
                },
            },
            selected_day_background: theme.primary.with_alpha(0.5),
            disabled_day_color: theme.text.with_alpha(0.3),
            grid_line_color,
        }
    }
}

/// Self-managed state -- mirrors [`crate::widgets::dropdown::DropdownState`]'s "initial prop,
/// then state takes over" shape. `view` is which month is currently displayed (independent of
/// `selected`, so browsing to a different month doesn't select a date until a day is clicked).
#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
struct DatePickerState {
    is_open: bool,
    selected: Option<CalendarDate>,
    view: CalendarDate,
    active_day: u8,
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle::default()
}

/// A text-trigger-plus-calendar-popover date picker, wrapping [`Popover`] for the
/// trigger/floating-panel mechanics and a [`GridTemplate`] 7-column grid for the day cells.
/// Fires `Change<DateChanged>` when a day is chosen (click, or Enter with a day
/// keyboard-active). Keyboard: Left/Right/Up/Down move a roving cursor within the displayed
/// month (deliberately not crossing a month boundary -- see the scope note below),
/// Enter selects, Escape closes.
///
/// No built-in "today": this widget never reads the system clock (cross-platform civil-
/// calendar correctness, including on WASM, is real extra work it doesn't need to own). A
/// caller wanting "today" highlighted computes it themselves and passes it as
/// `initial_view`/`selected_date`.
///
/// Scope note: arrow-key navigation clamps at the first/last day of the *currently displayed*
/// month rather than rolling over into the adjacent month -- crossing a month boundary
/// requires the same nav-button click (or Left/Right at the boundary, a fast-follow) either
/// way, so this is a minor precision trim, not a missing capability.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = default_style(),
    WidgetRender = WidgetRender::Quad,
    WidgetChildren,
    Pickable,
    Focusable,
    DatePickerStyles
)]
pub struct DatePicker {
    /// The currently selected date, if any.
    pub selected_date: Option<CalendarDate>,
    /// The earliest selectable date, if any.
    pub min_date: Option<CalendarDate>,
    /// The latest selectable date, if any.
    pub max_date: Option<CalendarDate>,
    /// Which month the calendar opens showing, when `selected_date` is `None`.
    pub initial_view: CalendarDate,
    /// Shown in the trigger when nothing is selected yet.
    pub placeholder: String,
}

impl Default for DatePicker {
    fn default() -> Self {
        Self {
            selected_date: None,
            min_date: None,
            max_date: None,
            initial_view: CalendarDate::new(1970, 1, 1),
            placeholder: "Select a date".into(),
        }
    }
}

fn in_range(date: CalendarDate, min: Option<CalendarDate>, max: Option<CalendarDate>) -> bool {
    min.is_none_or(|min| date >= min) && max.is_none_or(|max| date <= max)
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    icon_font: Res<IconFont>,
    theme: Res<Theme>,
    mut query: Query<(
        &DatePicker,
        &DatePickerStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    state_query: Query<&DatePickerState>,
) {
    let Ok((date_picker, styles, mut woodpecker_style, mut children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let view = date_picker
        .selected_date
        .unwrap_or(date_picker.initial_view);
    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        DatePickerState {
            is_open: false,
            selected: date_picker.selected_date,
            view,
            active_day: date_picker.selected_date.unwrap_or(view).day,
        },
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    let min_date = date_picker.min_date;
    let max_date = date_picker.max_date;
    let current_widget_val = *current_widget;
    let combo_entity = current_widget_val.0;

    *woodpecker_style = styles.background;

    *children = WidgetChildren::default();
    // Self-observers (queue still empty) -- attached to `DatePicker`'s own root, which now
    // carries `Pickable`/`Focusable` so the *whole* trigger pill is one click/focus target
    // (the `text` `Element` below has no `Pickable` of its own; a click on it hits this root
    // instead, exactly like `Dropdown`'s own trigger row).
    children.observe(
        current_widget_val,
        move |_: On<Pointer<Click>>, mut state_query: Query<&mut DatePickerState>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.is_open = true;
        },
    );
    children.observe(
        current_widget_val,
        move |trigger: On<WidgetKeyboardButtonEvent>,
              mut commands: Commands,
              date_picker_query: Query<&DatePicker>,
              mut state_query: Query<&mut DatePickerState>| {
            let Ok(date_picker) = date_picker_query.get(combo_entity) else {
                return;
            };
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            if !state.is_open {
                return;
            }
            let days_in_month = state.view.days_in_this_month();
            match trigger.code {
                KeyCode::ArrowRight => {
                    state.active_day = (state.active_day + 1).min(days_in_month);
                }
                KeyCode::ArrowLeft => {
                    state.active_day = state.active_day.saturating_sub(1).max(1);
                }
                KeyCode::ArrowDown => {
                    state.active_day = (state.active_day + 7).min(days_in_month);
                }
                KeyCode::ArrowUp => {
                    state.active_day = state.active_day.saturating_sub(7).max(1);
                }
                KeyCode::Enter => {
                    let date = state.view.with_day(state.active_day);
                    if !in_range(date, date_picker.min_date, date_picker.max_date) {
                        return;
                    }
                    state.selected = Some(date);
                    state.is_open = false;
                    commands.trigger(Change {
                        target: combo_entity,
                        data: DateChanged { date },
                    });
                }
                KeyCode::Escape => {
                    state.is_open = false;
                }
                _ => {}
            }
        },
    );

    // A full-screen click-catcher, only present while open, added *before* the `Popover`
    // below so the popover's own floating content (added later, deeper in the tree, but
    // still later in this same depth-first `order` walk) wins picking priority over it --
    // matches `Dropdown`'s overlay for the same reason.
    if state.is_open {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Fixed,
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                ..Default::default()
            },
            Pickable::default(),
        ));
        children
            .observe(current_widget_val, |mut trigger: On<Pointer<Over>>| {
                trigger.propagate(false);
            })
            .observe(current_widget_val, |mut trigger: On<Pointer<Out>>| {
                trigger.propagate(false);
            })
            .observe(
                current_widget_val,
                move |mut trigger: On<Pointer<Click>>,
                      mut state_query: Query<&mut DatePickerState>| {
                    trigger.propagate(false);
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };
                    state.is_open = false;
                },
            );
        children.add_key("overlay");
    }

    // Header: prev/next nav + "Month YYYY" label.
    let mut header_children = WidgetChildren::default();
    header_children
        .add::<WButton>((
            WButton,
            styles.nav_button,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: styles.nav_button.normal.color,
                    font_size: styles.nav_button.normal.font_size,
                    font: Some(icon_font.0.id()),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: icons::CARET_LEFT.into(),
                },
            )),
        ))
        .observe(
            current_widget_val,
            move |_: On<Pointer<Click>>, mut state_query: Query<&mut DatePickerState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.view = state.view.prev_month();
                state.active_day = state.active_day.min(state.view.days_in_this_month());
            },
        )
        .hover_cursor(current_widget_val, SystemCursorIcon::Pointer);
    header_children.add_key("prev");
    header_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Element>((
            Element,
            styles.header_label,
            WidgetRender::Text {
                content: format!(
                    "{} {}",
                    MONTH_NAMES[(state.view.month - 1) as usize],
                    state.view.year
                ),
            },
        )),
    ));
    header_children.add_key("label_wrapper");
    header_children
        .add::<WButton>((
            WButton,
            styles.nav_button,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: styles.nav_button.normal.color,
                    font_size: styles.nav_button.normal.font_size,
                    font: Some(icon_font.0.id()),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: icons::CARET_RIGHT.into(),
                },
            )),
        ))
        .observe(
            current_widget_val,
            move |_: On<Pointer<Click>>, mut state_query: Query<&mut DatePickerState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.view = state.view.next_month();
                state.active_day = state.active_day.min(state.view.days_in_this_month());
            },
        )
        .hover_cursor(current_widget_val, SystemCursorIcon::Pointer);
    header_children.add_key("next");

    let mut panel_children = WidgetChildren::default();
    panel_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Row,
            align_items: Some(WidgetAlignItems::Center),
            margin: Edge::all(0.0).bottom(8.0),
            ..Default::default()
        },
        header_children,
        WidgetRender::Quad,
    ));
    panel_children.add_key("header");

    // Weekday initials row.
    let mut weekday_children = WidgetChildren::default();
    for label in WEEKDAY_LABELS {
        weekday_children.add::<Element>((
            Element,
            styles.weekday_label,
            WidgetRender::Text {
                content: label.into(),
            },
        ));
        weekday_children.add_key(label);
    }
    panel_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            display: WidgetDisplay::Grid,
            width: Units::Percentage(100.0),
            ..Default::default()
        },
        GridTemplate {
            columns: vec![GridTrackSize::Fraction(1.0); 7],
            ..Default::default()
        },
        weekday_children,
    ));
    panel_children.add_key("weekdays");

    // Day-cell grid, with leading blank cells for the 1st's weekday offset.
    let leading_blanks = CalendarDate::day_of_week(state.view.year, state.view.month, 1);
    let days_in_month = state.view.days_in_this_month();
    let mut day_children = WidgetChildren::default();
    for blank in 0..leading_blanks {
        day_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                border: Edge::all(0.0).right(1.0).bottom(1.0),
                border_color: styles.grid_line_color,
                ..Default::default()
            },
        ));
        day_children.add_key(format!("blank{blank}"));
    }
    for day in 1..=days_in_month {
        let date = state.view.with_day(day);
        let enabled = in_range(date, min_date, max_date);
        let is_selected = state.selected == Some(date);
        let is_active = state.active_day == day;
        let mut cell_style = if is_selected {
            WoodpeckerStyle {
                background_color: styles.selected_day_background,
                ..styles.day_cell.normal
            }
        } else {
            styles.day_cell.normal
        };
        if !enabled {
            cell_style.color = styles.disabled_day_color;
        }
        if is_active && !is_selected {
            cell_style.border = Edge::all(1.0);
            cell_style.border_color = styles.selected_day_background;
        }

        day_children.add::<WButton>((
            WButton,
            ButtonStyles {
                normal: cell_style,
                hovered: if enabled {
                    styles.day_cell.hovered
                } else {
                    cell_style
                },
            },
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: cell_style.color,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("{day}"),
                },
            )),
        ));
        if enabled {
            day_children
                .observe(
                    current_widget_val,
                    move |_: On<Pointer<Click>>,
                          mut commands: Commands,
                          mut state_query: Query<&mut DatePickerState>| {
                        let Ok(mut state) = state_query.get_mut(state_entity) else {
                            return;
                        };
                        state.selected = Some(date);
                        state.active_day = date.day;
                        state.is_open = false;
                        commands.trigger(Change {
                            target: combo_entity,
                            data: DateChanged { date },
                        });
                    },
                )
                .hover_cursor(current_widget_val, SystemCursorIcon::Pointer);
        }
        day_children.add_key(format!("day{day}"));
    }
    panel_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            display: WidgetDisplay::Grid,
            width: Units::Percentage(100.0),
            ..Default::default()
        },
        GridTemplate {
            columns: vec![GridTrackSize::Fraction(1.0); 7],
            ..Default::default()
        },
        day_children,
    ));
    panel_children.add_key("days");

    // `Popover`'s own floating-content wrapper defaults to `flex_direction: Row` (it exposes
    // no field to override that), so `header`/`weekdays`/`days` need their own explicit
    // `Column` wrapper here rather than being handed to `PopoverContent` directly -- otherwise
    // they'd lay out side by side instead of stacked. `Popover`'s own `PopoverStyles` (left at
    // its themed default below) draws the visible card -- background/border/radius/padding --
    // around this wrapper, matching the trigger pill's own colors and corner radius since both
    // come from the same `Theme`.
    let mut panel = WidgetChildren::default();
    panel.add::<Element>((
        Element,
        // A fixed width -- `Popover`'s own floating-content wrapper (which contains this one)
        // has no explicit width of its own, so it auto-sizes to fit its child; this needs to
        // be the thing establishing a real size, not `Percentage(100.0)` of an auto-sized
        // parent (which would just collapse).
        WoodpeckerStyle {
            width: 240.0.into(),
            flex_direction: WidgetFlexDirection::Column,
            ..Default::default()
        },
        panel_children,
    ));
    panel.add_key("panel");

    // Added as the first *normal-flow* child (the overlay above is `position: Fixed`, pulled
    // out of flow entirely, so it doesn't affect this) -- anchors the popover's own box, and
    // therefore its absolutely-positioned floating content, at the trigger row's left edge.
    // `internal_styles.height` matches the trigger pill's own height (`styles.background`'s)
    // so the panel's `top: 100%` placement (relative to *this* box, not the pill itself, which
    // is a sibling on `DatePicker`'s own root, not an ancestor) drops the panel flush below the
    // pill instead of overlapping it -- `Popover`'s own "trigger" wrapper is otherwise empty
    // (we render the real trigger content, `text`/`icon`, as `DatePicker`'s own children, not
    // `Popover`'s), so without this its box would auto-size to zero height.
    children.add::<Popover>((PopoverBundle {
        popover: Popover {
            visible: state.is_open,
            placement: PopoverPlacement::Bottom,
        },
        content: PopoverContent(panel),
        // Not `PopoverStyles::default()` -- that's always `Theme::default()` (dark) baked in,
        // independent of the live theme this render already has access to, and independent of
        // `DatePickerStyles` (a *separate* component covering only the trigger/text/day-cell
        // colors, not this composed `Popover`'s own panel background/border). Deriving it from
        // the live `theme` here keeps the calendar panel's background in sync with everything
        // else instead of staying frozen dark regardless of theme.
        styles: PopoverStyles::from_theme(&theme),
        internal_styles: WoodpeckerStyle {
            height: styles.background.height,
            // `Popover` (this widget) is a normal-flow child of `DatePicker`'s own row, which
            // has 12px of left padding for the trigger's text -- flex children are positioned
            // *inside* that padding, so without this the popover's anchor (and therefore the
            // panel's `left: 0` placement, which is relative to *this* box) would land 12px
            // right of the trigger pill's own outer left edge. Must match
            // `DatePickerStyles::background`'s own `padding.left`.
            margin: Edge::all(0.0).left(-12.0),
            ..Default::default()
        },
        ..Default::default()
    },));
    children.add_key("popover");

    let trigger_text = state
        .selected
        .map(|d| d.to_string())
        .unwrap_or_else(|| date_picker.placeholder.clone());
    children.add::<Element>((
        Element,
        styles.text,
        WidgetRender::Text {
            content: trigger_text,
        },
    ));
    children.add_key("text");

    children
        .add::<Element>((
            Element,
            WoodpeckerStyle {
                font: Some(icon_font.0.id()),
                ..styles.icon
            },
            WidgetRender::Text {
                content: icons::CALENDAR.into(),
            },
            Pickable::default(),
        ))
        .observe(
            current_widget_val,
            move |mut trigger: On<Pointer<Click>>, mut state_query: Query<&mut DatePickerState>| {
                trigger.propagate(false);
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.is_open = !state.is_open;
            },
        );
    children.add_key("icon");

    children.apply(current_widget_val.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leap_years_follow_the_gregorian_rule() {
        assert!(CalendarDate::is_leap_year(2000));
        assert!(!CalendarDate::is_leap_year(1900));
        assert!(CalendarDate::is_leap_year(2024));
        assert!(!CalendarDate::is_leap_year(2023));
    }

    #[test]
    fn days_in_month_accounts_for_leap_february() {
        assert_eq!(CalendarDate::days_in_month(2024, 2), 29);
        assert_eq!(CalendarDate::days_in_month(2023, 2), 28);
        assert_eq!(CalendarDate::days_in_month(2024, 4), 30);
        assert_eq!(CalendarDate::days_in_month(2024, 1), 31);
    }

    #[test]
    fn day_of_week_matches_known_reference_dates() {
        // Jan 1, 2000 was a Saturday; Jan 1, 2024 was a Monday.
        assert_eq!(CalendarDate::day_of_week(2000, 1, 1), 6);
        assert_eq!(CalendarDate::day_of_week(2024, 1, 1), 1);
    }

    #[test]
    fn next_month_clamps_day_to_the_target_months_length() {
        let jan31 = CalendarDate::new(2024, 1, 31);
        assert_eq!(jan31.next_month(), CalendarDate::new(2024, 2, 29));

        let dec31 = CalendarDate::new(2023, 12, 31);
        assert_eq!(dec31.next_month(), CalendarDate::new(2024, 1, 31));
    }

    #[test]
    fn prev_month_clamps_day_and_wraps_the_year() {
        let mar31 = CalendarDate::new(2024, 3, 31);
        assert_eq!(mar31.prev_month(), CalendarDate::new(2024, 2, 29));

        let jan1 = CalendarDate::new(2024, 1, 1);
        assert_eq!(jan1.prev_month(), CalendarDate::new(2023, 12, 1));
    }

    #[test]
    fn ordering_is_chronological() {
        let earlier = CalendarDate::new(2024, 1, 31);
        let later = CalendarDate::new(2024, 2, 1);
        assert!(earlier < later);
    }

    #[test]
    fn in_range_respects_optional_bounds() {
        let date = CalendarDate::new(2024, 6, 15);
        assert!(in_range(date, None, None));
        assert!(in_range(date, Some(CalendarDate::new(2024, 1, 1)), None));
        assert!(!in_range(date, Some(CalendarDate::new(2024, 7, 1)), None));
        assert!(in_range(date, None, Some(CalendarDate::new(2024, 12, 31))));
        assert!(!in_range(date, None, Some(CalendarDate::new(2024, 1, 1))));
    }
}
