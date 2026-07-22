use crate::prelude::*;
use bevy::prelude::*;
use bevy_vello::vello::kurbo::{self, Shape};
use std::f64::consts::PI;
use web_time::Instant;

/// A collection of styles for [`Spinner`].
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct SpinnerStyles {
    /// The stroke color.
    pub color: Color,
    /// The background track color, drawn as a full circle behind the progress arc. Only used
    /// by [`SpinnerMode::Determinate`] -- [`SpinnerMode::Indeterminate`] has no "remaining"
    /// portion to contextualize, so it draws no track (unchanged from before this field
    /// existed, to avoid a visual regression for every existing indeterminate call site).
    pub track_color: Color,
    /// The stroke width, in logical pixels.
    pub line_width: f32,
    /// How much of the circle the spinning arc covers, in degrees (e.g. 90.0 = a
    /// quarter-circle arc chasing itself around). Only used by [`SpinnerMode::Indeterminate`].
    pub sweep_degrees: f32,
    /// Full rotations per second. Only used by [`SpinnerMode::Indeterminate`].
    pub speed: f32,
}

impl Default for SpinnerStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for SpinnerStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            color: theme.primary,
            track_color: theme.background_light,
            line_width: 3.0,
            sweep_degrees: 90.0,
            speed: 1.2,
        }
    }
}

/// [`Spinner`]'s two behaviors.
#[derive(Reflect, Clone, Copy, PartialEq, Default, Debug)]
pub enum SpinnerMode {
    /// A rotating arc animated continuously from wall-clock time (MUI's own default
    /// `CircularProgress` behavior) -- unknown-duration work.
    #[default]
    Indeterminate,
    /// A fixed arc sweeping clockwise from the top, proportional to `value` (clamped 0..=1),
    /// drawn over a full-circle background track -- known-progress work (a download, an
    /// upload, a multi-step job). MUI's own determinate `CircularProgress` is a *stroked
    /// ring* in exactly this shape, not a filled pie slice -- despite "arc fill" reading like
    /// a wedge at a glance, matching that (and this widget's own existing indeterminate ring
    /// language) is both the authentic design and the simpler implementation, so that's what
    /// this renders. A closed, `Scene::fill`-able wedge `BezPath` (center -> arc start ->
    /// [arc's own curve segments] -> center) is still the right building block if a literal
    /// pie-chart-style widget is ever needed elsewhere -- just not required here.
    Determinate(f32),
}

/// A loading/progress indicator -- a circular arc, either spinning indefinitely
/// ([`SpinnerMode::Indeterminate`]) or showing a known fraction complete
/// ([`SpinnerMode::Determinate`]). Rendered inside its own [`WidgetRender::Custom`] closure
/// rather than the normal prop-diffing re-render path, since indeterminate mode needs to read
/// wall-clock time every frame rather than a diffed value -- `render` (below) keeps [`Animating`]
/// in sync with `mode` so the vello render traversal keeps running every frame while this spinner
/// is indeterminate, rather than being skipped by the frame-skip cache.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_style(), WidgetChildren, SpinnerStyles)]
pub struct Spinner {
    /// Diameter, in logical pixels.
    pub size: f32,
    /// Indeterminate (default) or determinate -- see [`SpinnerMode`].
    pub mode: SpinnerMode,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            size: 24.0,
            mode: SpinnerMode::default(),
        }
    }
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: 24.0.into(),
        height: 24.0.into(),
        ..Default::default()
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut commands: Commands,
    mut query: Query<(
        &Spinner,
        &SpinnerStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
) {
    let Ok((spinner, spinner_styles, mut styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    styles.width = spinner.size.into();
    styles.height = spinner.size.into();

    match spinner.mode {
        SpinnerMode::Indeterminate => {
            commands.entity(**current_widget).insert(Animating);
        }
        SpinnerMode::Determinate(_) => {
            commands.entity(**current_widget).remove::<Animating>();
        }
    }

    *children = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            ..Default::default()
        },
        build_spinner_render(*spinner_styles, spinner.mode, Instant::now()),
    ));
    children.add_key("arc");

    children.apply(current_widget.as_parent());
}

fn vello_color(color: Color) -> bevy_vello::vello::peniko::Color {
    let srgba = color.to_srgba();
    bevy_vello::vello::peniko::Color::new([srgba.red, srgba.green, srgba.blue, srgba.alpha])
}

fn build_spinner_render(styles: SpinnerStyles, mode: SpinnerMode, start: Instant) -> WidgetRender {
    WidgetRender::Custom {
        render: WidgetRenderCustom::new(move |scene, layout, _widget_styles, dpi| {
            let dpi = dpi as f64;
            let cx = (layout.location.x + layout.size.x / 2.0) as f64 * dpi;
            let cy = (layout.location.y + layout.size.y / 2.0) as f64 * dpi;
            let radius = (layout.size.x.min(layout.size.y) / 2.0) as f64 * dpi
                - (styles.line_width as f64 * dpi) / 2.0;
            if radius <= 0.0 {
                return;
            }

            let center = kurbo::Point::new(cx, cy);
            let stroke =
                kurbo::Stroke::new(styles.line_width as f64 * dpi).with_caps(kurbo::Cap::Round);

            // Both branches converge on a single (start_angle, sweep) pair, drawn identically
            // below -- only how that pair is derived differs.
            let (start_angle, sweep) = match mode {
                SpinnerMode::Indeterminate => {
                    let rotations = start.elapsed().as_secs_f64() * styles.speed as f64;
                    (
                        (rotations % 1.0) * 2.0 * PI,
                        (styles.sweep_degrees as f64).to_radians(),
                    )
                }
                SpinnerMode::Determinate(value) => {
                    scene.stroke(
                        &stroke,
                        kurbo::Affine::default(),
                        vello_color(styles.track_color),
                        None,
                        &kurbo::Circle::new(center, radius),
                    );
                    determinate_angles(value)
                }
            };

            if sweep <= 0.0 {
                return;
            }

            let arc = kurbo::Arc::new(
                center,
                kurbo::Vec2::new(radius, radius),
                start_angle,
                sweep,
                0.0,
            );
            let mut path = kurbo::BezPath::new();
            path.extend(arc.path_elements(0.1));

            scene.stroke(
                &stroke,
                kurbo::Affine::default(),
                vello_color(styles.color),
                None,
                &path,
            );
        }),
    }
}

/// `(start_angle, sweep)` for [`SpinnerMode::Determinate`], in radians. `-PI/2` starts at the
/// top; kurbo's angle convention increases clockwise on screen (y grows downward), matching a
/// typical progress ring's direction. Pulled out of the render closure so the clamping and
/// angle math is unit-testable without a live vello scene.
fn determinate_angles(value: f32) -> (f64, f64) {
    (-PI / 2.0, value.clamp(0.0, 1.0) as f64 * 2.0 * PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_defaults_to_indeterminate_mode() {
        assert_eq!(Spinner::default().mode, SpinnerMode::Indeterminate);
        assert_eq!(SpinnerMode::default(), SpinnerMode::Indeterminate);
    }

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = SpinnerStyles::from_theme(&theme);
        assert_eq!(styles.color, theme.primary);
        assert_eq!(styles.track_color, theme.background_light);

        // `primary` is deliberately the same brand accent in both themes (see
        // `Theme::dark`/`Theme::light`), so assert on `track_color` (which does differ) to
        // confirm `from_theme` genuinely reads from its argument rather than a fixed default.
        let dark = SpinnerStyles::from_theme(&Theme::dark());
        let light = SpinnerStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.track_color, light.track_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn determinate_angles_start_at_the_top() {
        let (start_angle, _) = determinate_angles(0.5);
        assert_eq!(start_angle, -PI / 2.0);
    }

    #[test]
    fn determinate_angles_sweep_is_proportional_to_value() {
        let (_, zero) = determinate_angles(0.0);
        assert_eq!(zero, 0.0);

        let (_, quarter) = determinate_angles(0.25);
        assert!((quarter - PI / 2.0).abs() < 1e-6);

        let (_, full) = determinate_angles(1.0);
        assert!((full - 2.0 * PI).abs() < 1e-6);
    }

    #[test]
    fn determinate_angles_clamps_out_of_range_values() {
        let (_, below) = determinate_angles(-1.0);
        assert_eq!(below, 0.0);

        let (_, above) = determinate_angles(2.0);
        assert!((above - 2.0 * PI).abs() < 1e-6);
    }
}
