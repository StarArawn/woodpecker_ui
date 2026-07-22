//! Keyboard Tab-order navigation, plus modal focus trapping via [`FocusTrapRoot`].
//!
//! Unlike `gamepad_focus` (which navigates spatially, since there's no declared order to walk),
//! Tab/Shift+Tab follow the same `Children` hierarchy the rest of the crate already treats as
//! DOM order, collecting every [`Focusable`](crate::focus::Focusable) descendant not hidden
//! behind a `WidgetDisplay::None` ancestor. The order is recomputed fresh on every Tab press
//! (a rare, user-driven event) rather than cached, so it can never go stale the way a
//! maintained index would.

use bevy::prelude::*;

use crate::{
    focus::{CurrentFocus, Focusable, WidgetBlur, WidgetFocus},
    hook_helper::HookHelper,
    layout::system::WidgetLayout,
    prelude::{TextBoxState, WidgetDisplay, WoodpeckerStyle},
    CurrentWidget, WoodpeckerContext,
};

/// Marks the root of a modal-style overlay's content so Tab/Shift+Tab navigation stays
/// confined to it while it's on screen, instead of leaking focus out to whatever's behind it.
/// [`Modal`](crate::widgets::Modal) attaches this to its own "window" child automatically;
/// hand-rolled overlays can attach it too.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct FocusTrapRoot;

/// Depth-first, `Children`-order walk starting at `root`, collecting every `Focusable`
/// descendant. An ancestor styled `WidgetDisplay::None` prunes its entire subtree -- an
/// invisible widget (and anything nested inside it) is never a valid Tab stop.
pub(crate) fn compute_tab_order(
    root: Entity,
    children_query: &Query<&Children>,
    focusable_query: &Query<(), With<Focusable>>,
    style_query: &Query<&WoodpeckerStyle>,
) -> Vec<Entity> {
    let mut order = Vec::new();
    walk_tab_order(
        root,
        children_query,
        focusable_query,
        style_query,
        &mut order,
    );
    order
}

fn walk_tab_order(
    entity: Entity,
    children_query: &Query<&Children>,
    focusable_query: &Query<(), With<Focusable>>,
    style_query: &Query<&WoodpeckerStyle>,
    order: &mut Vec<Entity>,
) {
    if style_query
        .get(entity)
        .is_ok_and(|style| style.display == WidgetDisplay::None)
    {
        return;
    }

    if focusable_query.contains(entity) {
        order.push(entity);
    }

    let Ok(children) = children_query.get(entity) else {
        return;
    };
    for child in children.iter() {
        walk_tab_order(child, children_query, focusable_query, style_query, order);
    }
}

/// Finds the topmost live [`FocusTrapRoot`] -- the one with the highest layout `order` (see
/// `WidgetLayout`'s doc comment: higher `order` renders on top) -- in case more than one
/// happens to be on screen at once (nested modals). Returns `None` when no trap is active, so
/// callers fall back to the real UI root.
fn topmost_focus_trap(
    trap_query: &Query<(Entity, &WidgetLayout), With<FocusTrapRoot>>,
) -> Option<Entity> {
    trap_query
        .iter()
        .max_by_key(|(_, layout)| layout.order)
        .map(|(entity, _)| entity)
}

/// Guards on `Tab`/`Shift+Tab`, moving [`CurrentFocus`] to the next/previous `Focusable`
/// widget in DOM order. Runs `.after(keyboard_input::runner)` in the same `Update` group as
/// `CurrentFocus::click_focus` (see `lib.rs`'s system registration).
///
/// Plain `Tab` skips entirely when the current focus is a multi-line `TextBox` -- its own
/// keyboard handler (`text_box.rs`'s `textbox_handle_keyboard_events`) treats Tab as an indent
/// keystroke in that one case, so Tab navigation must not also fight it for the same keypress.
/// Three escape hatches bypass that carve-out, all landing on the *next* stop (`Escape` has no
/// "reverse" reading, so unlike the other two it's forward-only):
/// - `Shift+Tab` -- the same convention a plain HTML `<textarea>` uses (Shift+Tab always moves
///   focus, even where Tab itself is special-cased).
/// - `Ctrl+Tab` (or `Cmd+Tab` on macOS) -- an explicit, always-available "move focus" gesture,
///   the same escape VS Code's own "Toggle Tab Key Moves Focus" accessibility command exists
///   for.
/// - `Escape` -- but *only* while trapped in a multi-line `TextBox`; for every other focused
///   widget it's left alone, since some (`ComboBox`, `DatePicker`) already use Escape for
///   their own "close the popup" behavior, not focus movement.
///
/// Without at least one modifier-free-Tab-independent way out, a multi-line box would be a
/// genuine keyboard trap (WCAG 2.1.2) -- these three are deliberately redundant with each
/// other so there's always an obvious one to reach for.
pub(crate) fn tab_navigate(
    mut commands: Commands,
    mut current_focus: ResMut<CurrentFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    context: Res<WoodpeckerContext>,
    hooks: Res<HookHelper>,
    children_query: Query<&Children>,
    focusable_query: Query<(), With<Focusable>>,
    style_query: Query<&WoodpeckerStyle>,
    trap_query: Query<(Entity, &WidgetLayout), With<FocusTrapRoot>>,
    text_box_state_query: Query<&TextBoxState>,
) {
    let tab_pressed = keyboard_input.just_pressed(KeyCode::Tab);
    let escape_pressed = keyboard_input.just_pressed(KeyCode::Escape);
    if !tab_pressed && !escape_pressed {
        return;
    }

    let current = current_focus.get();
    // `TextBoxState` lives on a separate `hooks.use_state`-spawned child entity, not on the
    // focused `TextBox` entity itself (`current`) -- querying `current` directly always missed,
    // so Tab always moved focus out of a multi-line box instead of ever reaching
    // `textbox_handle_keyboard_events`'s own indent-insert handling.
    let is_multiline_text_box = hooks
        .get_state::<TextBoxState>(CurrentWidget(current))
        .and_then(|state_entity| text_box_state_query.get(state_entity).ok())
        .is_some_and(|state| state.multi_line);

    if escape_pressed && !is_multiline_text_box {
        return;
    }

    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight)
        || keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight);
    let reverse = tab_pressed
        && (keyboard_input.pressed(KeyCode::ShiftLeft)
            || keyboard_input.pressed(KeyCode::ShiftRight));

    if tab_pressed && !ctrl_pressed && !reverse && is_multiline_text_box {
        return;
    }

    let scope_root = topmost_focus_trap(&trap_query).unwrap_or_else(|| context.get_root_widget());
    let order = compute_tab_order(scope_root, &children_query, &focusable_query, &style_query);
    let Some((first, last)) = order.first().copied().zip(order.last().copied()) else {
        return;
    };

    let next = match order.iter().position(|&e| e == current) {
        Some(index) => {
            let len = order.len();
            if reverse {
                order[(index + len - 1) % len]
            } else {
                order[(index + 1) % len]
            }
        }
        // No current focus, or focus lives outside this trap's/tree's Focusable order (e.g.
        // set programmatically, or a modal just opened) -- land on the first/last stop rather
        // than doing nothing.
        None => {
            if reverse {
                last
            } else {
                first
            }
        }
    };

    if next == current {
        return;
    }
    if current != Entity::PLACEHOLDER {
        commands.trigger(WidgetBlur { target: current });
    }
    *current_focus = CurrentFocus::new(next);
    commands.trigger(WidgetFocus { target: next });
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    fn order_from(world: &mut World, root: Entity) -> Vec<Entity> {
        world
            .run_system_once(
                move |children_query: Query<&Children>,
                      focusable_query: Query<(), With<Focusable>>,
                      style_query: Query<&WoodpeckerStyle>| {
                    compute_tab_order(root, &children_query, &focusable_query, &style_query)
                },
            )
            .unwrap()
    }

    #[test]
    fn empty_tree_has_no_tab_stops() {
        let mut world = World::new();
        let root = world.spawn_empty().id();
        assert_eq!(order_from(&mut world, root), Vec::<Entity>::new());
    }

    #[test]
    fn single_focusable_item_is_its_own_order() {
        let mut world = World::new();
        let root = world.spawn(Focusable).id();
        assert_eq!(order_from(&mut world, root), vec![root]);
    }

    #[test]
    fn nested_tree_walks_depth_first_in_child_order() {
        let mut world = World::new();
        let a = world.spawn(Focusable).id();
        // `b` isn't itself focusable, but its focusable descendant should still surface.
        let b = world.spawn_empty().id();
        let c = world.spawn(Focusable).id();
        let root = world.spawn_empty().id();
        world.entity_mut(a).insert(ChildOf(root));
        world.entity_mut(b).insert(ChildOf(root));
        world.entity_mut(c).insert(ChildOf(b));

        assert_eq!(order_from(&mut world, root), vec![a, c]);
    }

    #[test]
    fn display_none_ancestor_prunes_its_whole_subtree() {
        let mut world = World::new();
        let hidden_child = world.spawn(Focusable).id();
        let hidden = world
            .spawn(WoodpeckerStyle {
                display: WidgetDisplay::None,
                ..Default::default()
            })
            .id();
        let visible = world.spawn(Focusable).id();
        let root = world.spawn_empty().id();
        world.entity_mut(hidden).insert(ChildOf(root));
        world.entity_mut(visible).insert(ChildOf(root));
        world.entity_mut(hidden_child).insert(ChildOf(hidden));

        assert_eq!(order_from(&mut world, root), vec![visible]);
    }

    #[test]
    fn modal_scope_restriction_only_walks_the_given_subtree() {
        let mut world = World::new();
        let outside = world.spawn(Focusable).id();
        let trap_root = world.spawn_empty().id();
        let inside = world.spawn(Focusable).id();
        let root = world.spawn_empty().id();
        world.entity_mut(outside).insert(ChildOf(root));
        world.entity_mut(trap_root).insert(ChildOf(root));
        world.entity_mut(inside).insert(ChildOf(trap_root));

        // Full-tree order sees both...
        assert_eq!(order_from(&mut world, root), vec![outside, inside]);
        // ...but scoping to the trap root (as `tab_navigate` does while it's the topmost live
        // `FocusTrapRoot`) only sees what's inside it.
        assert_eq!(order_from(&mut world, trap_root), vec![inside]);
    }

    /// Builds a minimal tree with two `Focusable` stops (`text_box`, then `other`), registers
    /// `text_box`'s `TextBoxState` the *real* way (via `hooks.use_state`, which spawns it on a
    /// separate child entity rather than `text_box` itself -- exactly the distinction
    /// `tab_navigate`'s own doc comment calls out as the bug this guards against), focuses
    /// `text_box`, presses every key in `pressed_keys`, and runs `tab_navigate` once. Returns
    /// the `World` (so the caller can inspect `CurrentFocus` afterward) plus both entities.
    fn run_tab_navigate_with_focused_text_box(
        multi_line: bool,
        pressed_keys: &[KeyCode],
    ) -> (World, Entity, Entity) {
        let mut world = World::new();
        world.init_resource::<HookHelper>();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.init_resource::<WoodpeckerContext>();

        let text_box = world.spawn(Focusable).id();
        let other = world.spawn(Focusable).id();
        let root = world.spawn_empty().id();
        world.entity_mut(text_box).insert(ChildOf(root));
        world.entity_mut(other).insert(ChildOf(root));
        world
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root);
        world.insert_resource(CurrentFocus::new(text_box));
        {
            let mut input = world.resource_mut::<ButtonInput<KeyCode>>();
            for &key in pressed_keys {
                input.press(key);
            }
        }

        world
            .run_system_once(
                move |mut hooks: ResMut<HookHelper>, mut commands: Commands| {
                    hooks.use_state(
                        &mut commands,
                        CurrentWidget(text_box),
                        TextBoxState {
                            multi_line,
                            ..Default::default()
                        },
                    );
                },
            )
            .unwrap();

        world.run_system_once(tab_navigate).unwrap();

        (world, text_box, other)
    }

    #[test]
    fn multiline_text_box_focus_blocks_tab_from_moving_focus() {
        let (world, text_box, _other) =
            run_tab_navigate_with_focused_text_box(true, &[KeyCode::Tab]);

        // A regression here means `tab_navigate` looked `TextBoxState` up on the wrong entity,
        // always missed, and moved focus away instead of leaving Tab for
        // `textbox_handle_keyboard_events`'s own indent-insert handling.
        assert_eq!(
            world.resource::<CurrentFocus>().get(),
            text_box,
            "Tab should not move focus out of a focused multi-line TextBox"
        );
    }

    #[test]
    fn single_line_text_box_focus_still_lets_tab_move_focus() {
        let (world, _text_box, other) =
            run_tab_navigate_with_focused_text_box(false, &[KeyCode::Tab]);

        assert_eq!(
            world.resource::<CurrentFocus>().get(),
            other,
            "Tab should move focus normally out of a single-line TextBox"
        );
    }

    #[test]
    fn shift_tab_still_escapes_a_multiline_text_box() {
        let (world, _text_box, other) =
            run_tab_navigate_with_focused_text_box(true, &[KeyCode::Tab, KeyCode::ShiftLeft]);

        // Otherwise a multi-line box would be a keyboard dead end -- Tab always inserts, and
        // without this exemption nothing would ever move focus back out.
        assert_eq!(
            world.resource::<CurrentFocus>().get(),
            other,
            "Shift+Tab should still move focus out of a multi-line TextBox"
        );
    }

    #[test]
    fn ctrl_tab_also_escapes_a_multiline_text_box() {
        let (world, _text_box, other) =
            run_tab_navigate_with_focused_text_box(true, &[KeyCode::Tab, KeyCode::ControlLeft]);

        assert_eq!(
            world.resource::<CurrentFocus>().get(),
            other,
            "Ctrl+Tab should move focus out of a multi-line TextBox, same as VS Code's own \
             accessibility escape hatch for a Tab-traps-focus editor"
        );
    }

    #[test]
    fn escape_also_escapes_a_multiline_text_box() {
        let (world, _text_box, other) =
            run_tab_navigate_with_focused_text_box(true, &[KeyCode::Escape]);

        assert_eq!(
            world.resource::<CurrentFocus>().get(),
            other,
            "Escape should move focus out of a multi-line TextBox"
        );
    }

    #[test]
    fn escape_does_nothing_when_not_trapped_in_a_multiline_text_box() {
        let (world, text_box, _other) =
            run_tab_navigate_with_focused_text_box(false, &[KeyCode::Escape]);

        // Escape is only a focus-navigation escape hatch for the specific case that needs one
        // -- for every other focused widget it must be left alone, since some (`ComboBox`,
        // `DatePicker`) already bind Escape to their own "close the popup" behavior rather
        // than moving focus.
        assert_eq!(
            world.resource::<CurrentFocus>().get(),
            text_box,
            "Escape should not move focus away from a non-multi-line-TextBox widget"
        );
    }
}
