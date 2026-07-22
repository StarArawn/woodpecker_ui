use bevy::prelude::*;
use woodpecker_ui::prelude::*;

/// A minimal demo of `WidgetChildren::drag_state`/`droppable`: one draggable card and two
/// drop zones. Drag the card into either zone to see it accept the drop and record where it
/// landed; drag it off-screen or release it outside both zones and it just springs back.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<DragDropDemo>()
        .register_widget::<Card>()
        .register_widget::<DropZone>()
        .register_watched_resource::<DropHistory>()
        .insert_resource(DropHistory::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert(
        WidgetChildren::default()
            .with_child::<DragDropDemo>(DragDropDemo)
            .with_key("demo")
            // Required for the card's drag ghost (see `card_render`'s `.with_drag_ghost`) to
            // portal above everything while dragging -- without this, `OverlayRoot` never gets
            // provisioned and the ghost silently never appears.
            .with_child::<OverlayRootWidget>(OverlayRootWidget),
    );
}

const CARD_SIZE: f32 = 90.0;
const ZONE_SIZE: f32 = 150.0;
/// Above every normal widget, below floating overlays (`Popover`/`Tooltip`/`ToastViewport`).
const CARD_DRAG_Z: u32 = 1500;

/// Where the card was last successfully dropped -- watched so the status line above the
/// card/zones updates live.
#[derive(Resource, Reflect, Clone, PartialEq, Default)]
struct DropHistory(Option<String>);

/// Marks the card's payload -- attached permanently to the card's own bundle (not inserted
/// reactively), per `drag_state`'s contract. Carries no data since this demo only ever has
/// one kind of draggable thing.
#[derive(Component, Clone, Copy)]
struct DraggableCard;

/// Root widget: watches `DropHistory` so the status line reflects the live drop state, and
/// lays out the card next to the two drop zones.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(demo_render)]
#[require(WoodpeckerStyle = demo_style(), WidgetChildren, WatchedResource<DropHistory>)]
struct DragDropDemo;

fn demo_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        padding: Edge::all(24.0),
        flex_direction: WidgetFlexDirection::Column,
        gap: (0.0.into(), 20.0.into()),
        ..Default::default()
    }
}

fn demo_render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&WatchedResource<DropHistory>, &mut WidgetChildren)>,
) {
    let Ok((history, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;

    let status = match &history.0 .0 {
        Some(zone) => format!("Dropped in {zone}!"),
        None => "Drag the card into a zone.".into(),
    };

    *children = WidgetChildren::default()
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                color: Color::WHITE,
                font_size: 16.0,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text { content: status },
        ))
        .with_key("status")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Row,
                align_items: Some(WidgetAlignItems::Start),
                gap: (32.0.into(), 0.0.into()),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Card>(Card)
                .with_key("card")
                .with_child::<DropZone>((DropZone {
                    label: "Zone A".into(),
                },))
                .with_key("zone-a")
                .with_child::<DropZone>((DropZone {
                    label: "Zone B".into(),
                },))
                .with_key("zone-b"),
        ))
        .with_key("row");

    children.apply(current_widget.as_parent());
}

/// Per-card drag state: whether it's being dragged, and (while dragging) the world-space
/// position its `Fixed`-positioned ghost should render at.
#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct CardState {
    dragging: bool,
    position: Vec2,
}

/// The draggable card.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(card_render)]
#[require(WoodpeckerStyle = card_style(), WidgetChildren)]
struct Card;

fn card_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: CARD_SIZE.into(),
        height: CARD_SIZE.into(),
        ..Default::default()
    }
}

fn card_render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    overlay_root: Option<Res<OverlayRoot>>,
    mut query: Query<&mut WidgetChildren, With<Card>>,
    state_query: Query<&CardState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;

    let state_entity = hooks.use_state(&mut commands, current_widget, CardState::default());
    let default_state = CardState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    let label = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            color: Color::WHITE,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: "Drag me".into(),
        },
    ));

    // While dragging, the box itself is hidden (`Card`'s own root -- see `card_style()` --
    // still unconditionally reserves `CARD_SIZE` in the row, so nothing reflows) rather than
    // despawned: its entity has to survive for the rest of the drag gesture, since that's what
    // actually receives the `Pointer<Drag>`/`DragEnd` events below. The visible drag ghost is
    // a separate, portaled child (see `.with_drag_ghost` below) -- merely restyling this
    // entity to `Fixed` in place (the previous approach) doesn't escape whatever `Clip`-masked
    // or z-bucketed ancestor it's declared inside.
    let box_style = if state.dragging {
        WoodpeckerStyle {
            display: WidgetDisplay::None,
            ..Default::default()
        }
    } else {
        WoodpeckerStyle {
            width: CARD_SIZE.into(),
            height: CARD_SIZE.into(),
            background_color: Color::srgb(0.373, 0.51, 0.965),
            border_radius: Corner::all(8.0),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        }
    };

    *children = WidgetChildren::default()
        .with_child::<Element>((
            Element,
            box_style,
            WidgetRender::Quad,
            Pickable::default(),
            DraggableCard,
            label.clone(),
        ))
        .with_key("box")
        .with_drag_state(
            current_widget,
            state_entity,
            |state: &mut CardState, phase| match phase {
                DragPhase::Start(pos) | DragPhase::Move(pos) => {
                    state.dragging = true;
                    state.position = pos;
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
                    width: CARD_SIZE.into(),
                    height: CARD_SIZE.into(),
                    background_color: Color::srgb(0.373, 0.51, 0.965),
                    border_radius: Corner::all(8.0),
                    justify_content: Some(WidgetAlignContent::Center),
                    align_items: Some(WidgetAlignItems::Center),
                    position: WidgetPosition::Fixed,
                    left: state.position.x.into(),
                    top: state.position.y.into(),
                    z_index: Some(WidgetZ::Global(CARD_DRAG_Z)),
                    opacity: 0.85,
                    ..Default::default()
                },
                WidgetRender::Quad,
                // The ghost floats over the pointer and would otherwise block its own drop
                // target's `DragEnter`/`DragLeave`/`DragDrop` (`Pickable::default()` blocks
                // picking underneath it). `IGNORE` lets pointer events pass through to the
                // zone beneath.
                Pickable::IGNORE,
                label,
            ),
        );

    children.apply(current_widget.as_parent());
}

/// Per-zone hover state: `Some(true)`/`Some(false)` while a valid/invalid drag hovers,
/// `None` otherwise.
#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct DropZoneState {
    hover_valid: Option<bool>,
}

/// A drop target for [`DraggableCard`].
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(drop_zone_render)]
#[require(WoodpeckerStyle = drop_zone_style(), WidgetChildren, Pickable)]
struct DropZone {
    label: String,
}

fn drop_zone_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: ZONE_SIZE.into(),
        height: ZONE_SIZE.into(),
        border: Edge::all(2.0),
        border_color: Color::srgb(0.4, 0.4, 0.45),
        border_radius: Corner::all(8.0),
        justify_content: Some(WidgetAlignContent::Center),
        align_items: Some(WidgetAlignItems::Center),
        ..Default::default()
    }
}

fn drop_zone_render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&DropZone, &mut WoodpeckerStyle, &mut WidgetChildren)>,
    state_query: Query<&DropZoneState>,
) {
    let Ok((zone, mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;

    let state_entity = hooks.use_state(&mut commands, current_widget, DropZoneState::default());
    let default_state = DropZoneState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    styles.border_color = match state.hover_valid {
        Some(true) => Color::srgb(0.35, 0.73, 0.5),
        Some(false) => Color::srgb(0.92, 0.4, 0.4),
        None => Color::srgb(0.4, 0.4, 0.45),
    };
    styles.background_color = if state.hover_valid == Some(true) {
        Color::srgba(0.35, 0.73, 0.5, 0.15)
    } else {
        Color::NONE
    };

    // Attached before any child is added, so `droppable`'s observers land on this zone's own
    // (full-size, already-`Pickable`) root entity -- not on the small label text child below.
    *children = WidgetChildren::default();
    let label = zone.label.clone();
    children.droppable(
        current_widget,
        state_entity,
        // Everything this demo ever drags is a valid drop -- a real app would inspect the
        // payload here (e.g. an item's type) to decide.
        |_payload: &DraggableCard, _state: &DropZoneState| true,
        |state: &mut DropZoneState, hover_valid| state.hover_valid = hover_valid,
        move |_payload: &DraggableCard, valid: bool, _state: &DropZoneState, commands: &mut Commands| {
            if valid {
                commands.insert_resource(DropHistory(Some(label.clone())));
            }
        },
    );
    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            color: Color::WHITE,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: zone.label.clone(),
        },
    ));
    children.add_key("label");

    children.apply(current_widget.as_parent());
}
