use crate::prelude::*;
use bevy::prelude::*;

/// A menu item selection event.
#[derive(Reflect, Debug, Clone, PartialEq, Default)]
pub struct MenuItemSelected {
    /// The index of the selected item in `Menu::items`.
    pub index: usize,
    /// The label of the selected item.
    pub label: String,
}

/// A collection of styles for the menu.
#[derive(Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct MenuStyles {
    /// The popup list area styles.
    pub list_area: WoodpeckerStyle,
    /// Each item's button styles.
    pub list_item: ButtonStyles,
    /// Each item's text styles.
    pub text: WoodpeckerStyle,
}

impl Default for MenuStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for MenuStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            list_area: WoodpeckerStyle {
                background_color: theme.background,
                border: Edge::all(1.0),
                border_color: theme.border,
                border_radius: Corner::all(theme.control_radius),
                min_width: 160.0.into(),
                flex_direction: WidgetFlexDirection::Column,
                padding: Edge::all(4.0),
                box_shadow: Some(theme.elevation.md),
                ..Default::default()
            },
            list_item: {
                let normal = WoodpeckerStyle {
                    align_items: Some(WidgetAlignItems::Center),
                    background_color: theme.background,
                    width: Units::Percentage(100.0),
                    height: theme.control_height.into(),
                    font_size: theme.font_size,
                    border_radius: Corner::all(theme.control_radius - 2.0),
                    padding: Edge::all(0.0).left(12.0).right(8.0),
                    ..Default::default()
                };
                ButtonStyles {
                    normal,
                    hovered: WoodpeckerStyle {
                        background_color: theme.background_light,
                        ..normal
                    },
                }
            },
            text: WoodpeckerStyle {
                color: theme.text,
                font_size: theme.font_size,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
        }
    }
}

/// Menu state: is it open, and where (world/viewport space) should it appear -- the position
/// captured from the pointer at the moment it was opened.
#[derive(Component, Default, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct MenuState {
    is_open: bool,
    /// `is_open` as of the last render -- see `DropdownState::previous_open`'s doc comment for
    /// why this is needed (mirrors the identical pattern there).
    previous_open: bool,
    open_position: Vec2,
}

/// The list panel's open/close `Transition` preset -- a quick fade, not yet started.
fn list_transition() -> Transition {
    Transition {
        easing: TransitionEasing::CubicOut,
        timeout: 120.0,
        looping: false,
        playing: false,
        ..Default::default()
    }
}

/// A context menu: wraps a caller-supplied trigger (its `PassedChildren`) and pops up a list
/// of items at the pointer on right-click. Reuses `Dropdown`'s overlay/`StackingContext`/
/// positioned-list-area machinery, but opens at the click position instead of anchored below
/// a persistent trigger, and fires `Change<MenuItemSelected>` (an index/label pair) instead
/// of updating a bound value.
#[derive(Widget, Component, Clone, PartialEq, Reflect, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, PassedChildren, WidgetChildren, MenuStyles, Pickable, Transition = list_transition())]
pub struct Menu {
    /// The items shown in the menu, in order.
    pub items: Vec<String>,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    // `Option`, not a hard `Res`: a sibling of `OverlayRootWidget` in the same initial tree can
    // in principle render before `sync_overlay_root` has observed it -- see that system's doc
    // comment. Skipping a frame beats panicking on a legitimate startup-order race.
    overlay_root: Option<Res<OverlayRoot>>,
    mut query: Query<(
        &Menu,
        &WoodpeckerStyle,
        &MenuStyles,
        &PassedChildren,
        &mut WidgetChildren,
        &mut Transition,
    )>,
    mut state_query: Query<&mut MenuState>,
) {
    let Ok((menu, style, menu_styles, passed_children, mut children, mut transition)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, MenuState::default());
    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    *transition = Transition {
        reversing: !state.is_open,
        timeout: 120.0,
        ..*transition
    };
    if state.previous_open != state.is_open {
        if transition.reversing {
            transition.start_reverse();
        } else {
            transition.start();
        }
        state.previous_open = state.is_open;
    }
    transition.style_a = *style;
    transition.style_b = *style;

    let current_widget = *current_widget;
    let menu_entity = current_widget.0;

    *children = WidgetChildren::default();

    // Trigger -- passes the caller's own content through untouched; only a right-click opens
    // the menu, so a left-click (or any other interaction the caller wired onto their own
    // trigger content) keeps propagating and behaves normally.
    //
    // Uses `Menu`'s own `WoodpeckerStyle` (not `::default()`) so a caller's sizing --
    // `width: 100%` to fill a flex row, for instance -- actually reaches the content it wraps
    // instead of being silently dropped: an `Auto`-width wrapper around a `width: 100%` child
    // has nothing real to hand that percentage down from, so the child would collapse toward
    // its own min-content size regardless of what the caller declared on `Menu` itself.
    children
        .add::<Element>((
            Element,
            *style,
            // Required for this crate's picking backend to hit-test the entity at all --
            // without it, `process_entity` in `picking_backend.rs` skips it entirely and no
            // pointer event (left or right click) ever fires here.
            Pickable::default(),
            passed_children.0.clone(),
        ))
        .observe(
            current_widget,
            move |trigger: On<Pointer<Click>>,
                  mut state_query: Query<&mut MenuState>,
                  pointer_world: PointerWorldPosition| {
                if trigger.button != PointerButton::Secondary {
                    return;
                }
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Some(cursor_pos_world) =
                    pointer_world.convert(trigger.pointer_location.position)
                else {
                    return;
                };
                state.open_position = cursor_pos_world;
                state.is_open = true;
            },
        );
    children.add_key("trigger");

    let mut list_children = WidgetChildren::default();
    for (index, item) in menu.items.iter().enumerate() {
        let label = item.clone();
        list_children.add::<WButton>((
            WButton,
            WidgetChildren::default().with_child::<Element>((
                Element,
                menu_styles.text,
                WidgetRender::Text {
                    content: item.clone(),
                },
            )),
            menu_styles.list_item,
        ));
        list_children.add_key(item.clone());
        list_children.observe(
            current_widget,
            move |mut trigger: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut MenuState>| {
                trigger.propagate(false);
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.is_open = false;
                commands.trigger(Change {
                    target: menu_entity,
                    data: MenuItemSelected {
                        index,
                        label: label.clone(),
                    },
                });
            },
        );
    }
    // Neither the click-catcher nor `list` can portal itself (only whoever *declares* a child
    // can mark it `.portal()`-ed), and both need to escape clipping/z-scoping the same way
    // `Dropdown`'s `list_area` does -- so they're portaled to `OverlayRoot` together, as a
    // single wrapper, so `list` reliably out-ranks the click-catcher on picking regardless of
    // whatever else is portaled elsewhere. See `Dropdown::render`'s matching comment for the
    // full reasoning. Unlike `Dropdown`/`ComboBox`, no positioning math changes here: `left`/
    // `top` are already the captured pointer position in absolute viewport space, not relative
    // to Menu's own box, so `Fixed` already meant the same thing before and after portaling.
    if transition.is_playing() || state.is_open {
        if let Some(overlay_root) = overlay_root {
            let mut overlay_children = WidgetChildren::default();

            // Full-screen click-catcher, added *before* `list` so `list` wins picking
            // priority over it (later siblings win ties via `order`). Closes on
            // `Pointer<Click>` rather than `WidgetBlur` to avoid a race with this crate's
            // picking backend -- matches `Dropdown`.
            overlay_children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    position: WidgetPosition::Fixed,
                    width: Units::Percentage(100.0),
                    height: Units::Percentage(100.0),
                    ..Default::default()
                },
                Pickable::default(),
            ));
            overlay_children
                .observe(current_widget, |mut trigger: On<Pointer<Over>>| {
                    trigger.propagate(false);
                })
                .observe(current_widget, |mut trigger: On<Pointer<Out>>| {
                    trigger.propagate(false);
                })
                .observe(
                    current_widget,
                    move |mut trigger: On<Pointer<Click>>,
                          mut state_query: Query<&mut MenuState>| {
                        trigger.propagate(false);
                        let Ok(mut state) = state_query.get_mut(state_entity) else {
                            return;
                        };
                        state.is_open = false;
                    },
                );
            overlay_children.add_key("overlay");

            let list_style = WoodpeckerStyle {
                position: WidgetPosition::Fixed,
                left: state.open_position.x.into(),
                top: state.open_position.y.into(),
                opacity: 1.0,
                // No explicit `z_index` needed here: `list` and `overlay` are true
                // siblings under `overlay_wrapper` now, and `list` is declared *after*
                // `overlay`, so plain DOM-order tiebreaking within the `Auto` bucket
                // already ranks it above the click-catcher (see
                // `layout::system::order_children_for_paint`). The crate-wide
                // `StackingTier::Dropdown` value belongs on `overlay_wrapper` itself
                // instead (below), which is the entity that's actually portaled and so
                // the one that needs to rank correctly against *other* portaled overlays
                // (an open `Popover`/`Toast`, etc).
                ..menu_styles.list_area
            };
            const SLIDE_OFFSET: f32 = 6.0;
            let list_transition = Transition {
                style_a: WoodpeckerStyle {
                    opacity: 0.0,
                    top: (state.open_position.y - SLIDE_OFFSET).into(),
                    ..list_style
                },
                style_b: list_style,
                ..*transition
            };
            overlay_children.add::<Element>((
                Element,
                list_style,
                list_children,
                WidgetRender::Quad,
                list_transition,
                // Same reasoning as `Dropdown`/`Modal`: without this, the open list renders
                // on top but doesn't win picking priority at the ambient depth.
                StackingContext,
            ));
            overlay_children.add_key("list");

            children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    // Ranks `overlay_wrapper` (the entity actually portaled to `OverlayRoot`)
                    // against whatever else is portaled there -- an open `Popover`/`Tooltip`/
                    // `Toast`, etc. See the comment on `list`'s own style above for why this
                    // moved here instead of living on `list` directly.
                    z_index: Some(WidgetZ::Global(StackingTier::Dropdown as u32)),
                    ..Default::default()
                },
                overlay_children,
            ));
            children.add_key("overlay_wrapper");
            children.portal_to(overlay_root.0);
        }
    }

    children.apply(current_widget.as_parent());
}
