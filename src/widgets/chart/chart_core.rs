use std::sync::Arc;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_vello::prelude::UiVelloScene;
use bevy_vello::vello::{kurbo, peniko};

/// Wraps a chart series' numeric data in an `Arc`, so an otherwise-unchanged series compares in
/// O(1) (pointer equality) across renders instead of paying a full `Vec<f32>` deep comparison
/// every frame regardless of whether anything changed -- see `diffing::diff_widget_entity`'s
/// reflection-based `PartialEq` check, which runs on every `DiffableProp` component on every
/// widget entity every frame. Mirrors [`super::super::table::TableRows`]'s identical reasoning
/// for the same problem.
///
/// Cloning a `ChartSeriesData` is cheap -- it just bumps the `Arc`'s refcount, and compares equal
/// to the original via [`Arc::ptr_eq`]. Construct a genuinely new one (`ChartSeriesData::new`, or
/// `data.into()`) only when the underlying values actually changed.
#[derive(Debug, Clone, Reflect)]
pub struct ChartSeriesData(pub Arc<Vec<f32>>);

impl ChartSeriesData {
    /// Wraps `data` for use as a chart series' values.
    pub fn new(data: Vec<f32>) -> Self {
        Self(Arc::new(data))
    }
}

impl Default for ChartSeriesData {
    fn default() -> Self {
        Self(Arc::new(Vec::new()))
    }
}

impl PartialEq for ChartSeriesData {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl std::ops::Deref for ChartSeriesData {
    type Target = [f32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<f32>> for ChartSeriesData {
    fn from(data: Vec<f32>) -> Self {
        Self::new(data)
    }
}

/// A single named line on a [`super::LineChart`]/[`super::AreaChart`], plotted in its own color.
#[derive(Reflect, Clone, PartialEq, Debug, Default)]
pub struct ChartSeries {
    /// The series name, shown in the legend.
    pub name: String,
    /// The data points to plot, left to right. Fewer than 2 points draws nothing for this
    /// series (it still appears in the legend).
    pub data: ChartSeriesData,
    /// The color of this series' line, fill, and legend swatch.
    pub color: Color,
}

impl ChartSeries {
    /// Convenience constructor for a named series.
    pub fn new(name: impl Into<String>, data: impl Into<ChartSeriesData>, color: Color) -> Self {
        Self {
            name: name.into(),
            data: data.into(),
            color,
        }
    }
}

/// Maps a data value to a `0.0..=1.0` fraction across a chart axis' range.
pub trait Scale {
    /// Maps `value` to a `0.0..=1.0` fraction across this scale's range.
    fn to_fraction(&self, value: f32) -> f32;
}

/// A continuous numeric scale spanning `min..=max`.
#[derive(Clone, Copy, Debug)]
pub struct LinearScale {
    /// The value that maps to fraction `0.0`.
    pub min: f32,
    /// The value that maps to fraction `1.0`.
    pub max: f32,
}

impl LinearScale {
    /// Builds a scale spanning the min/max of `values`. Falls back to `0.0..=1.0` if `values`
    /// is empty (no finite min/max to derive a range from).
    pub fn from_values(values: impl Iterator<Item = f32>) -> Self {
        let (min, max) = values.fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), v| {
            (min.min(v), max.max(v))
        });
        if min.is_finite() && max.is_finite() {
            Self { min, max }
        } else {
            Self { min: 0.0, max: 1.0 }
        }
    }

    fn range(&self) -> f32 {
        (self.max - self.min).max(f32::EPSILON)
    }

    /// `count + 1` evenly spaced tick values spanning the scale's range, inclusive of both
    /// ends (`count == 0` returns no ticks at all).
    pub fn ticks(&self, count: u8) -> Vec<f32> {
        if count == 0 {
            return Vec::new();
        }
        (0..=count)
            .map(|i| self.min + self.range() * (i as f32 / count as f32))
            .collect()
    }
}

impl Scale for LinearScale {
    fn to_fraction(&self, value: f32) -> f32 {
        (value - self.min) / self.range()
    }
}

/// A discrete scale spanning one equal-width band per category, left to right.
#[derive(Clone, Debug, Default)]
pub struct CategoricalScale {
    /// The category labels, left to right.
    pub categories: Vec<String>,
}

impl CategoricalScale {
    /// The `(start, end)` fraction this category's band occupies.
    pub fn band(&self, index: usize) -> (f32, f32) {
        let n = self.categories.len().max(1) as f32;
        (index as f32 / n, (index as f32 + 1.0) / n)
    }

    /// The fraction at this category's band center.
    pub fn center(&self, index: usize) -> f32 {
        let (start, end) = self.band(index);
        (start + end) / 2.0
    }
}

/// Which edge of the plot area an [`AxisSpec`]'s ticks run along.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisOrientation {
    /// Ticks run left-to-right along the bottom edge, vertical gridlines.
    X,
    /// Ticks run top-to-bottom along the left edge, horizontal gridlines.
    Y,
}

/// One labeled tick along an axis, `fraction` in `0.0..=1.0` across the plot area.
#[derive(Clone, Debug)]
pub struct AxisTick {
    /// Where this tick sits, `0.0..=1.0` across the plot area along the axis' own orientation.
    pub fraction: f32,
    /// The tick's label text.
    pub label: String,
}

/// A fully-specified axis: which edge it runs along, its ticks, and how it's styled. Built by
/// each chart type from its own [`LinearScale`]/[`CategoricalScale`], then drawn via
/// [`draw_axis_lines`] (gridline geometry, from inside a [`WidgetRender::Custom`] closure) and
/// [`axis_label_children`] (tick text, as real declared children -- text can't be drawn from a
/// `Custom` closure, only laid out via ordinary widget children).
#[derive(Clone, Debug)]
pub struct AxisSpec {
    /// Which edge the ticks run along.
    pub orientation: AxisOrientation,
    /// The ticks to draw/label.
    pub ticks: Vec<AxisTick>,
    /// The color of gridlines drawn by [`draw_axis_lines`].
    pub line_color: Color,
    /// The text color of labels built by [`axis_label_children`].
    pub label_color: Color,
    /// The font size of labels built by [`axis_label_children`].
    pub font_size: f32,
}

pub(super) fn to_peniko(color: Color) -> peniko::Color {
    let srgba = color.to_srgba();
    peniko::Color::new([srgba.red, srgba.green, srgba.blue, srgba.alpha])
}

/// Draws `spec`'s gridlines into `scene`. Call from inside a [`WidgetRender::Custom`] closure;
/// `plot_origin`/`plot_size` must already be dpi-scaled layout-space pixels (matching the
/// closure's own `(scene, layout, style, dpi)` convention).
pub fn draw_axis_lines(
    scene: &mut UiVelloScene,
    plot_origin: kurbo::Point,
    plot_size: kurbo::Size,
    spec: &AxisSpec,
) {
    let brush = to_peniko(spec.line_color);
    for tick in &spec.ticks {
        let line = match spec.orientation {
            AxisOrientation::Y => {
                let y = plot_origin.y + (1.0 - tick.fraction as f64) * plot_size.height;
                kurbo::Line::new((plot_origin.x, y), (plot_origin.x + plot_size.width, y))
            }
            AxisOrientation::X => {
                let x = plot_origin.x + tick.fraction as f64 * plot_size.width;
                kurbo::Line::new((x, plot_origin.y), (x, plot_origin.y + plot_size.height))
            }
        };
        scene.stroke(
            &kurbo::Stroke::new(1.0),
            kurbo::Affine::default(),
            brush,
            None,
            &line,
        );
    }
}

/// Builds real text children for `spec`'s tick labels, absolutely positioned against
/// `plot_origin_px`/`plot_size_px` (logical, *not* dpi-scaled -- `WidgetLayout` units, unlike
/// [`draw_axis_lines`]'s dpi-scaled vello-space points).
pub fn axis_label_children(plot_origin_px: Vec2, plot_size_px: Vec2, spec: &AxisSpec) -> WidgetChildren {
    let mut children = WidgetChildren::default();
    for (i, tick) in spec.ticks.iter().enumerate() {
        let (left, top) = match spec.orientation {
            AxisOrientation::Y => (
                0.0,
                (plot_origin_px.y + (1.0 - tick.fraction) * plot_size_px.y - spec.font_size * 0.5)
                    .max(0.0),
            ),
            AxisOrientation::X => (
                (plot_origin_px.x + tick.fraction * plot_size_px.x - 16.0).max(0.0),
                plot_origin_px.y + plot_size_px.y + 4.0,
            ),
        };
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                left: left.into(),
                top: top.into(),
                font_size: spec.font_size,
                color: spec.label_color,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: tick.label.clone(),
            },
        ));
        children.add_key(format!("tick_{i}"));
    }
    children
}

/// Builds a filled wedge path for a pie/donut slice, for [`super::PieChart`]. `start_angle`/
/// `sweep_angle` are radians, `0.0` pointing straight up (12 o'clock) and increasing clockwise
/// -- the convention every slice angle in this module is computed in. `inner_radius` of `0.0`
/// yields a plain pie wedge (a triangular sector out to the center); anything greater yields a
/// donut segment (an annular sector).
///
/// Approximated as a many-sided polygon rather than built from `kurbo::Arc` directly, to avoid
/// depending on that type's exact flattening API (which has shifted across `kurbo` versions) --
/// at chart sizes this crate targets, the segment count below is visually indistinguishable
/// from a true arc while staying trivially cheap to recompute every time a chart's data changes.
pub fn wedge_path(
    center: kurbo::Point,
    inner_radius: f64,
    outer_radius: f64,
    start_angle: f64,
    sweep_angle: f64,
) -> kurbo::BezPath {
    const SEGMENTS_PER_TURN: f64 = 90.0;
    let segment_count = ((sweep_angle.abs() / std::f64::consts::TAU) * SEGMENTS_PER_TURN)
        .ceil()
        .max(2.0) as usize;

    let point_at = |radius: f64, angle: f64| {
        let a = angle - std::f64::consts::FRAC_PI_2;
        kurbo::Point::new(center.x + radius * a.cos(), center.y + radius * a.sin())
    };

    let outer_points: Vec<kurbo::Point> = (0..=segment_count)
        .map(|i| {
            let t = i as f64 / segment_count as f64;
            point_at(outer_radius, start_angle + sweep_angle * t)
        })
        .collect();

    let mut path = kurbo::BezPath::new();
    path.move_to(outer_points[0]);
    for p in &outer_points[1..] {
        path.line_to(*p);
    }

    if inner_radius > 0.0 {
        for i in (0..=segment_count).rev() {
            let t = i as f64 / segment_count as f64;
            path.line_to(point_at(inner_radius, start_angle + sweep_angle * t));
        }
    } else {
        path.line_to(center);
    }
    path.close_path();
    path
}

/// Cycles through the theme's semantic accent colors for a multi-series chart that wasn't given
/// explicit per-series colors by its caller.
pub fn default_series_palette(theme: &Theme, n: usize) -> Vec<Color> {
    let cycle = [
        theme.primary,
        theme.success,
        theme.warning,
        theme.danger,
        theme.primary_light,
    ];
    (0..n).map(|i| cycle[i % cycle.len()]).collect()
}

/// For each category index, the cumulative `(lower, upper)` bound of every series stacked on
/// top of the previous ones -- shared by [`super::BarChart`]'s `Stacked` mode and
/// [`super::AreaChart`], which both need the same running-total baseline math.
pub fn stacked_baselines(series: &[ChartSeriesData]) -> Vec<Vec<(f32, f32)>> {
    let Some(len) = series.iter().map(|s| s.len()).max() else {
        return Vec::new();
    };
    let mut running = vec![0.0_f32; len];
    series
        .iter()
        .map(|s| {
            (0..len)
                .map(|i| {
                    let v = s.get(i).copied().unwrap_or(0.0);
                    let lower = running[i];
                    running[i] += v;
                    (lower, running[i])
                })
                .collect()
        })
        .collect()
}

/// One color-swatch + label pair in a [`ChartLegend`].
#[derive(Reflect, Clone, PartialEq, Debug, Default)]
pub struct LegendEntry {
    /// The row's label text.
    pub label: String,
    /// The row's swatch color.
    pub color: Color,
    /// When set, hovering this entry shows it in a [`Tooltip`] -- the one hover affordance every
    /// chart type in this module gets "for free" through this shared widget, since a legend row
    /// is always a plain rectangle (unlike a pie wedge or a point on a line, which
    /// `picking_backend.rs`'s rectangle-only hit testing can't hover precisely).
    pub value: Option<String>,
}

impl LegendEntry {
    /// A legend row with no hover value.
    pub fn new(label: impl Into<String>, color: Color) -> Self {
        Self {
            label: label.into(),
            color,
            value: None,
        }
    }

    /// Sets the text shown when this row is hovered.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}

/// Styles for [`ChartLegend`].
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ChartLegendStyles {
    /// Font size of legend labels.
    pub font_size: f32,
    /// Text color of legend labels.
    pub text_color: Color,
    /// Size (both width and height) of each legend color swatch.
    pub swatch_size: f32,
    /// Gap between legend items, and between wrapped rows.
    pub gap: f32,
}

impl Default for ChartLegendStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ChartLegendStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            font_size: 12.0,
            text_color: theme.text,
            swatch_size: 10.0,
            gap: 8.0,
        }
    }
}

fn default_legend_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        flex_direction: WidgetFlexDirection::Row,
        flex_wrap: WidgetFlexWrap::Wrap,
        ..Default::default()
    }
}

/// A row of color-swatch + label pairs -- the legend for every chart type in this module.
/// Extracted out of what was originally `LineChart`'s own inlined legend-building code so each
/// chart type declares one `ChartLegend { entries }` child instead of duplicating it.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(legend_render)]
#[require(WoodpeckerStyle = default_legend_style(), WidgetChildren, ChartLegendStyles)]
pub struct ChartLegend {
    /// The rows to render, top to bottom (wrapping into a grid once the row runs out of width).
    pub entries: Vec<LegendEntry>,
}

fn legend_render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&ChartLegend, &ChartLegendStyles, &mut WidgetChildren)>,
) {
    let Ok((legend, styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    *children = WidgetChildren::default();
    for entry in &legend.entries {
        let inner_row = WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: styles.swatch_size.into(),
                        height: styles.swatch_size.into(),
                        background_color: entry.color,
                        border_radius: Corner::all(styles.swatch_size / 2.0),
                        margin: Edge::all(0.0).right(6.0),
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                ))
                .with_key("swatch")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: styles.font_size,
                        color: styles.text_color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: entry.label.clone(),
                    },
                ))
                .with_key("label"),
        ));
        let spacing_style = WoodpeckerStyle {
            margin: Edge::all(0.0).right(styles.gap).bottom(styles.gap),
            ..Default::default()
        };

        if let Some(value) = &entry.value {
            children.add::<Tooltip>((
                Tooltip {
                    text: format!("{}: {value}", entry.label),
                    placement: PopoverPlacement::Top,
                },
                spacing_style,
                PassedChildren(inner_row),
            ));
        } else {
            children.add::<Element>((Element, spacing_style, inner_row));
        }
        children.add_key(entry.label.clone());
    }

    children.apply(current_widget.as_parent());
}

/// Wraps `visual` (real declared children -- typically one `Element` with a colored
/// `WidgetRender::Quad`) in the existing [`Tooltip`] widget so hovering shows `text`. The
/// caller still supplies its own `WoodpeckerStyle` alongside this in the same spawn tuple to
/// size/position the resulting `Tooltip` entity within its parent's layout.
///
/// Fits shapes `picking_backend.rs`'s rectangle-only hit testing already handles exactly (i.e.
/// bars). Non-rectangular hover (pie wedges, nearest-point on a line) can't use this -- those
/// compute the hovered element manually (angle/nearest-X math against cached geometry) and float
/// a themed label directly instead of relying on `Tooltip`'s own per-child `Pointer<Over>`.
pub fn chart_tooltip(
    text: impl Into<String>,
    placement: PopoverPlacement,
    visual: WidgetChildren,
) -> impl Bundle + Clone {
    (
        Tooltip {
            text: text.into(),
            placement,
        },
        PassedChildren(visual),
    )
}
