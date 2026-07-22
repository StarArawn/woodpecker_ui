use crate::prelude::*;
use bevy::prelude::*;

/// Fired when [`Rating`] is clicked (not fired for `read_only: true`, which has no
/// interaction at all).
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct RatingChanged {
    /// The newly-selected value, in half-star increments (e.g. `3.5`).
    pub value: f32,
}

/// [`Rating`]'s themed colors.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct RatingStyles {
    /// Filled star color, showing the committed `value`.
    pub filled_color: Color,
    /// Outline (unfilled) star color.
    pub empty_color: Color,
    /// Filled star color while previewing a hovered value -- a distinct (typically brighter)
    /// color so a hover-preview reads as tentative, not yet committed.
    pub hover_color: Color,
}

impl Default for RatingStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for RatingStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            filled_color: theme.warning,
            empty_color: theme.border,
            hover_color: theme.warning.with_alpha(0.7),
        }
    }
}

#[derive(Component, Default, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
struct RatingHoverState {
    /// The value a hovered half-star region represents, previewed in place of `Rating::value`
    /// while the pointer's over it. `None` when not hovering (or `read_only`).
    hover_value: Option<f32>,
}

/// Computes each star's fill fraction (`0.0`, `0.5`, or `1.0`) for a given effective value
/// (already resolved from either the live hover preview or the committed `Rating::value`).
/// Pulled out as a pure, unit-testable function -- the render body just maps this over
/// `0..max`.
fn star_fraction(star_index: u8, value: f32) -> f32 {
    let value = value.max(0.0);
    if value >= (star_index + 1) as f32 {
        1.0
    } else if value >= star_index as f32 + 0.5 {
        0.5
    } else {
        0.0
    }
}

/// A star rating input -- half-star increments, with a hover preview before committing a
/// click. Half-star granularity comes from splitting each star into two side-by-side
/// half-width hit regions (left = `.5`, right = whole) rather than continuous pointer-position
/// math, and the partial fill itself is a [`Clip`]-masked filled star layered over an outline
/// star, clipped to the appropriate half/full width.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = WoodpeckerStyle { flex_direction: WidgetFlexDirection::Row, ..Default::default() },
    WidgetChildren,
    RatingStyles
)]
pub struct Rating {
    /// The current value, 0..=`max` in half-star increments.
    pub value: f32,
    /// Number of stars.
    pub max: u8,
    /// Disables hover preview and click interaction, leaving a pure display.
    pub read_only: bool,
}

impl Default for Rating {
    fn default() -> Self {
        Self {
            value: 0.0,
            max: 5,
            read_only: false,
        }
    }
}

/// Builds one star's full cell: the outline glyph, a `Clip`-masked filled glyph layered on top
/// (sized to `fraction`), and -- when interactive -- two absolutely-positioned half-width hit
/// regions for hover-preview/click. All positioned relative to this cell's own `position:
/// Relative` root, so percentage-based `left`/`width` on the overlay and hit regions resolve
/// against this one star, not the whole row.
#[allow(clippy::too_many_arguments)]
fn star_cell(
    size: f32,
    fraction: f32,
    empty_color: Color,
    filled_color: Color,
    interactive: bool,
    index: u8,
    current_widget: CurrentWidget,
    state_entity: Entity,
    icon_font: &IconFont,
) -> impl Bundle + Clone {
    let mut cell = WidgetChildren::default();

    cell.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: size.into(),
            height: size.into(),
            font_size: size,
            color: empty_color,
            font: Some(icon_font.0.id()),
            ..Default::default()
        },
        WidgetRender::Text {
            content: icons::STAR.into(),
        },
    ));
    cell.add_key("outline");

    if fraction > 0.0 {
        // Phosphor's bundled set has only one "star" weight (no separate filled glyph) -- the
        // "filled" look comes from clip-masking a *recolored* copy of the same glyph over the
        // outline layer below, not a shape swap.
        cell.add::<Clip>((
            Clip,
            WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                left: 0.0.into(),
                top: 0.0.into(),
                width: Units::Percentage(fraction * 100.0),
                height: Units::Percentage(100.0),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: size.into(),
                    height: size.into(),
                    font_size: size,
                    color: filled_color,
                    font: Some(icon_font.0.id()),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: icons::STAR.into(),
                },
            )),
        ));
        cell.add_key("fill");
    }

    if interactive {
        for (region_key, region_value, left) in [
            ("half", index as f32 + 0.5, 0.0),
            ("whole", index as f32 + 1.0, 50.0),
        ] {
            cell.add::<Element>((
                Element,
                WoodpeckerStyle {
                    position: WidgetPosition::Absolute,
                    left: Units::Percentage(left),
                    top: 0.0.into(),
                    width: Units::Percentage(50.0),
                    height: Units::Percentage(100.0),
                    ..Default::default()
                },
                Pickable::default(),
            ));
            cell.add_key(region_key);
            cell.hover_cursor(current_widget, SystemCursorIcon::Pointer)
                .observe(
                    current_widget,
                    move |_trigger: On<Pointer<Over>>, mut query: Query<&mut RatingHoverState>| {
                        if let Ok(mut state) = query.get_mut(state_entity) {
                            state.hover_value = Some(region_value);
                        }
                    },
                )
                .observe(
                    current_widget,
                    move |_trigger: On<Pointer<Out>>, mut query: Query<&mut RatingHoverState>| {
                        if let Ok(mut state) = query.get_mut(state_entity) {
                            if state.hover_value == Some(region_value) {
                                state.hover_value = None;
                            }
                        }
                    },
                )
                .observe(
                    current_widget,
                    move |_trigger: On<Pointer<Click>>, mut commands: Commands| {
                        commands.trigger(Change {
                            target: current_widget.entity(),
                            data: RatingChanged {
                                value: region_value,
                            },
                        });
                    },
                );
        }
    }

    (
        Element,
        WoodpeckerStyle {
            width: size.into(),
            height: size.into(),
            position: WidgetPosition::Relative,
            ..Default::default()
        },
        cell,
    )
}

fn render(
    theme: Res<Theme>,
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    icon_font: Res<IconFont>,
    mut query: Query<(&Rating, &RatingStyles, &mut WidgetChildren)>,
    state_query: Query<&RatingHoverState>,
) {
    let Ok((rating, rating_styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, RatingHoverState::default());
    let default_state = RatingHoverState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);
    let current_widget = *current_widget;

    let effective_value = state.hover_value.unwrap_or(rating.value);
    let fill_color = if state.hover_value.is_some() {
        rating_styles.hover_color
    } else {
        rating_styles.filled_color
    };
    let star_size = theme.typography.h2;

    *children = WidgetChildren::default();
    for index in 0..rating.max {
        let fraction = star_fraction(index, effective_value);
        children.add::<Element>(star_cell(
            star_size,
            fraction,
            rating_styles.empty_color,
            fill_color,
            !rating.read_only,
            index,
            current_widget,
            state_entity,
            &icon_font,
        ));
        children.add_key(format!("star-{index}"));
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = RatingStyles::from_theme(&theme);
        assert_eq!(styles.filled_color, theme.warning);
        assert_eq!(styles.empty_color, theme.border);

        let dark = RatingStyles::from_theme(&Theme::dark());
        let light = RatingStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.empty_color, light.empty_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn default_rating_is_zero_out_of_five() {
        let rating = Rating::default();
        assert_eq!(rating.value, 0.0);
        assert_eq!(rating.max, 5);
        assert!(!rating.read_only);
    }

    #[test]
    fn star_fraction_is_empty_well_below_its_threshold() {
        assert_eq!(star_fraction(2, 1.0), 0.0);
    }

    #[test]
    fn star_fraction_is_half_at_the_half_threshold() {
        assert_eq!(star_fraction(2, 2.5), 0.5);
    }

    #[test]
    fn star_fraction_is_full_at_the_whole_threshold() {
        assert_eq!(star_fraction(2, 3.0), 1.0);
    }

    #[test]
    fn star_fraction_is_full_for_any_higher_value() {
        assert_eq!(star_fraction(0, 5.0), 1.0);
    }

    #[test]
    fn star_fraction_treats_negative_values_as_zero() {
        assert_eq!(star_fraction(0, -1.0), 0.0);
    }
}
