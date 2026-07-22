use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use woodpecker_ui::prelude::*;

use crate::mock_data::*;
use crate::theme;

/// Global z-index for a dragged `ItemSlot` ghost -- above every window/panel, but below
/// `Popover`(2000)/`Tooltip`(3000)/`ToastViewport`(5000) so those still read on top of a drag.
const DRAG_Z: u32 = 1500;

const SLOT_SIZE: f32 = 56.0;

/// Marks the entity being dragged as carrying a specific inventory item, so a drop target's
/// `droppable` handlers can look up `trigger.dragged`/`trigger.dropped`'s item with a plain
/// query. Attached permanently as part of the popover's own bundle below (not inserted
/// reactively at drag start) -- `droppable` needs it present the instant a drag reaches a
/// drop target, however early in the drag that happens. Carries `item_type` alongside the id
/// so `droppable`'s `accept` closure (which runs with no system-param/resource access) can
/// validate the drop from the payload alone, without an `Inventory` lookup.
#[derive(Component, Clone)]
pub(crate) struct DraggableItem {
    pub id: u64,
    pub item_type: String,
}

/// Per-`ItemSlot` state: hover (drives its nested `Popover`'s `visible`, since `Popover` is
/// a controlled component and `Tooltip::text` can't show name + rarity + stat lines together)
/// and drag (whether this slot is being dragged, and where to draw its ghost).
#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ItemSlotState {
    hovering: bool,
    dragging: bool,
    /// World-space position the ghost should render at -- already anchored to the slot's
    /// on-screen position at drag start by `drag_state`, so it doesn't visually snap under
    /// the cursor when the drag begins.
    drag_position: Vec2,
}

/// A single inventory slot. Watches `Inventory` directly to look itself up by id; `selected`
/// is a plain prop set by `InventoryGrid` (which already watches `SelectedItem`).
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(item_slot_render)]
#[require(WidgetChildren, WoodpeckerStyle, WatchedResource<Inventory>)]
pub struct ItemSlot {
    pub item_id: u64,
    pub selected: bool,
}

fn item_slot_render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    overlay_root: Option<Res<OverlayRoot>>,
    mut query: Query<(&ItemSlot, &WatchedResource<Inventory>, &mut WidgetChildren)>,
    state_query: Query<&ItemSlotState>,
) {
    let Ok((slot, inventory, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let Some(item) = inventory.0.iter().find(|i| i.id == slot.item_id) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, ItemSlotState::default());
    let current_widget = *current_widget;
    let default_state = ItemSlotState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    let (border_width, border_color) = if slot.selected {
        (3.0, Color::WHITE)
    } else {
        (2.0, theme::rarity_color(item.rarity))
    };

    let trigger = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 20.0,
                color: item.icon_color,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: item.glyph.to_string(),
            },
        )),
    ));

    let mut content = WidgetChildren::default();
    content.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 14.0,
            color: theme::TEXT_PRIMARY,
            text_wrap: TextWrap::None,
            margin: Edge::all(0.0).bottom(theme::SPACE_XS),
            ..Default::default()
        },
        WidgetRender::Text {
            content: item.name.clone(),
        },
    ));
    content.add_key("name");
    content.add::<Badge>((Badge {
        label: item.rarity.label().into(),
        variant: item.rarity.badge_variant(),
    },));
    content.add_key("rarity");
    for (i, line) in item.stat_lines.iter().enumerate() {
        content.add::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 12.0,
                color: theme::TEXT_MUTED,
                text_wrap: TextWrap::None,
                margin: Edge::all(0.0).top(theme::SPACE_XS),
                ..Default::default()
            },
            WidgetRender::Text {
                content: line.clone(),
            },
        ));
        content.add_key(format!("stat{i}"));
    }

    let item_id = slot.item_id;

    // While dragged, the slot itself is hidden (reserving no space in the parent's layout --
    // the grid closes the gap during a drag) rather than despawned: its entity has to survive
    // for the rest of the drag gesture, since that's what's actually receiving the
    // `Pointer<Drag>`/`DragEnd` events below. The visible drag ghost is a separate, portaled
    // child (see `.with_drag_ghost` below) -- `display: None` alone can't escape the
    // `WoodpeckerWindow`'s `Clip`-masked interior this slot is nested in, so restyling this
    // entity in place (the previous approach) left the dragged item clipped to the window.
    let popover_style = if state.dragging {
        WoodpeckerStyle {
            display: WidgetDisplay::None,
            ..Default::default()
        }
    } else {
        WoodpeckerStyle {
            width: SLOT_SIZE.into(),
            height: SLOT_SIZE.into(),
            background_color: theme::BG_INPUT,
            border: Edge::all(border_width),
            border_color,
            border_radius: Corner::all(theme::RADIUS_SM),
            ..Default::default()
        }
    };

    *children = WidgetChildren::default()
        .with_child::<Popover>((
            Popover {
                // Suppress the hover tooltip while dragging, or it can flash mid-drag as
                // the slot hides itself.
                visible: state.hovering && !state.dragging,
                placement: PopoverPlacement::Right,
            },
            WidgetRender::Quad,
            Pickable::default(),
            popover_style,
            PassedChildren(trigger.clone()),
            PopoverContent(content),
            DraggableItem {
                id: item_id,
                item_type: item.item_type.clone(),
            },
        ))
        .with_key("popover")
        .with_hover_state(
            current_widget,
            state_entity,
            SystemCursorIcon::Pointer,
            |state: &mut ItemSlotState, hovering| state.hovering = hovering,
        )
        .with_observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut selected: ResMut<SelectedItem>| {
                selected.0 = Some(item_id);
            },
        )
        .with_drag_state(
            current_widget,
            state_entity,
            |state: &mut ItemSlotState, phase| match phase {
                DragPhase::Start(pos) | DragPhase::Move(pos) => {
                    state.dragging = true;
                    state.drag_position = pos;
                }
                DragPhase::End => state.dragging = false,
            },
        )
        .with_drag_ghost(
            overlay_root.as_deref().copied(),
            state.dragging,
            (
                Element,
                WoodpeckerStyle {
                    width: SLOT_SIZE.into(),
                    height: SLOT_SIZE.into(),
                    background_color: theme::BG_INPUT,
                    border: Edge::all(border_width),
                    border_color,
                    border_radius: Corner::all(theme::RADIUS_SM),
                    position: WidgetPosition::Fixed,
                    left: state.drag_position.x.into(),
                    top: state.drag_position.y.into(),
                    z_index: Some(WidgetZ::Global(DRAG_Z)),
                    opacity: 0.85,
                    ..Default::default()
                },
                WidgetRender::Quad,
                // The ghost floats over the pointer and would otherwise block its own drop
                // target's `DragEnter`/`DragOver`/`DragDrop` (`Pickable::default()` blocks
                // picking underneath it). `IGNORE` lets pointer events pass through to the
                // slot beneath.
                Pickable::IGNORE,
                trigger,
            ),
        );

    children.apply(current_widget.as_parent());
}

fn equipment_slot_for(item_type: &str) -> Option<fn(&mut Equipment) -> &mut Option<u64>> {
    match item_type {
        "Weapon" => Some(|e| &mut e.weapon),
        "Armor" => Some(|e| &mut e.armor),
        "Helm" => Some(|e| &mut e.helm),
        "Boots" => Some(|e| &mut e.boots),
        "Ring" => Some(|e| &mut e.ring1),
        _ => None,
    }
}

fn detail_pane(
    current_widget: CurrentWidget,
    inventory: &Inventory,
    selected: &SelectedItem,
    equipment: &Equipment,
) -> impl Bundle + Clone {
    let item = selected
        .0
        .and_then(|id| inventory.0.iter().find(|i| i.id == id));

    let mut children = WidgetChildren::default();
    match item {
        None => {
            children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: theme::TEXT_MUTED,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Select an item to see its details.".into(),
                },
            ));
            children.add_key("placeholder");
        }
        Some(item) => {
            children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 16.0,
                    color: theme::TEXT_PRIMARY,
                    text_wrap: TextWrap::None,
                    margin: Edge::all(0.0).bottom(theme::SPACE_XS),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: item.name.clone(),
                },
            ));
            children.add_key("name");

            let mut stat_row = WidgetChildren::default();
            stat_row.add::<Badge>((Badge {
                label: item.rarity.label().into(),
                variant: item.rarity.badge_variant(),
            },));
            stat_row.add_key("rarity");
            stat_row.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 12.0,
                    color: theme::TEXT_MUTED,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: item.item_type.clone(),
                },
            ));
            stat_row.add_key("type");
            children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_direction: WidgetFlexDirection::Row,
                    align_items: Some(WidgetAlignItems::Center),
                    gap: (8.0.into(), 0.0.into()),
                    margin: Edge::all(0.0).bottom(theme::SPACE_SM),
                    ..Default::default()
                },
                stat_row,
            ));
            children.add_key("badges");

            for (i, line) in item.stat_lines.iter().enumerate() {
                children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 12.0,
                        color: theme::TEXT_MUTED,
                        text_wrap: TextWrap::None,
                        margin: Edge::all(0.0).bottom(2.0),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: line.clone(),
                    },
                ));
                children.add_key(format!("stat{i}"));
            }

            if let Some(slot_fn) = equipment_slot_for(&item.item_type) {
                let mut equipment_copy = *equipment;
                let currently_equipped = *slot_fn(&mut equipment_copy) == Some(item.id);
                let item_id = item.id;
                children.add::<WButton>((
                    WButton,
                    theme::primary_button_styles(),
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            color: Color::WHITE,
                            text_wrap: TextWrap::None,
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: if currently_equipped {
                                "Unequip".into()
                            } else {
                                "Equip".into()
                            },
                        },
                    )),
                ));
                children.add_key("equip");
                children.observe(
                    current_widget,
                    move |_trigger: On<Pointer<Click>>, mut equipment: ResMut<Equipment>| {
                        let slot = slot_fn(&mut equipment);
                        if *slot == Some(item_id) {
                            *slot = None;
                        } else {
                            *slot = Some(item_id);
                        }
                    },
                );
            }
        }
    }

    (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            margin: Edge::all(0.0).top(theme::SPACE_SM),
            padding: Edge::all(theme::SPACE_SM),
            background_color: theme::BG_INPUT,
            border_radius: Corner::all(theme::RADIUS_SM),
            ..Default::default()
        },
        WidgetRender::Quad,
        children,
    )
}

/// Watches `Inventory`, `SelectedItem`, and `Equipment` -- a wrapped grid of `ItemSlot`s
/// plus a detail pane for the current selection.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(inventory_grid_render)]
#[require(
    WidgetChildren,
    WoodpeckerStyle = default_style(),
    WatchedResource<Inventory>,
    WatchedResource<SelectedItem>,
    WatchedResource<Equipment>
)]
pub struct InventoryGrid;

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: 420.0.into(),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

fn inventory_grid_render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &WatchedResource<Inventory>,
        &WatchedResource<SelectedItem>,
        &WatchedResource<Equipment>,
        &mut WidgetChildren,
    )>,
) {
    let Ok((inventory, selected, equipment, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let mut grid = WidgetChildren::default();
    for item in inventory.0.iter() {
        grid.add::<ItemSlot>((ItemSlot {
            item_id: item.id,
            selected: selected.get() == Some(item.id),
        },));
        grid.add_key(item.id.to_string());
    }

    let grid_wrapper = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Row,
            flex_wrap: WidgetFlexWrap::Wrap,
            gap: (8.0.into(), 8.0.into()),
            ..Default::default()
        },
        grid,
    ));

    *children = WidgetChildren::default()
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: 220.0.into(),
                ..Default::default()
            },
            theme::scrollable_panel(grid_wrapper),
        ))
        .with_key("grid")
        .with_child::<Element>(detail_pane(*current_widget, inventory, selected, equipment))
        .with_key("detail");

    children.apply(current_widget.as_parent());
}
