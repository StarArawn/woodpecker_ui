use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::inventory_panel::DraggableItem;
use crate::mock_data::*;
use crate::theme;

/// One equipment slot: a display label, the `InventoryItem::item_type` it accepts, and
/// getter/setter pairs into `Equipment`'s fields. The setter lets a drop handler target the
/// specific slot under the pointer (e.g. distinguishing "Ring 1" from "Ring 2").
struct EquipmentSlotDef {
    label: &'static str,
    item_type: &'static str,
    get: fn(&Equipment) -> Option<u64>,
    set: fn(&mut Equipment) -> &mut Option<u64>,
}

const EQUIPMENT_SLOTS: [EquipmentSlotDef; 6] = [
    EquipmentSlotDef {
        label: "Weapon",
        item_type: "Weapon",
        get: |e| e.weapon,
        set: |e| &mut e.weapon,
    },
    EquipmentSlotDef {
        label: "Armor",
        item_type: "Armor",
        get: |e| e.armor,
        set: |e| &mut e.armor,
    },
    EquipmentSlotDef {
        label: "Helm",
        item_type: "Helm",
        get: |e| e.helm,
        set: |e| &mut e.helm,
    },
    EquipmentSlotDef {
        label: "Boots",
        item_type: "Boots",
        get: |e| e.boots,
        set: |e| &mut e.boots,
    },
    EquipmentSlotDef {
        label: "Ring 1",
        item_type: "Ring",
        get: |e| e.ring1,
        set: |e| &mut e.ring1,
    },
    EquipmentSlotDef {
        label: "Ring 2",
        item_type: "Ring",
        get: |e| e.ring2,
        set: |e| &mut e.ring2,
    },
];

/// Per-`CharacterSheet` state: which equipment slot (if any) a drag is currently hovering,
/// and whether the dragged item is valid for it -- drives the green/red border highlight in
/// `equipment_square`.
#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct CharacterSheetState {
    hovered_slot: Option<usize>,
    hover_valid: bool,
}

/// `drop_highlight`: `Some(true)`/`Some(false)` for a valid/invalid drag hover (green/red
/// border), `None` otherwise (normal rarity/empty-slot border).
fn equipment_square(
    label: &str,
    item: Option<&InventoryItem>,
    drop_highlight: Option<bool>,
) -> impl Bundle + Clone {
    let (glyph, mut border_color, icon_color) = match item {
        Some(item) => (
            item.glyph.to_string(),
            theme::rarity_color(item.rarity),
            item.icon_color,
        ),
        None => ("".into(), theme::BORDER_DARK, theme::BG_INPUT),
    };
    let border_width = if drop_highlight.is_some() { 3.0 } else { 2.0 };
    if let Some(valid) = drop_highlight {
        border_color = if valid { theme::STAMINA } else { theme::HEALTH };
    }
    (
        Element,
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Column,
            align_items: Some(WidgetAlignItems::Center),
            gap: (0.0.into(), 4.0.into()),
            ..Default::default()
        },
        Pickable::default(),
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: 48.0.into(),
                    height: 48.0.into(),
                    background_color: theme::BG_INPUT,
                    border: Edge::all(border_width),
                    border_color,
                    border_radius: Corner::all(theme::RADIUS_SM),
                    justify_content: Some(WidgetAlignContent::Center),
                    align_items: Some(WidgetAlignItems::Center),
                    ..Default::default()
                },
                WidgetRender::Quad,
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 18.0,
                        color: icon_color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text { content: glyph },
                )),
            ))
            .with_key("icon")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 10.0,
                    color: theme::TEXT_MUTED,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: label.into(),
                },
            ))
            .with_key("label"),
    )
}

fn stat_line(label: &str, value: String) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Row,
            width: Units::Percentage(100.0),
            justify_content: Some(WidgetAlignContent::SpaceBetween),
            margin: Edge::all(0.0).bottom(4.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 12.0,
                    color: theme::TEXT_MUTED,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: label.into(),
                },
            ))
            .with_key("label")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 12.0,
                    color: theme::TEXT_PRIMARY,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text { content: value },
            ))
            .with_key("value"),
    )
}

/// Watches `Equipment`, `PlayerVitals`, and `Inventory`. Shows a portrait, a 6-slot equipment
/// grid, a derived stat list, and a `LineChart` power trend.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(character_sheet_render)]
#[require(
    WidgetChildren,
    WoodpeckerStyle = default_style(),
    WatchedResource<Equipment>,
    WatchedResource<PlayerVitals>,
    WatchedResource<Inventory>
)]
pub struct CharacterSheet;

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: 420.0.into(),
        // Fixed height, content scrolls inside it (see `scrollable_panel` below), so this
        // panel doesn't grow unbounded and overlap other floating panels.
        height: 460.0.into(),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

fn character_sheet_render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &WatchedResource<Equipment>,
        &WatchedResource<PlayerVitals>,
        &WatchedResource<Inventory>,
        &mut WidgetChildren,
    )>,
    state_query: Query<&CharacterSheetState>,
) {
    let Ok((equipment, vitals, inventory, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        CharacterSheetState::default(),
    );
    let current_widget = *current_widget;
    let default_state = CharacterSheetState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    let mut equipment_row = WidgetChildren::default();
    for (i, slot) in EQUIPMENT_SLOTS.iter().enumerate() {
        let item = (slot.get)(equipment).and_then(|id| inventory.0.iter().find(|it| it.id == id));
        let drop_highlight = (state.hovered_slot == Some(i)).then_some(state.hover_valid);
        equipment_row.add::<Element>(equipment_square(slot.label, item, drop_highlight));
        equipment_row.add_key(slot.label);

        let item_type = slot.item_type;
        let set_fn = slot.set;
        equipment_row.droppable(
            current_widget,
            state_entity,
            move |dragged: &DraggableItem, _state: &CharacterSheetState| {
                dragged.item_type == item_type
            },
            move |state: &mut CharacterSheetState, hover| match hover {
                Some(valid) => {
                    state.hovered_slot = Some(i);
                    state.hover_valid = valid;
                }
                None => {
                    if state.hovered_slot == Some(i) {
                        state.hovered_slot = None;
                    }
                }
            },
            move |dragged: &DraggableItem, valid: bool, _state: &CharacterSheetState, commands: &mut Commands| {
                let item_id = dragged.id;
                commands.queue(move |world: &mut World| {
                    let Some(item) = world
                        .resource::<Inventory>()
                        .0
                        .iter()
                        .find(|it| it.id == item_id)
                        .cloned()
                    else {
                        return;
                    };

                    if !valid {
                        world.resource_mut::<ToastQueue>().push(
                            format!("{} can't be equipped there.", item.name),
                            BadgeVariant::Danger,
                        );
                        return;
                    }

                    *(set_fn)(&mut world.resource_mut::<Equipment>()) = Some(item.id);
                    world
                        .resource_mut::<ToastQueue>()
                        .push(format!("Equipped {}.", item.name), BadgeVariant::Success);
                });
            },
        );
    }

    let content = WidgetChildren::default()
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Row,
                align_items: Some(WidgetAlignItems::Center),
                gap: (theme::SPACE_MD.into(), 0.0.into()),
                margin: Edge::all(0.0).bottom(theme::SPACE_MD),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Avatar>((Avatar {
                    initials: "AT".into(),
                    color: Some(theme::BORDER_GOLD),
                    size: 64.0,
                },))
                .with_key("avatar")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        flex_direction: WidgetFlexDirection::Column,
                        flex_grow: 1.0,
                        ..Default::default()
                    },
                    WidgetChildren::default()
                        .with_child::<Element>(stat_line("Level", vitals.level.to_string()))
                        .with_key("level")
                        .with_child::<Element>(stat_line("Gold", vitals.gold.to_string()))
                        .with_key("gold")
                        .with_child::<Element>(stat_line(
                            "Health",
                            format!("{}/{}", vitals.health as i32, vitals.health_max as i32),
                        ))
                        .with_key("health")
                        .with_child::<Element>(stat_line(
                            "Mana",
                            format!("{}/{}", vitals.mana as i32, vitals.mana_max as i32),
                        ))
                        .with_key("mana"),
                ))
                .with_key("stats"),
        ))
        .with_key("header")
        .with_child::<Element>((
            Element,
            theme::section_title_style(),
            WidgetRender::Text {
                content: "Equipment".into(),
            },
        ))
        .with_key("equipment_title")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Row,
                gap: (theme::SPACE_SM.into(), 0.0.into()),
                margin: Edge::all(0.0).bottom(theme::SPACE_MD),
                ..Default::default()
            },
            equipment_row,
        ))
        .with_key("equipment")
        .with_child::<Element>((
            Element,
            theme::section_title_style(),
            WidgetRender::Text {
                content: "Power Trend".into(),
            },
        ))
        .with_key("chart_title")
        .with_child::<LineChart>((LineChart {
            series: vec![ChartSeries::new("Power", seed_power_trend(), theme::XP)],
            show_legend: false,
        },))
        .with_key("chart");

    *children = theme::scrollable_panel(content);

    children.apply(current_widget.as_parent());
}
