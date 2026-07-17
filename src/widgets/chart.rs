use crate::prelude::*;
use bevy::prelude::*;
use bevy_vello::vello::{kurbo, peniko};

/// A single named line on a [`LineChart`], plotted in its own color.
#[derive(Reflect, Clone, PartialEq, Debug, Default)]
pub struct ChartSeries {
    /// The series name, shown in the legend.
    pub name: String,
    /// The data points to plot, left to right. Fewer than 2 points draws nothing for this
    /// series (it still appears in the legend).
    pub data: Vec<f32>,
    /// The color of this series' line, fill, and legend swatch.
    pub color: Color,
}

impl ChartSeries {
    /// Convenience constructor for a named series.
    pub fn new(name: impl Into<String>, data: Vec<f32>, color: Color) -> Self {
        Self {
            name: name.into(),
            data,
            color,
        }
    }
}

/// A collection of styles for the [`LineChart`] widget. Per-series color lives on
/// [`ChartSeries`] itself -- these are the properties shared by the whole chart.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ChartStyles {
    /// The color of the horizontal reference grid lines.
    pub grid_color: Color,
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
    /// Font size of legend labels.
    pub legend_font_size: f32,
    /// Text color of legend labels.
    pub legend_text_color: Color,
    /// Size (both width and height) of each legend color swatch.
    pub legend_swatch_size: f32,
    /// Gap between legend items, and between the plot and the legend row.
    pub legend_gap: f32,
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
            line_width: 2.5,
            point_radius: 4.0,
            grid_lines: 3,
            smooth: true,
            show_points: true,
            show_fill: true,
            legend_font_size: 12.0,
            legend_text_color: theme.text,
            legend_swatch_size: 10.0,
            legend_gap: 8.0,
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

/// A dependency-free line chart supporting multiple named series and an optional legend.
///
/// The plot is drawn directly into the vello scene via [`WidgetRender::Custom`]; the legend
/// is built from ordinary child widgets since it needs real text.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_chart_style(), WidgetChildren, ChartStyles)]
pub struct LineChart {
    /// The series to plot, all sharing one Y scale so they're directly comparable.
    pub series: Vec<ChartSeries>,
    /// Whether to render a legend row (name + color swatch per series) below the plot.
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

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&LineChart, &ChartStyles, &mut WidgetChildren)>,
) {
    let Ok((chart, styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    *children = WidgetChildren::default();

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_grow: 1.0,
            ..Default::default()
        },
        build_plot_render(chart.series.clone(), *styles),
    ));
    children.add_key("plot");

    if chart.show_legend && !chart.series.is_empty() {
        let mut legend_children = WidgetChildren::default();
        for series in &chart.series {
            legend_children.add::<Element>(legend_item(series, styles));
            legend_children.add_key(series.name.clone());
        }
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Row,
                flex_wrap: WidgetFlexWrap::Wrap,
                gap: (styles.legend_gap.into(), styles.legend_gap.into()),
                margin: Edge::all(0.0).top(styles.legend_gap),
                ..Default::default()
            },
            legend_children,
        ));
        children.add_key("legend");
    }

    children.apply(current_widget.as_parent());
}

fn legend_item(series: &ChartSeries, styles: &ChartStyles) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: styles.legend_swatch_size.into(),
                    height: styles.legend_swatch_size.into(),
                    background_color: series.color,
                    border_radius: Corner::all(styles.legend_swatch_size / 2.0),
                    margin: Edge::all(0.0).right(6.0),
                    ..Default::default()
                },
                WidgetRender::Quad,
            ))
            .with_key("swatch")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: styles.legend_font_size,
                    color: styles.legend_text_color,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: series.name.clone(),
                },
            ))
            .with_key("label"),
    )
}

fn to_peniko(color: Color) -> peniko::Color {
    let srgba = color.to_srgba();
    peniko::Color::new([srgba.red, srgba.green, srgba.blue, srgba.alpha])
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

/// Builds the (x, y) points for one series, normalized against a shared `(min, max)` Y range
/// so multiple series are directly comparable on one plot.
fn series_points(
    data: &[f32],
    min: f32,
    range: f32,
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
            let norm = ((v - min) / range) as f64;
            kurbo::Point::new(plot_x0 + t * plot_w, plot_y0 + (1.0 - norm) * plot_h)
        })
        .collect()
}

fn build_plot_render(series: Vec<ChartSeries>, styles: ChartStyles) -> WidgetRender {
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

            // Inset so point markers and the stroke's own width never clip against the
            // widget's own edges.
            let pad = ((styles.point_radius.max(styles.line_width)) as f64 + 2.0) * dpi;
            let plot_x0 = x0 + pad;
            let plot_y0 = y0 + pad;
            let plot_w = (w - pad * 2.0).max(1.0);
            let plot_h = (h - pad * 2.0).max(1.0);

            // One shared Y scale across every series so they're directly comparable.
            let min = plottable
                .iter()
                .flat_map(|s| s.data.iter().cloned())
                .fold(f32::INFINITY, f32::min);
            let max = plottable
                .iter()
                .flat_map(|s| s.data.iter().cloned())
                .fold(f32::NEG_INFINITY, f32::max);
            let range = (max - min).max(f32::EPSILON);

            // Grid lines -- evenly spaced horizontal reference lines behind everything else.
            if styles.grid_lines > 0 {
                let grid_brush = to_peniko(styles.grid_color);
                for g in 0..=styles.grid_lines {
                    let gt = g as f64 / styles.grid_lines as f64;
                    let gy = plot_y0 + gt * plot_h;
                    let line = kurbo::Line::new((x0, gy), (x0 + w, gy));
                    scene.stroke(
                        &kurbo::Stroke::new(1.0),
                        kurbo::Affine::default(),
                        grid_brush,
                        None,
                        &line,
                    );
                }
            }

            for s in &plottable {
                let points = series_points(&s.data, min, range, plot_x0, plot_y0, plot_w, plot_h);

                // Build the line path -- smoothed via Catmull-Rom-derived Bezier segments, or
                // a plain polyline if the caller opted out.
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

                // Gradient fill under the curve, fading from the line's own color to fully
                // transparent -- closes the path down to the plot's bottom edge first.
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

                    // The last (current/most-recent) point gets a soft halo behind its marker
                    // so it reads as this series' "current value" at a glance.
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
