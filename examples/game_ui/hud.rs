use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::ability_slot::AbilitySlot;
use crate::mock_data::*;
use crate::stat_bar::{StatBar, StatBarStyles};
use crate::theme;

/// The always-on, never-draggable HUD -- fixed to the full viewport, sitting alongside (not
/// inside) `game_menu.rs`'s `WindowingContextProvider`.
pub fn build_hud(current_widget: CurrentWidget, hotbar_slots: usize) -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            position: WidgetPosition::Fixed,
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    position: WidgetPosition::Absolute,
                    left: 20.0.into(),
                    top: 20.0.into(),
                    flex_direction: WidgetFlexDirection::Column,
                    gap: (0.0.into(), 12.0.into()),
                    ..Default::default()
                },
                WidgetChildren::default()
                    .with_child::<VitalsCluster>((VitalsCluster,))
                    .with_key("vitals")
                    .with_child::<BuffRow>((BuffRow,))
                    .with_key("buffs")
                    .with_child::<PartyFrames>((PartyFrames,))
                    .with_key("party"),
            ))
            .with_key("left_column")
            .with_child::<Element>(minimap_placeholder())
            .with_key("minimap")
            .with_child::<Element>(quick_launch_row(current_widget))
            .with_key("quicklaunch")
            .with_child::<Element>(bottom_center(current_widget, hotbar_slots))
            .with_key("bottom_center"),
    ))
}

fn minimap_placeholder() -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            position: WidgetPosition::Absolute,
            top: 20.0.into(),
            right: 20.0.into(),
            width: 140.0.into(),
            height: 140.0.into(),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..theme::hud_panel_style()
        },
        WidgetRender::Quad,
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 12.0,
                color: theme::TEXT_MUTED,
                ..Default::default()
            },
            WidgetRender::Text {
                content: "Map".into(),
            },
        )),
    )
}

fn quick_launch_row(current_widget: CurrentWidget) -> impl Bundle + Clone {
    let mut children = WidgetChildren::default();
    let buttons: [(&str, fn(&mut PanelVisibility)); 5] = [
        ("I", |v| v.inventory = !v.inventory),
        ("C", |v| v.character = !v.character),
        ("Q", |v| v.quests = !v.quests),
        ("S", |v| v.settings = !v.settings),
        ("G", |v| v.guild = !v.guild),
    ];
    for (glyph, toggle) in buttons {
        children.add::<IconButton>((
            IconButton,
            IconButtonStyles {
                width: 40.0.into(),
                height: 40.0.into(),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<Element>((
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
                        font_size: 14.0,
                        color: theme::TEXT_PRIMARY,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: glyph.into(),
                    },
                )),
            )),
        ));
        children.add_key(glyph);
        children.observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut visibility: ResMut<PanelVisibility>| {
                toggle(&mut visibility);
            },
        );
    }

    (
        Element,
        WoodpeckerStyle {
            position: WidgetPosition::Absolute,
            top: 172.0.into(),
            right: 20.0.into(),
            flex_direction: WidgetFlexDirection::Row,
            gap: (8.0.into(), 0.0.into()),
            ..Default::default()
        },
        children,
    )
}

fn bottom_center(current_widget: CurrentWidget, hotbar_slots: usize) -> impl Bundle + Clone {
    let mut dev_row = WidgetChildren::default();
    let actions: [(&str, fn(&mut Commands, CurrentWidget)); 6] = [
        ("Take Damage", |commands, _cw| {
            commands.queue(|world: &mut World| {
                if let Some(mut vitals) = world.get_resource_mut::<PlayerVitals>() {
                    vitals.health = (vitals.health - vitals.health_max * 0.15).max(0.0);
                }
                if let Some(mut buffs) = world.get_resource_mut::<BuffState>() {
                    buffs.push(
                        "Bleeding",
                        BadgeVariant::Danger,
                        "Losing health over time",
                        6.0,
                    );
                }
            });
        }),
        ("Heal", |commands, _cw| {
            commands.queue(|world: &mut World| {
                if let Some(mut vitals) = world.get_resource_mut::<PlayerVitals>() {
                    vitals.health =
                        (vitals.health + vitals.health_max * 0.2).min(vitals.health_max);
                }
                if let Some(mut buffs) = world.get_resource_mut::<BuffState>() {
                    buffs.push(
                        "Regeneration",
                        BadgeVariant::Success,
                        "Recovering health",
                        4.0,
                    );
                }
            });
        }),
        ("Use Mana", |commands, _cw| {
            commands.queue(|world: &mut World| {
                if let Some(mut vitals) = world.get_resource_mut::<PlayerVitals>() {
                    vitals.mana = (vitals.mana - vitals.mana_max * 0.25).max(0.0);
                }
            });
        }),
        ("Sprint", |commands, _cw| {
            commands.queue(|world: &mut World| {
                if let Some(mut vitals) = world.get_resource_mut::<PlayerVitals>() {
                    vitals.stamina = (vitals.stamina - vitals.stamina_max * 0.35).max(0.0);
                }
            });
        }),
        ("Loot Item", |commands, _cw| {
            commands.queue(|world: &mut World| {
                let mut counter = world.get_resource_or_insert_with(LootCounter::default);
                let item = roll_loot(&mut counter);
                let rarity = item.rarity;
                let name = item.name.clone();
                if let Some(mut inventory) = world.get_resource_mut::<Inventory>() {
                    inventory.0.push(item);
                }
                if let Some(mut toasts) = world.get_resource_mut::<ToastQueue>() {
                    toasts.push(
                        format!("Looted {name} ({})", rarity.label()),
                        rarity.badge_variant(),
                    );
                }
            });
        }),
        ("Grant XP", |commands, _cw| {
            commands.queue(|world: &mut World| {
                let leveled = world
                    .get_resource_mut::<PlayerVitals>()
                    .and_then(|mut vitals| vitals.grant_xp(140.0));
                if let Some(new_level) = leveled {
                    if let Some(mut flag) = world.get_resource_mut::<LevelUpFlag>() {
                        flag.0 = Some(new_level);
                    }
                    if let Some(mut toasts) = world.get_resource_mut::<ToastQueue>() {
                        toasts.push(format!("Level {new_level}!"), BadgeVariant::Info);
                    }
                }
            });
        }),
    ];
    for (label, action) in actions {
        dev_row.add::<WButton>((
            WButton,
            // `min_width` breaks the circular layout dependency for an `Auto`-width button:
            // the button can't size to its text until the text is measured against a width.
            ButtonStyles {
                normal: WoodpeckerStyle {
                    min_width: 100.0.into(),
                    ..theme::secondary_button_styles().normal
                },
                hovered: WoodpeckerStyle {
                    min_width: 100.0.into(),
                    ..theme::secondary_button_styles().hovered
                },
            },
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: theme::TEXT_PRIMARY,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: label.into(),
                },
            )),
        ));
        dev_row.add_key(label);
        dev_row.observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut commands: Commands| {
                action(&mut commands, current_widget);
            },
        );
    }

    let mut hotbar_row = WidgetChildren::default();
    for i in 0..hotbar_slots {
        hotbar_row.add::<AbilitySlot>((AbilitySlot { index: i },));
        hotbar_row.add_key(i.to_string());
    }

    (
        Element,
        WoodpeckerStyle {
            position: WidgetPosition::Absolute,
            left: 0.0.into(),
            right: 0.0.into(),
            bottom: 16.0.into(),
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            align_items: Some(WidgetAlignItems::Center),
            gap: (0.0.into(), 10.0.into()),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_direction: WidgetFlexDirection::Row,
                    flex_wrap: WidgetFlexWrap::Wrap,
                    justify_content: Some(WidgetAlignContent::Center),
                    gap: (6.0.into(), 6.0.into()),
                    ..Default::default()
                },
                dev_row,
            ))
            .with_key("dev_row")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_direction: WidgetFlexDirection::Row,
                    gap: (10.0.into(), 0.0.into()),
                    padding: Edge::all(theme::SPACE_SM),
                    ..theme::hud_panel_style()
                },
                WidgetRender::Quad,
                hotbar_row,
            ))
            .with_key("hotbar_row"),
    )
}

fn stat_row(
    caption: &str,
    value: f32,
    label: String,
    fill_color: Color,
    track_color: Color,
) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Column,
            gap: (0.0.into(), 2.0.into()),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 11.0,
                    color: theme::TEXT_MUTED,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: caption.into(),
                },
            ))
            .with_key("caption")
            .with_child::<StatBar>((
                StatBar {
                    value,
                    label: Some(label),
                },
                StatBarStyles {
                    track_color,
                    fill_color,
                    height: 16.0,
                },
                WoodpeckerStyle {
                    width: 220.0.into(),
                    ..Default::default()
                },
            ))
            .with_key("bar"),
    )
}

/// Watches `PlayerVitals` (drives the 4 stat bars) and `GameSettings` (the accent trim --
/// a cross-panel link from Settings to HUD).
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(vitals_cluster_render)]
#[require(WidgetChildren, WoodpeckerStyle, WatchedResource<PlayerVitals>, WatchedResource<GameSettings>)]
pub(crate) struct VitalsCluster;

fn vitals_cluster_render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &WatchedResource<PlayerVitals>,
        &WatchedResource<GameSettings>,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
) {
    let Ok((vitals, settings, mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    *styles = WoodpeckerStyle {
        border_color: settings.accent_color,
        ..theme::hud_panel_style()
    };

    *children = WidgetChildren::default()
        .with_child::<Element>(stat_row(
            "Health",
            vitals.health / vitals.health_max,
            format!("{}/{}", vitals.health as i32, vitals.health_max as i32),
            theme::HEALTH,
            theme::HEALTH_TRACK,
        ))
        .with_key("health")
        .with_child::<Element>(stat_row(
            "Mana",
            vitals.mana / vitals.mana_max,
            format!("{}/{}", vitals.mana as i32, vitals.mana_max as i32),
            theme::MANA,
            theme::MANA_TRACK,
        ))
        .with_key("mana")
        .with_child::<Element>(stat_row(
            "Stamina",
            vitals.stamina / vitals.stamina_max,
            format!("{}/{}", vitals.stamina as i32, vitals.stamina_max as i32),
            theme::STAMINA,
            theme::STAMINA_TRACK,
        ))
        .with_key("stamina")
        .with_child::<Element>(stat_row(
            "XP",
            vitals.xp / vitals.xp_max,
            format!("Lvl {}", vitals.level),
            theme::XP,
            theme::XP_TRACK,
        ))
        .with_key("xp");

    children.apply(current_widget.as_parent());
}

/// Watches `BuffState`; each active buff/debuff is a `Badge` wrapped in a real `Tooltip`.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(buff_row_render)]
#[require(WidgetChildren, WoodpeckerStyle, WatchedResource<BuffState>)]
pub(crate) struct BuffRow;

fn buff_row_render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&WatchedResource<BuffState>, &mut WidgetChildren)>,
) {
    let Ok((buffs, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    *children = WidgetChildren::default();
    if buffs.buffs.is_empty() {
        children.apply(current_widget.as_parent());
        return;
    }

    let mut row = WidgetChildren::default();
    for buff in buffs.buffs.iter() {
        row.add::<Tooltip>((
            Tooltip {
                text: buff.tooltip.clone(),
                placement: PopoverPlacement::Bottom,
            },
            PassedChildren(WidgetChildren::default().with_child::<Badge>((Badge {
                label: buff.label.clone(),
                variant: buff.variant,
            },))),
        ));
        row.add_key(buff.id.to_string());
    }

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Row,
            flex_wrap: WidgetFlexWrap::Wrap,
            gap: (6.0.into(), 6.0.into()),
            ..Default::default()
        },
        row,
    ));
    children.add_key("row");

    children.apply(current_widget.as_parent());
}

/// Watches `PartyState`; each member is an `Avatar` + name + a plain `ProgressBar`.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(party_frames_render)]
#[require(WidgetChildren, WoodpeckerStyle, WatchedResource<PartyState>)]
pub(crate) struct PartyFrames;

fn party_frames_render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &WatchedResource<PartyState>,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
) {
    let Ok((party, mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    *styles = WoodpeckerStyle {
        flex_direction: WidgetFlexDirection::Column,
        gap: (0.0.into(), 8.0.into()),
        padding: Edge::all(theme::SPACE_SM),
        ..theme::hud_panel_style()
    };

    *children = WidgetChildren::default();
    for member in party.0.iter() {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Row,
                align_items: Some(WidgetAlignItems::Center),
                gap: (8.0.into(), 0.0.into()),
                width: 200.0.into(),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Avatar>((Avatar {
                    initials: member.initials.clone(),
                    color: Some(member.avatar_color),
                    size: 28.0,
                },))
                .with_key("avatar")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        flex_direction: WidgetFlexDirection::Column,
                        flex_grow: 1.0,
                        gap: (0.0.into(), 3.0.into()),
                        ..Default::default()
                    },
                    WidgetChildren::default()
                        .with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                font_size: 11.0,
                                color: theme::TEXT_PRIMARY,
                                text_wrap: TextWrap::None,
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                content: member.name.clone(),
                            },
                        ))
                        .with_key("name")
                        .with_child::<ProgressBar>((
                            ProgressBar {
                                value: member.hp_fraction,
                            },
                            ProgressBarStyles {
                                track_color: theme::HEALTH_TRACK,
                                fill_color: theme::HEALTH,
                                height: 6.0,
                            },
                        ))
                        .with_key("hp"),
                ))
                .with_key("info"),
        ));
        children.add_key(member.name.clone());
    }

    children.apply(current_widget.as_parent());
}
