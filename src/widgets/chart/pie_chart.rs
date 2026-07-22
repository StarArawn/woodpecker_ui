use crate::picking_backend::PointerWorldPosition;
use crate::portal::OverlayRoot;
use crate::prelude::*;
use bevy::prelude::*;
use bevy_vello::vello::{kurbo, peniko};

use super::chart_core::{to_peniko, wedge_path, ChartLegend, LegendEntry};

/// One slice of a [`PieChart`].
#[derive(Reflect, Clone, PartialEq, Debug, Default)]
pub struct PieSlice {
    /// The slice's label, shown in the legend.
    pub label: String,
    /// The slice's value -- its share of the total is `value / sum(all slice values)`.
    pub value: f32,
    /// The slice's fill and legend swatch color.
    pub color: Color,
}

impl PieSlice {
    /// Convenience constructor for a named slice.
    pub fn new(label: impl Into<String>, value: f32, color: Color) -> Self {
        Self {
            label: label.into(),
            value,
            color,
        }
    }
}

/// Styles for [`PieChart`].
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct PieChartStyles {
    /// The angle (degrees, clockwise from 12 o'clock) the first slice starts at.
    pub start_angle_deg: f32,
    /// A thin gap left between slices, in degrees, for visual separation.
    pub gap_deg: f32,
    /// Font size of the donut-hole center label (only drawn when `PieChart::center_label` is
    /// `Some` and `PieChart::inner_radius > 0.0`).
    pub center_label_font_size: f32,
    /// Text color of the center label.
    pub center_label_color: Color,
}

impl Default for PieChartStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for PieChartStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            start_angle_deg: 0.0,
            gap_deg: 1.5,
            center_label_font_size: 18.0,
            center_label_color: theme.text,
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

/// See `line_chart::LineChartHoverState`'s identical doc comment -- a wedge is even further
/// from a real pickable entity than a line's points (there's no per-point rect fallback at
/// all), so hover here is entirely angle/radius math against the plot's own resolved geometry,
/// replicated to exactly match `wedge_path`'s own angle convention (see `hovered_slice_index`).
#[derive(Component, Reflect, Clone, Copy, PartialEq, Default, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
struct PieChartHoverState {
    index: Option<usize>,
    cursor: Vec2,
}

/// Which slice (if any) contains `cursor_angle` -- a clockwise-from-12-o'clock angle in the
/// *same* convention `wedge_path`'s own `point_at` closure draws with (`angle - FRAC_PI_2` before
/// `cos`/`sin`), relative to `start_angle_deg`. Mirrors `build_plot_render`'s own
/// slice-by-slice angle accumulation exactly, so hit-testing agrees with what's actually drawn.
fn hovered_slice_index(cursor_angle: f64, slices: &[PieSlice], total: f32) -> Option<usize> {
    if total <= 0.0 {
        return None;
    }
    let mut angle = 0.0_f64;
    for (index, slice) in slices.iter().enumerate() {
        let fraction = (slice.value.max(0.0) as f64) / total as f64;
        let sweep = fraction * std::f64::consts::TAU;
        if cursor_angle >= angle && cursor_angle < angle + sweep {
            return Some(index);
        }
        angle += sweep;
    }
    None
}

/// A pie or donut chart (`inner_radius > 0.0`) for showing proportions of a whole.
///
/// Wedges are drawn in one [`WidgetRender::Custom`] closure via [`wedge_path`]. Since a wedge is
/// not a real pickable entity `picking_backend.rs` (rectangle-only) can hit-test, per-wedge
/// hover is driven manually -- angle/radius math against the plot's own resolved geometry (see
/// `hovered_slice_index`) -- the same technique [`super::LineChart`]/[`super::AreaChart`] use
/// for their own non-rectangular hover. Hovering a slice's [`ChartLegend`] entry works too.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_chart_style(), WidgetChildren, PieChartStyles, TooltipStyles)]
pub struct PieChart {
    /// The slices to plot, their share computed as a fraction of the total across all slices.
    pub slices: Vec<PieSlice>,
    /// `0.0` draws a plain pie; anything greater draws a donut with this hole radius (in
    /// logical pixels).
    pub inner_radius: f32,
    /// Text shown in the donut hole -- only meaningful when `inner_radius > 0.0`.
    pub center_label: Option<String>,
    /// Whether to render a legend row below the plot. Hovering an entry shows that slice's
    /// value and percentage of the total.
    pub show_legend: bool,
}

impl Default for PieChart {
    fn default() -> Self {
        Self {
            slices: Vec::new(),
            inner_radius: 0.0,
            center_label: None,
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

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    overlay_root: Option<Res<OverlayRoot>>,
    mut query: Query<(&PieChart, &PieChartStyles, &TooltipStyles, &mut WidgetChildren)>,
    hover_state_query: Query<&PieChartHoverState>,
) {
    let Ok((chart, styles, tooltip_styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let hover_state_entity =
        hooks.use_state(&mut commands, *current_widget, PieChartHoverState::default());
    let hover = hover_state_query
        .get(hover_state_entity)
        .copied()
        .unwrap_or_default();

    *children = WidgetChildren::default();

    let total: f32 = chart.slices.iter().map(|s| s.value.max(0.0)).sum();

    let mut plot_children = WidgetChildren::default();
    if chart.inner_radius > 0.0 {
        let mut overlay = WidgetChildren::default();
        if let Some(label) = &chart.center_label {
            overlay.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: styles.center_label_font_size,
                    color: styles.center_label_color,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: label.clone(),
                },
            ));
            overlay.add_key("center_label_text");
        }
        plot_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                left: 0.0.into(),
                top: 0.0.into(),
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Column,
                align_items: Some(WidgetAlignItems::Center),
                justify_content: Some(WidgetJustifyContent::Center),
                ..Default::default()
            },
            overlay,
        ));
        plot_children.add_key("center_overlay");
    }

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_grow: 1.0,
            ..Default::default()
        },
        Pickable::default(),
        plot_children,
        build_plot_render(chart.slices.clone(), *styles, chart.inner_radius, total),
    ));
    children.add_key("plot");

    // See `line_chart::render`'s identical block for why this captures only stable entity ids
    // and re-queries chart data fresh on every fire, rather than closing over `chart`/`styles`.
    let current_widget_val = *current_widget;
    let current_entity = **current_widget;
    children.observe(
        current_widget_val,
        move |trigger: On<Pointer<Move>>,
              chart_query: Query<(&PieChart, &PieChartStyles)>,
              layout_query: Query<&WidgetLayout>,
              pointer_world: PointerWorldPosition,
              mut hover_query: Query<&mut PieChartHoverState>| {
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

            let total: f32 = chart.slices.iter().map(|s| s.value.max(0.0)).sum();
            let center = plot_layout.location + plot_layout.size * 0.5;
            let outer_radius = (plot_layout.size.x.min(plot_layout.size.y) * 0.5 - 2.0).max(1.0);
            let inner_radius = chart.inner_radius.min(outer_radius * 0.95);
            let offset = world_pos - center;
            let radius = offset.length();

            // See `line_chart::render`'s identical guard on every branch below -- an
            // unconditional write marks `hover` changed (feeding this chart's own re-render
            // gate) on every single pixel of cursor movement, not just when the hovered slice
            // changes.
            if radius > outer_radius || (inner_radius > 0.0 && radius < inner_radius) {
                if hover.index.is_some() {
                    hover.index = None;
                }
                return;
            }

            // Matches `wedge_path`'s own `point_at` convention exactly (see this file's own
            // `hovered_slice_index` doc comment): invert `a = angle - FRAC_PI_2`, `(cos a, sin
            // a)`, then subtract the chart's own start angle and wrap into `[0, TAU)`.
            let raw_angle = (offset.y as f64).atan2(offset.x as f64) + std::f64::consts::FRAC_PI_2;
            let relative = raw_angle - (styles.start_angle_deg as f64).to_radians();
            let cursor_angle = relative.rem_euclid(std::f64::consts::TAU);

            let index = hovered_slice_index(cursor_angle, &chart.slices, total);
            if hover.index != index {
                hover.index = index;
                hover.cursor = trigger.pointer_location.position;
            }
        },
    );
    children.observe(
        current_widget_val,
        move |_trigger: On<Pointer<Out>>, mut hover_query: Query<&mut PieChartHoverState>| {
            if let Ok(mut hover) = hover_query.get_mut(hover_state_entity) {
                if hover.index.is_some() {
                    hover.index = None;
                }
            }
        },
    );

    if let (Some(index), true) = (hover.index, overlay_root.is_some()) {
        if let Some(slice) = chart.slices.get(index) {
            let pct = if total > 0.0 { slice.value / total * 100.0 } else { 0.0 };
            let text = format!("{}: {} ({pct:.0}%)", slice.label, format_value(slice.value));

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
                    ..Default::default()
                },
                WidgetRender::Quad,
                Pickable::IGNORE,
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: tooltip_styles.font_size,
                        color: tooltip_styles.text_color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text { content: text },
                )),
            ));
            children.add_key("hover_tooltip");
            children.portal();
        }
    }

    if chart.show_legend && !chart.slices.is_empty() {
        let entries: Vec<LegendEntry> = chart
            .slices
            .iter()
            .map(|s| {
                let pct = if total > 0.0 { s.value / total * 100.0 } else { 0.0 };
                LegendEntry::new(s.label.clone(), s.color)
                    .with_value(format!("{} ({pct:.0}%)", format_value(s.value)))
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
    slices: Vec<PieSlice>,
    styles: PieChartStyles,
    inner_radius: f32,
    total: f32,
) -> WidgetRender {
    WidgetRender::Custom {
        render: WidgetRenderCustom::new(move |scene, layout, _widget_styles, dpi| {
            if total <= 0.0 || slices.is_empty() {
                return;
            }

            let dpi = dpi as f64;
            let cx = (layout.location.x + layout.size.x * 0.5) as f64 * dpi;
            let cy = (layout.location.y + layout.size.y * 0.5) as f64 * dpi;
            let outer_radius = (layout.size.x.min(layout.size.y) * 0.5 - 2.0).max(1.0) as f64 * dpi;
            let inner_radius_px = (inner_radius as f64 * dpi).min(outer_radius * 0.95);

            let center = kurbo::Point::new(cx, cy);
            let gap = (styles.gap_deg.to_radians() as f64).max(0.0);
            let mut angle = (styles.start_angle_deg as f64).to_radians();

            for slice in &slices {
                let fraction = (slice.value.max(0.0) as f64) / total as f64;
                let sweep = fraction * std::f64::consts::TAU;
                if sweep > gap {
                    let path = wedge_path(
                        center,
                        inner_radius_px,
                        outer_radius,
                        angle + gap * 0.5,
                        sweep - gap,
                    );
                    scene.fill(
                        peniko::Fill::NonZero,
                        kurbo::Affine::default(),
                        to_peniko(slice.color),
                        None,
                        &path,
                    );
                }
                angle += sweep;
            }
        }),
    }
}
