use crate::picking_backend::PointerWorldPosition;
use crate::portal::OverlayRoot;
use crate::prelude::*;
use bevy::prelude::*;
use bevy_vello::vello::{kurbo, peniko};

use super::chart_core::{
    axis_label_children, draw_axis_lines, to_peniko, AxisOrientation, AxisSpec, AxisTick,
    ChartLegend, ChartSeries, LegendEntry, LinearScale, Scale,
};

/// A collection of styles for the [`LineChart`] widget. Per-series color lives on
/// [`ChartSeries`] itself -- these are the properties shared by the whole chart.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ChartStyles {
    /// The color of the horizontal reference grid lines and their labels.
    pub grid_color: Color,
    /// The color of axis tick labels.
    pub axis_label_color: Color,
    /// The stroke width of each line, in logical pixels.
    pub line_width: f32,
    /// The radius of each data point marker, in logical pixels.
    pub point_radius: f32,
    /// How many horizontal reference lines to draw (evenly spaced).
    pub grid_lines: u8,
    /// Whether to smooth each line through a Catmull-Rom curve instead of straight segments.
    pub smooth: bool,
    /// Whether to draw the circle markers at each data point.
    pub show_points: bool,
    /// Whether to draw the gradient fill under each line. Consider turning this off for
    /// charts with more than one or two series -- overlapping fills get muddy fast.
    pub show_fill: bool,
}

impl Default for ChartStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ChartStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            grid_color: theme.border,
            axis_label_color: theme.text.with_alpha(0.6),
            line_width: 2.5,
            point_radius: 4.0,
            grid_lines: 3,
            smooth: true,
            show_points: true,
            show_fill: true,
        }
    }
}

pub(super) fn default_chart_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: 220.0.into(),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

/// Which data point index the pointer is nearest, and where to float the value label -- a line
/// has no discrete per-point entity `Tooltip`'s own `Pointer<Over>` could hover (it's all one
/// `WidgetRender::Custom` draw call, per this widget's own doc comment), so hover here is driven
/// manually: a `Pointer<Move>`/`Pointer<Out>` pair on the plot area itself computes the nearest
/// index by X position against the exact same `plot_pad`/point-spacing math the vello closure
/// draws with, rather than relying on picking to hit anything finer-grained than the whole plot.
#[derive(Component, Reflect, Clone, Copy, PartialEq, Default, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
struct LineChartHoverState {
    index: Option<usize>,
    /// Screen-space cursor position as of the last `Pointer<Move>`, for positioning the
    /// floating label -- not world/viewport space, since the label itself is `Fixed`-positioned
    /// (portaled to `OverlayRoot`, same as `Tooltip`'s own label) against the real window.
    cursor: Vec2,
}

/// A dependency-free line chart supporting multiple named series, labeled axes, and an
/// optional legend.
///
/// The plot is drawn directly into the vello scene via [`WidgetRender::Custom`]; axis labels
/// and the legend are built from ordinary child widgets since they need real text.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_chart_style(), WidgetChildren, ChartStyles, TooltipStyles)]
pub struct LineChart {
    /// The series to plot, all sharing one Y scale so they're directly comparable.
    pub series: Vec<ChartSeries>,
    /// Whether to render a legend row (name + color swatch per series) below the plot. Hovering
    /// an entry shows that series' latest value.
    pub show_legend: bool,
}

impl Default for LineChart {
    fn default() -> Self {
        Self {
            series: Vec::new(),
            show_legend: true,
        }
    }
}

fn axis_spec(scale: &LinearScale, grid_lines: u8, styles: &ChartStyles) -> AxisSpec {
    AxisSpec {
        orientation: AxisOrientation::Y,
        ticks: scale
            .ticks(grid_lines)
            .into_iter()
            .map(|value| AxisTick {
                fraction: scale.to_fraction(value),
                label: format_tick(value),
            })
            .collect(),
        line_color: styles.grid_color,
        label_color: styles.axis_label_color,
        font_size: 10.0,
    }
}

fn format_tick(value: f32) -> String {
    if (value.fract()).abs() < 0.05 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

/// The plot area's inset (in logical pixels) so point markers and the stroke's own width never
/// clip against the widget's own edges -- shared between the vello closure (dpi-scaled) and the
/// axis label layout (logical), so both agree on where the plot area actually starts.
fn plot_pad(styles: &ChartStyles) -> f32 {
    styles.point_radius.max(styles.line_width) + 2.0
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    overlay_root: Option<Res<OverlayRoot>>,
    mut query: Query<(
        &LineChart,
        &ChartStyles,
        &TooltipStyles,
        &mut WidgetChildren,
        &WidgetLayout,
    )>,
    hover_state_query: Query<&LineChartHoverState>,
) {
    let Ok((chart, styles, tooltip_styles, mut children, layout)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let hover_state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        LineChartHoverState::default(),
    );
    let hover = hover_state_query
        .get(hover_state_entity)
        .copied()
        .unwrap_or_default();

    *children = WidgetChildren::default();

    let plottable: Vec<&ChartSeries> = chart.series.iter().filter(|s| s.data.len() >= 2).collect();
    let scale = LinearScale::from_values(plottable.iter().flat_map(|s| s.data.iter().copied()));
    let spec = axis_spec(&scale, styles.grid_lines, styles);

    if styles.grid_lines > 0 && !plottable.is_empty() {
        let pad = plot_pad(styles);
        let plot_size_px = Vec2::new(
            (layout.size.x - pad * 2.0).max(1.0),
            (layout.size.y - pad * 2.0).max(1.0),
        );
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                left: 0.0.into(),
                top: 0.0.into(),
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                ..Default::default()
            },
            axis_label_children(Vec2::new(pad, pad), plot_size_px, &spec),
        ));
        children.add_key("y_axis_labels");
    }

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_grow: 1.0,
            ..Default::default()
        },
        Pickable::default(),
        build_plot_render(chart.series.clone(), *styles, spec),
    ));
    children.add_key("plot");

    // Attached *after* `add_key("plot")`, so `observe` (last-added child) lands on the plot
    // area itself, not `y_axis_labels`. Captures only `current_widget_val`/`hover_state_entity`
    // -- stable entity ids, never chart *data* -- and re-queries `LineChart`/`ChartStyles` fresh
    // on every fire; this observer is spawned once and reused for the widget's whole lifetime
    // (see `WidgetChildren::droppable`'s own doc comment for the exact staleness bug this
    // dodges), so anything captured by value here would be frozen at whatever it was on first
    // mount even as the chart's own data changes on later renders.
    let current_widget_val = *current_widget;
    let current_entity = **current_widget;
    children.observe(
        current_widget_val,
        move |trigger: On<Pointer<Move>>,
              chart_query: Query<(&LineChart, &ChartStyles)>,
              layout_query: Query<&WidgetLayout>,
              pointer_world: PointerWorldPosition,
              mut hover_query: Query<&mut LineChartHoverState>| {
            let Ok(mut hover) = hover_query.get_mut(hover_state_entity) else {
                return;
            };
            let Ok((chart, styles)) = chart_query.get(current_entity) else {
                return;
            };
            let Ok(plot_layout) = layout_query.get(trigger.entity) else {
                return;
            };
            let Some(world_pos) = pointer_world.convert(trigger.pointer_location.position) else {
                return;
            };

            let max_len = chart.series.iter().map(|s| s.data.len()).max().unwrap_or(0);
            if max_len < 2 {
                // Guarded, not unconditional: `hover` is a `Mut<T>` -- an assignment marks it
                // changed (feeding this widget's own `DiffableProp` re-render gate) regardless
                // of whether the value actually differs, but a *read* (this comparison) doesn't.
                // `Pointer<Move>` fires on every single pixel of cursor movement, so writing
                // unconditionally here rebuilt this chart's entire `WidgetChildren` tree --
                // including its `WidgetRender::Custom` closure -- dozens of times a second while
                // hovering, which is what visibly destabilized its own layout (confirmed via
                // user report: every chart type wired up this way "bounces" a few pixels while
                // every other chart, sharing no such observer, sits still). Only actually
                // writing when the nearest index changes cuts that down to "whenever the
                // hovered point changes", which is the only time anything visible needs to.
                if hover.index.is_some() {
                    hover.index = None;
                }
                return;
            }

            let pad = plot_pad(styles);
            let plot_x0 = plot_layout.location.x + pad;
            let plot_w = (plot_layout.size.x - pad * 2.0).max(1.0);
            let t = ((world_pos.x - plot_x0) / plot_w).clamp(0.0, 1.0);
            let index = ((t * (max_len - 1) as f32).round() as usize).min(max_len - 1);

            if hover.index != Some(index) {
                hover.index = Some(index);
                hover.cursor = trigger.pointer_location.position;
            }
        },
    );
    children.observe(
        current_widget_val,
        move |_trigger: On<Pointer<Out>>, mut hover_query: Query<&mut LineChartHoverState>| {
            if let Ok(mut hover) = hover_query.get_mut(hover_state_entity) {
                if hover.index.is_some() {
                    hover.index = None;
                }
            }
        },
    );

    if let (Some(index), true) = (hover.index, overlay_root.is_some()) {
        let lines: Vec<String> = chart
            .series
            .iter()
            .filter_map(|s| s.data.get(index).map(|&v| format!("{}: {}", s.name, format_tick(v))))
            .collect();
        if !lines.is_empty() {
            let mut label_children = WidgetChildren::default();
            for line in lines {
                label_children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: tooltip_styles.font_size,
                        color: tooltip_styles.text_color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text { content: line.clone() },
                ));
                label_children.add_key(line);
            }

            children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    position: WidgetPosition::Fixed,
                    left: (hover.cursor.x + 12.0).into(),
                    top: (hover.cursor.y + 12.0).into(),
                    background_color: tooltip_styles.background_color,
                    border_radius: Corner::all(tooltip_styles.border_radius),
                    padding: Edge::all(0.0).left(8.0).right(8.0).top(4.0).bottom(4.0),
                    box_shadow: Some(tooltip_styles.box_shadow),
                    z_index: Some(WidgetZ::Global(StackingTier::Tooltip as u32)),
                    flex_direction: WidgetFlexDirection::Column,
                    ..Default::default()
                },
                WidgetRender::Quad,
                Pickable::IGNORE,
                label_children,
            ));
            children.add_key("hover_tooltip");
            children.portal();
        }
    }

    if chart.show_legend && !chart.series.is_empty() {
        let entries: Vec<LegendEntry> = chart
            .series
            .iter()
            .map(|s| {
                let mut entry = LegendEntry::new(s.name.clone(), s.color);
                if let Some(&last) = s.data.last() {
                    entry = entry.with_value(format_tick(last));
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

/// Converts 4 points of a Catmull-Rom spline segment (`p1`..`p2` is the segment being drawn,
/// `p0`/`p3` are the neighbors shaping the tangents) into a cubic Bezier control pair.
fn catmull_rom_to_bezier(
    p0: kurbo::Point,
    p1: kurbo::Point,
    p2: kurbo::Point,
    p3: kurbo::Point,
) -> (kurbo::Point, kurbo::Point) {
    let c1 = kurbo::Point::new(p1.x + (p2.x - p0.x) / 6.0, p1.y + (p2.y - p0.y) / 6.0);
    let c2 = kurbo::Point::new(p2.x - (p3.x - p1.x) / 6.0, p2.y - (p3.y - p1.y) / 6.0);
    (c1, c2)
}

/// Builds the (x, y) points for one series, normalized against `scale` so multiple series are
/// directly comparable on one plot.
fn series_points(
    data: &[f32],
    scale: &LinearScale,
    plot_x0: f64,
    plot_y0: f64,
    plot_w: f64,
    plot_h: f64,
) -> Vec<kurbo::Point> {
    data.iter()
        .enumerate()
        .map(|(i, &v)| {
            let t = if data.len() > 1 {
                i as f64 / (data.len() - 1) as f64
            } else {
                0.0
            };
            let norm = scale.to_fraction(v) as f64;
            kurbo::Point::new(plot_x0 + t * plot_w, plot_y0 + (1.0 - norm) * plot_h)
        })
        .collect()
}

fn build_plot_render(
    series: Vec<ChartSeries>,
    styles: ChartStyles,
    spec: AxisSpec,
) -> WidgetRender {
    WidgetRender::Custom {
        render: WidgetRenderCustom::new(move |scene, layout, _widget_styles, dpi| {
            let plottable: Vec<&ChartSeries> =
                series.iter().filter(|s| s.data.len() >= 2).collect();
            if plottable.is_empty() {
                return;
            }

            let dpi = dpi as f64;
            let x0 = layout.location.x as f64 * dpi;
            let y0 = layout.location.y as f64 * dpi;
            let w = layout.size.x as f64 * dpi;
            let h = layout.size.y as f64 * dpi;

            let pad = plot_pad(&styles) as f64 * dpi;
            let plot_x0 = x0 + pad;
            let plot_y0 = y0 + pad;
            let plot_w = (w - pad * 2.0).max(1.0);
            let plot_h = (h - pad * 2.0).max(1.0);

            let scale =
                LinearScale::from_values(plottable.iter().flat_map(|s| s.data.iter().copied()));

            if !spec.ticks.is_empty() {
                draw_axis_lines(
                    scene,
                    kurbo::Point::new(plot_x0, plot_y0),
                    kurbo::Size::new(plot_w, plot_h),
                    &spec,
                );
            }

            for s in &plottable {
                let points = series_points(&s.data, &scale, plot_x0, plot_y0, plot_w, plot_h);

                let mut line_path = kurbo::BezPath::new();
                line_path.move_to(points[0]);
                if styles.smooth {
                    for i in 0..points.len() - 1 {
                        let p0 = if i == 0 { points[i] } else { points[i - 1] };
                        let p1 = points[i];
                        let p2 = points[i + 1];
                        let p3 = points.get(i + 2).copied().unwrap_or(p2);
                        let (c1, c2) = catmull_rom_to_bezier(p0, p1, p2, p3);
                        line_path.curve_to(c1, c2, p2);
                    }
                } else {
                    for p in &points[1..] {
                        line_path.line_to(*p);
                    }
                }

                if styles.show_fill {
                    let mut fill_path = line_path.clone();
                    fill_path.line_to(kurbo::Point::new(
                        points[points.len() - 1].x,
                        plot_y0 + plot_h,
                    ));
                    fill_path.line_to(kurbo::Point::new(points[0].x, plot_y0 + plot_h));
                    fill_path.close_path();

                    let top = to_peniko(s.color).with_alpha(0.3);
                    let bottom = to_peniko(s.color).with_alpha(0.0);
                    let gradient = peniko::Gradient::new_linear(
                        (plot_x0, plot_y0),
                        (plot_x0, plot_y0 + plot_h),
                    )
                    .with_stops([top, bottom]);
                    scene.fill(
                        peniko::Fill::NonZero,
                        kurbo::Affine::default(),
                        &gradient,
                        None,
                        &fill_path,
                    );
                }

                let line_brush = to_peniko(s.color);
                scene.stroke(
                    &kurbo::Stroke::new(styles.line_width as f64),
                    kurbo::Affine::default(),
                    line_brush,
                    None,
                    &line_path,
                );

                if styles.show_points {
                    for p in &points[..points.len() - 1] {
                        let circle = kurbo::Circle::new(*p, styles.point_radius as f64 * dpi);
                        scene.fill(
                            peniko::Fill::NonZero,
                            kurbo::Affine::default(),
                            line_brush,
                            None,
                            &circle,
                        );
                    }

                    let last = points[points.len() - 1];
                    let halo = kurbo::Circle::new(last, styles.point_radius as f64 * dpi * 2.2);
                    scene.fill(
                        peniko::Fill::NonZero,
                        kurbo::Affine::default(),
                        to_peniko(s.color).with_alpha(0.25),
                        None,
                        &halo,
                    );
                    let marker = kurbo::Circle::new(last, styles.point_radius as f64 * dpi);
                    scene.fill(
                        peniko::Fill::NonZero,
                        kurbo::Affine::default(),
                        line_brush,
                        None,
                        &marker,
                    );
                }
            }
        }),
    }
}
