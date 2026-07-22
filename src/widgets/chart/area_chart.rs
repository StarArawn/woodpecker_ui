use crate::picking_backend::PointerWorldPosition;
use crate::portal::OverlayRoot;
use crate::prelude::*;
use bevy::prelude::*;
use bevy_vello::vello::{kurbo, peniko};

use super::chart_core::{
    axis_label_children, draw_axis_lines, stacked_baselines, to_peniko, AxisOrientation, AxisSpec,
    AxisTick, ChartLegend, ChartSeriesData, LegendEntry, LinearScale, Scale,
};

/// One named series on an [`AreaChart`], stacked on top of every series declared before it.
#[derive(Reflect, Clone, PartialEq, Debug, Default)]
pub struct AreaSeries {
    /// The series name, shown in the legend.
    pub name: String,
    /// The data points to plot, left to right.
    pub data: ChartSeriesData,
    /// The color of this series' fill, border, and legend swatch.
    pub color: Color,
}

impl AreaSeries {
    /// Convenience constructor for a named series.
    pub fn new(name: impl Into<String>, data: impl Into<ChartSeriesData>, color: Color) -> Self {
        Self {
            name: name.into(),
            data: data.into(),
            color,
        }
    }
}

/// Styles for [`AreaChart`].
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct AreaChartStyles {
    /// The color of the horizontal reference grid lines.
    pub grid_color: Color,
    /// The color of Y-axis tick labels.
    pub axis_label_color: Color,
    /// How many horizontal reference lines to draw (evenly spaced).
    pub grid_lines: u8,
    /// The stroke width of each series' top boundary line, in logical pixels.
    pub line_width: f32,
}

impl Default for AreaChartStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for AreaChartStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            grid_color: theme.border,
            axis_label_color: theme.text.with_alpha(0.6),
            grid_lines: 3,
            line_width: 1.5,
        }
    }
}

fn default_chart_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: 220.0.into(),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

/// See `line_chart::LineChartHoverState`'s identical doc comment -- same non-rectangular-hover
/// problem (one continuous fill per series, not a discrete entity per point), same fix.
#[derive(Component, Reflect, Clone, Copy, PartialEq, Default, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
struct AreaChartHoverState {
    index: Option<usize>,
    cursor: Vec2,
}

/// A stacked area chart -- each series' fill sits on top of the running total of every series
/// declared before it, so the topmost boundary reads as the combined total across all series.
///
/// Drawn via one [`WidgetRender::Custom`] closure per render, like [`super::LineChart`] (a
/// continuous fill path per series has no per-segment animation need the way
/// [`super::BarChart`]'s bars do).
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_chart_style(), WidgetChildren, AreaChartStyles, TooltipStyles)]
pub struct AreaChart {
    /// The series to plot, each stacked on top of the ones before it.
    pub series: Vec<AreaSeries>,
    /// Whether to render a legend row below the plot. Hovering an entry shows that series'
    /// last value.
    pub show_legend: bool,
}

impl Default for AreaChart {
    fn default() -> Self {
        Self {
            series: Vec::new(),
            show_legend: true,
        }
    }
}

fn format_tick(value: f32) -> String {
    if value.fract().abs() < 0.05 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

const PLOT_PAD: f32 = 8.0;

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    overlay_root: Option<Res<OverlayRoot>>,
    mut query: Query<(
        &AreaChart,
        &AreaChartStyles,
        &TooltipStyles,
        &mut WidgetChildren,
        &WidgetLayout,
    )>,
    hover_state_query: Query<&AreaChartHoverState>,
) {
    let Ok((chart, styles, tooltip_styles, mut children, layout)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let hover_state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        AreaChartHoverState::default(),
    );
    let hover = hover_state_query
        .get(hover_state_entity)
        .copied()
        .unwrap_or_default();

    *children = WidgetChildren::default();

    let data: Vec<ChartSeriesData> = chart.series.iter().map(|s| s.data.clone()).collect();
    let has_data = data.iter().any(|d| d.len() >= 2);

    let scale = if has_data {
        let bounds = stacked_baselines(&data);
        let max = bounds
            .iter()
            .flat_map(|b| b.iter().map(|(_, upper)| *upper))
            .fold(0.0_f32, f32::max);
        LinearScale {
            min: 0.0,
            max: max.max(f32::EPSILON),
        }
    } else {
        LinearScale { min: 0.0, max: 1.0 }
    };

    let spec = AxisSpec {
        orientation: AxisOrientation::Y,
        ticks: scale
            .ticks(styles.grid_lines)
            .into_iter()
            .map(|v| AxisTick {
                fraction: scale.to_fraction(v),
                label: format_tick(v),
            })
            .collect(),
        line_color: styles.grid_color,
        label_color: styles.axis_label_color,
        font_size: 10.0,
    };

    if has_data && !spec.ticks.is_empty() {
        let plot_size_px = Vec2::new(
            (layout.size.x - PLOT_PAD * 2.0).max(1.0),
            (layout.size.y - PLOT_PAD * 2.0).max(1.0),
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
            axis_label_children(Vec2::new(PLOT_PAD, PLOT_PAD), plot_size_px, &spec),
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

    // See `line_chart::render`'s identical block for why this captures only stable entity ids
    // and re-queries chart data fresh on every fire, rather than closing over `chart`/`styles`.
    let current_widget_val = *current_widget;
    let current_entity = **current_widget;
    children.observe(
        current_widget_val,
        move |trigger: On<Pointer<Move>>,
              chart_query: Query<&AreaChart>,
              layout_query: Query<&WidgetLayout>,
              pointer_world: PointerWorldPosition,
              mut hover_query: Query<&mut AreaChartHoverState>| {
            let Ok(mut hover) = hover_query.get_mut(hover_state_entity) else {
                return;
            };
            let Ok(chart) = chart_query.get(current_entity) else {
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
                // See `line_chart::render`'s identical guard -- an unconditional write here
                // marks `hover` changed (feeding this chart's own re-render gate) on every
                // single pixel of cursor movement, not just when the hovered point changes.
                if hover.index.is_some() {
                    hover.index = None;
                }
                return;
            }

            let plot_x0 = plot_layout.location.x + PLOT_PAD;
            let plot_w = (plot_layout.size.x - PLOT_PAD * 2.0).max(1.0);
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
        move |_trigger: On<Pointer<Out>>, mut hover_query: Query<&mut AreaChartHoverState>| {
            if let Ok(mut hover) = hover_query.get_mut(hover_state_entity) {
                if hover.index.is_some() {
                    hover.index = None;
                }
            }
        },
    );

    if let (Some(index), true) = (hover.index, overlay_root.is_some()) {
        // Each series' own (not cumulative-stacked) value at this index -- the fill's *visual*
        // top edge is the running total, but "how much of this stack is this series" is the
        // number someone hovering actually wants.
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

fn build_plot_render(
    series: Vec<AreaSeries>,
    styles: AreaChartStyles,
    spec: AxisSpec,
) -> WidgetRender {
    WidgetRender::Custom {
        render: WidgetRenderCustom::new(move |scene, layout, _widget_styles, dpi| {
            let data: Vec<ChartSeriesData> = series.iter().map(|s| s.data.clone()).collect();
            let len = data.iter().map(|d| d.len()).max().unwrap_or(0);
            if len < 2 {
                return;
            }

            let dpi = dpi as f64;
            let x0 = layout.location.x as f64 * dpi;
            let y0 = layout.location.y as f64 * dpi;
            let w = layout.size.x as f64 * dpi;
            let h = layout.size.y as f64 * dpi;

            let pad = PLOT_PAD as f64 * dpi;
            let plot_x0 = x0 + pad;
            let plot_y0 = y0 + pad;
            let plot_w = (w - pad * 2.0).max(1.0);
            let plot_h = (h - pad * 2.0).max(1.0);

            let bounds = stacked_baselines(&data);
            let max = bounds
                .iter()
                .flat_map(|b| b.iter().map(|(_, upper)| *upper))
                .fold(0.0_f32, f32::max)
                .max(f32::EPSILON);
            let scale = LinearScale { min: 0.0, max };

            if !spec.ticks.is_empty() {
                draw_axis_lines(
                    scene,
                    kurbo::Point::new(plot_x0, plot_y0),
                    kurbo::Size::new(plot_w, plot_h),
                    &spec,
                );
            }

            let x_at = |i: usize| plot_x0 + (i as f64 / (len - 1) as f64) * plot_w;
            let y_at = |v: f32| plot_y0 + (1.0 - scale.to_fraction(v) as f64) * plot_h;

            for (s, series_bounds) in series.iter().zip(bounds.iter()) {
                if series_bounds.is_empty() {
                    continue;
                }

                let mut path = kurbo::BezPath::new();
                path.move_to(kurbo::Point::new(x_at(0), y_at(series_bounds[0].1)));
                for (i, (_, upper)) in series_bounds.iter().enumerate().skip(1) {
                    path.line_to(kurbo::Point::new(x_at(i), y_at(*upper)));
                }
                for (i, (lower, _)) in series_bounds.iter().enumerate().rev() {
                    path.line_to(kurbo::Point::new(x_at(i), y_at(*lower)));
                }
                path.close_path();

                let fill = to_peniko(s.color).with_alpha(0.55);
                scene.fill(
                    peniko::Fill::NonZero,
                    kurbo::Affine::default(),
                    fill,
                    None,
                    &path,
                );

                let mut border = kurbo::BezPath::new();
                border.move_to(kurbo::Point::new(x_at(0), y_at(series_bounds[0].1)));
                for (i, (_, upper)) in series_bounds.iter().enumerate().skip(1) {
                    border.line_to(kurbo::Point::new(x_at(i), y_at(*upper)));
                }
                scene.stroke(
                    &kurbo::Stroke::new(styles.line_width as f64),
                    kurbo::Affine::default(),
                    to_peniko(s.color),
                    None,
                    &border,
                );
            }
        }),
    }
}
