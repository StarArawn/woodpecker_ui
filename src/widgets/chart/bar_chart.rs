use bevy::platform::collections::HashMap;
use bevy::platform::time::Instant;
use bevy::prelude::*;

use crate::prelude::*;

use super::chart_core::{stacked_baselines, ChartLegend, ChartSeriesData, LegendEntry};

/// How a [`BarChart`] with more than one series lays its bars out per category.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BarMode {
    /// Each series gets its own bar, side by side within the category.
    #[default]
    Grouped,
    /// Series stack on top of each other within one bar per category.
    Stacked,
}

/// One named series of values on a [`BarChart`], one value per `BarChart::categories` entry.
#[derive(Reflect, Clone, PartialEq, Debug, Default)]
pub struct BarSeries {
    /// The series name, shown in the legend.
    pub name: String,
    /// One value per `BarChart::categories` entry.
    pub values: ChartSeriesData,
    /// The color of this series' bars and legend swatch.
    pub color: Color,
}

impl BarSeries {
    /// Convenience constructor for a named series.
    pub fn new(name: impl Into<String>, values: impl Into<ChartSeriesData>, color: Color) -> Self {
        Self {
            name: name.into(),
            values: values.into(),
            color,
        }
    }
}

/// Styles for [`BarChart`].
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct BarChartStyles {
    /// Gap between bars within the same category (`Grouped` mode only).
    pub bar_gap: f32,
    /// Gap between categories.
    pub group_gap: f32,
    /// Corner radius of each bar.
    pub corner_radius: f32,
    /// Text color of category labels below the plot.
    pub category_label_color: Color,
    /// How long (ms) each bar's entrance grow-in animation takes.
    pub entrance_duration_ms: f32,
    /// How long (ms) each successive category's entrance is delayed relative to the last.
    pub entrance_stagger_ms: f32,
}

impl Default for BarChartStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for BarChartStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            bar_gap: 4.0,
            group_gap: 16.0,
            corner_radius: 3.0,
            category_label_color: theme.text.with_alpha(0.75),
            entrance_duration_ms: 350.0,
            entrance_stagger_ms: 40.0,
        }
    }
}

fn default_chart_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: 240.0.into(),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

/// A grouped or stacked bar chart for comparing values across categories.
///
/// Unlike [`super::LineChart`]/[`super::AreaChart`]/[`super::PieChart`], bars are real child
/// widgets laid out via ordinary flexbox (not one shared vello draw call) -- that gets
/// grouped/stacked layout for free from Taffy, and lets each bar own a real
/// [`Transition`] for its own staggered entrance animation.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_chart_style(), WidgetChildren, BarChartStyles)]
pub struct BarChart {
    /// The category labels along the X axis, left to right.
    pub categories: Vec<String>,
    /// The series to plot, one value per category (extra/missing values are treated as `0.0`).
    pub series: Vec<BarSeries>,
    /// Grouped (side-by-side) or stacked bars.
    pub mode: BarMode,
    /// Whether to render a legend row below the plot. Hovering an entry shows that series'
    /// last category's value; hovering an individual bar (always on, regardless of this flag)
    /// shows that exact category's value.
    pub show_legend: bool,
}

impl Default for BarChart {
    fn default() -> Self {
        Self {
            categories: Vec::new(),
            series: Vec::new(),
            mode: BarMode::default(),
            show_legend: true,
        }
    }
}

fn format_value(value: f32) -> String {
    if value.fract().abs() < 0.05 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

/// A single bar segment: a directly-drawn, `Transition`-animated quad (height entrance), wrapped
/// in a [`Tooltip`] showing its exact value on hover.
///
/// Earlier drafts wrapped each bar in a `Tooltip` and dropped the idea: `Tooltip::render`
/// re-spawns its trigger content as a child of a synthetic `WoodpeckerStyle::default()` (i.e.
/// auto-sized, not stretched) wrapper, and a `width: 100%, height: 100%` bar nested inside that
/// resolved against an indefinite size and collapsed to nothing. Revisited once that was fixed
/// at the source (`Tooltip::render` now sizes its trigger wrapper from its own resolved layout
/// instead) -- see that widget's own doc comment. The bar itself is now `Tooltip`'s own visible
/// `WidgetRender::Quad`/`Transition`-animated box (not a separate nested `Element`) -- `Tooltip`
/// requires `WoodpeckerStyle` like any other widget, so it can carry the exact same layout/paint
/// properties an inline `Element` bundle could, with an empty `PassedChildren` purely to get a
/// correctly-sized, `Pickable` hover surface out of `Tooltip::render`'s own auto-built trigger.
///
/// `share_row_width`: `Grouped` mode lays bars out in a `Row` (main axis = horizontal), where
/// every bar claiming `width: 100%` of that row fights its siblings for the same space instead
/// of sharing it -- `flex_grow`-based sharing (standard "equal columns" flexbox) fixes that.
/// `Stacked` mode's bars are the sole occupant of their own `Column` cross axis, where
/// `width: 100%` is exactly right (fills the stack) and no sharing is needed.
///
/// `entrance`: `None` plays a fresh grow-in from zero height, starting at `start`; `Some(_)`
/// (this bar's height is unchanged since the last render that actually built it) instead builds
/// an already-settled, non-`playing` `Transition` pinned at `final_style` -- see `render`'s own
/// comment for why a bar can't just omit `Transition` on a repeat render instead.
#[allow(clippy::too_many_arguments)]
fn bar_segment(
    height_pct: f32,
    color: Color,
    corner_radius: f32,
    share_row_width: bool,
    start: Instant,
    duration_ms: f32,
    already_settled: bool,
    tooltip_text: String,
) -> impl Bundle + Clone {
    let final_style = WoodpeckerStyle {
        width: if share_row_width {
            Units::Auto
        } else {
            Units::Percentage(100.0)
        },
        height: Units::Percentage(height_pct.clamp(0.0, 100.0)),
        background_color: color,
        border_radius: Corner::all(corner_radius),
        flex_grow: if share_row_width { 1.0 } else { 0.0 },
        flex_shrink: if share_row_width { 1.0 } else { 0.0 },
        flex_basis: if share_row_width {
            Units::Pixels(0.0)
        } else {
            Units::Auto
        },
        ..Default::default()
    };
    let start_style = WoodpeckerStyle {
        height: Units::Percentage(0.0),
        ..final_style
    };

    let transition = if already_settled {
        Transition {
            playing: false,
            easing: TransitionEasing::CubicOut,
            reversing: false,
            start,
            timeout: duration_ms,
            looping: false,
            style_a: final_style,
            style_b: final_style,
        }
    } else {
        Transition {
            playing: true,
            easing: TransitionEasing::CubicOut,
            reversing: false,
            start,
            timeout: duration_ms,
            looping: false,
            style_a: start_style,
            style_b: final_style,
        }
    };

    (
        Tooltip {
            text: tooltip_text,
            placement: PopoverPlacement::Top,
        },
        final_style,
        WidgetRender::Quad,
        transition,
        PassedChildren(WidgetChildren::default()),
    )
}

/// Which of a `BarChart`'s bars (keyed `"{cat_index}_{series_index}"`) have already played their
/// entrance grow-in, and at what height percentage -- so a render this widget didn't ask for
/// (e.g. a `ScrollBox` ancestor scrolling: it re-declares its entire `PassedChildren` subtree,
/// forcing every descendant's `WidgetChildren::apply` to reinsert a fresh `Mounted` and thus
/// re-render regardless of prop diffing) doesn't restart every bar's animation from zero. See
/// `bar_segment`'s own comment for why the fix is "keep declaring an (inert) `Transition`", not
/// "stop declaring one" -- `update_transitions` unconditionally overwrites a bar's style with
/// its `Transition`'s own resolved value every frame, so omitting the component on a later
/// render would leave the *previous* render's `Transition` (with a stale target) still in sole
/// control of the bar's height forever, never able to pick up a genuine future data change.
#[derive(Component, Default, Clone)]
struct BarEntranceState {
    last_pct: HashMap<String, f32>,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&BarChart, &BarChartStyles, &mut WidgetChildren)>,
    mut state_query: Query<&mut BarEntranceState>,
) {
    let Ok((chart, styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, BarEntranceState::default());
    let Ok(mut entrance) = state_query.get_mut(state_entity) else {
        return;
    };

    *children = WidgetChildren::default();

    if chart.categories.is_empty() || chart.series.is_empty() {
        children.apply(current_widget.as_parent());
        return;
    }

    let values: Vec<ChartSeriesData> = chart.series.iter().map(|s| s.values.clone()).collect();
    let domain_max = match chart.mode {
        BarMode::Grouped => values
            .iter()
            .flat_map(|v| v.iter().copied())
            .fold(0.0_f32, f32::max),
        BarMode::Stacked => stacked_baselines(&values)
            .iter()
            .flat_map(|b| b.iter().map(|(_, upper)| *upper))
            .fold(0.0_f32, f32::max),
    }
    .max(f32::EPSILON);

    let stacked_bounds = matches!(chart.mode, BarMode::Stacked).then(|| stacked_baselines(&values));

    let mut plot = WidgetChildren::default();
    for (cat_index, category) in chart.categories.iter().enumerate() {
        let start = AnimationGroup::staggered_start(cat_index, styles.entrance_stagger_ms);

        let mut bars_row = WidgetChildren::default();
        match chart.mode {
            BarMode::Grouped => {
                for (series_index, series) in chart.series.iter().enumerate() {
                    let value = series.values.get(cat_index).copied().unwrap_or(0.0);
                    let pct = (value / domain_max) * 100.0;
                    let key = format!("{cat_index}_{series_index}");
                    let already_settled = entrance
                        .last_pct
                        .insert(key, pct)
                        .is_some_and(|last| (last - pct).abs() < f32::EPSILON);
                    bars_row.add::<Tooltip>(bar_segment(
                        pct,
                        series.color,
                        styles.corner_radius,
                        true,
                        start,
                        styles.entrance_duration_ms,
                        already_settled,
                        format!("{}, {category}: {}", series.name, format_value(value)),
                    ));
                    bars_row.add_key(format!("series_{series_index}"));
                }
            }
            BarMode::Stacked => {
                let bounds = stacked_bounds
                    .as_ref()
                    .expect("computed above for Stacked mode");
                for (series_index, series) in chart.series.iter().enumerate().rev() {
                    let (lower, upper) = bounds[series_index][cat_index];
                    let pct = ((upper - lower) / domain_max) * 100.0;
                    let key = format!("{cat_index}_{series_index}");
                    let already_settled = entrance
                        .last_pct
                        .insert(key, pct)
                        .is_some_and(|last| (last - pct).abs() < f32::EPSILON);
                    bars_row.add::<Tooltip>(bar_segment(
                        pct,
                        series.color,
                        styles.corner_radius,
                        false,
                        start,
                        styles.entrance_duration_ms,
                        already_settled,
                        format!("{}, {category}: {}", series.name, format_value(upper - lower)),
                    ));
                    bars_row.add_key(format!("series_{series_index}"));
                }
            }
        }

        let category_container = WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_direction: match chart.mode {
                        BarMode::Grouped => WidgetFlexDirection::Row,
                        BarMode::Stacked => WidgetFlexDirection::Column,
                    },
                    justify_content: Some(match chart.mode {
                        BarMode::Grouped => WidgetJustifyContent::Center,
                        BarMode::Stacked => WidgetJustifyContent::FlexEnd,
                    }),
                    align_items: Some(match chart.mode {
                        BarMode::Grouped => WidgetAlignItems::FlexEnd,
                        BarMode::Stacked => WidgetAlignItems::Stretch,
                    }),
                    gap: (styles.bar_gap.into(), styles.bar_gap.into()),
                    ..Default::default()
                },
                bars_row,
            ))
            .with_key("bars")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    font_size: 10.0,
                    color: styles.category_label_color,
                    text_alignment: Some(TextAlign::Center),
                    text_wrap: TextWrap::None,
                    margin: Edge::all(0.0).top(4.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: category.clone(),
                },
            ))
            .with_key("label");

        plot.add::<Element>((
            Element,
            WoodpeckerStyle {
                flex_grow: 1.0,
                height: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Column,
                ..Default::default()
            },
            category_container,
        ));
        plot.add_key(format!("category_{cat_index}"));
    }

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_grow: 1.0,
            flex_direction: WidgetFlexDirection::Row,
            align_items: Some(WidgetAlignItems::Stretch),
            gap: (styles.group_gap.into(), 0.0.into()),
            ..Default::default()
        },
        plot,
    ));
    children.add_key("plot");

    if chart.show_legend {
        let entries: Vec<LegendEntry> = chart
            .series
            .iter()
            .map(|s| {
                let mut entry = LegendEntry::new(s.name.clone(), s.color);
                if let Some(&last) = s.values.last() {
                    entry = entry.with_value(format_value(last));
                }
                entry
            })
            .collect();
        children.add::<ChartLegend>((
            ChartLegend { entries },
            WoodpeckerStyle {
                margin: Edge::all(0.0).top(8.0),
                ..Default::default()
            },
        ));
        children.add_key("legend");
    }

    children.apply(current_widget.as_parent());
}
