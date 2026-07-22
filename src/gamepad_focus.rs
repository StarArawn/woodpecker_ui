//! Gamepad/controller focus navigation, behind the opt-in `gamepad-nav` Cargo feature (which
//! pulls in `bevy/gamepad`, the actual hardware-polling backend -- without it these systems
//! never see a connected `Gamepad` component and are harmless no-ops).
//!
//! There's no declared focus-order graph anywhere in this crate to walk sequentially, so
//! navigation is spatial: pressing a D-pad direction moves [`CurrentFocus`] to whichever
//! other [`Focusable`] widget is the best match for that direction, scored by primary-axis
//! distance plus a heavier penalty for cross-axis distance (the standard TV-remote/console
//! "spatial navigation" heuristic).

use bevy::prelude::*;

use crate::focus::{CurrentFocus, Focusable, WidgetActivate, WidgetBlur, WidgetFocus};
use crate::layout::system::WidgetLayout;

/// How much a candidate's off-axis offset counts against it, relative to its on-axis
/// distance. `2.0` is the standard spatial-navigation weighting -- heavily prefers a widget
/// directly ahead over one merely closer but off to the side.
const CROSS_AXIS_PENALTY: f32 = 2.0;

/// Reads D-pad input across all connected gamepads and moves [`CurrentFocus`] to the nearest
/// [`Focusable`] widget in the pressed direction. Runs `.after(CurrentFocus::click_focus)` (see
/// `lib.rs`'s system registration) so a same-frame mouse click always wins over a stale
/// gamepad press.
pub(crate) fn navigate(
    mut commands: Commands,
    mut current_focus: ResMut<CurrentFocus>,
    gamepads: Query<&Gamepad>,
    focusable_query: Query<(Entity, &WidgetLayout), With<Focusable>>,
) {
    let mut direction = None;
    for gamepad in gamepads.iter() {
        direction = if gamepad.just_pressed(GamepadButton::DPadUp) {
            Some(Vec2::new(0.0, -1.0))
        } else if gamepad.just_pressed(GamepadButton::DPadDown) {
            Some(Vec2::new(0.0, 1.0))
        } else if gamepad.just_pressed(GamepadButton::DPadLeft) {
            Some(Vec2::new(-1.0, 0.0))
        } else if gamepad.just_pressed(GamepadButton::DPadRight) {
            Some(Vec2::new(1.0, 0.0))
        } else {
            None
        };
        if direction.is_some() {
            break;
        }
    }
    let Some(direction) = direction else {
        return;
    };

    let current = current_focus.get();
    let current_center = focusable_query
        .get(current)
        .ok()
        .map(|(_, layout)| layout.position() + layout.size / 2.0);

    let mut best: Option<(Entity, f32)> = None;
    for (entity, layout) in focusable_query.iter() {
        if entity == current {
            continue;
        }
        let center = layout.position() + layout.size / 2.0;

        let score = match current_center {
            Some(current_center) => {
                let delta = center - current_center;
                let primary = delta.dot(direction);
                // Only candidates actually ahead in the pressed direction are eligible --
                // without this, the closest widget in *any* direction could win.
                if primary <= 0.0 {
                    continue;
                }
                let cross = (delta - direction * primary).length();
                primary + cross * CROSS_AXIS_PENALTY
            }
            // No current focus to navigate from -- land on a stable, predictable starting
            // point (top-left-most) rather than an arbitrary query-iteration-order pick.
            None => center.y * 1_000_000.0 + center.x,
        };

        if best.is_none_or(|(_, best_score)| score < best_score) {
            best = Some((entity, score));
        }
    }

    if let Some((entity, _)) = best {
        if current != entity && current != Entity::PLACEHOLDER {
            commands.trigger(WidgetBlur { target: current });
        }
        *current_focus = CurrentFocus::new(entity);
        commands.trigger(WidgetFocus { target: entity });
    }
}

/// Fires [`WidgetActivate`] at the focused widget when any connected gamepad's south button
/// (i.e. PS: Cross, Xbox: A) is pressed -- the "confirm" gesture, mirroring a mouse click.
pub(crate) fn activate(
    mut commands: Commands,
    current_focus: Res<CurrentFocus>,
    gamepads: Query<&Gamepad>,
) {
    let focused = current_focus.get();
    if focused == Entity::PLACEHOLDER {
        return;
    }
    if gamepads
        .iter()
        .any(|gamepad| gamepad.just_pressed(GamepadButton::South))
    {
        commands.trigger(WidgetActivate { target: focused });
    }
}
