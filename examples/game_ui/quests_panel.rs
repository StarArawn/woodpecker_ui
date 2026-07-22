use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::mock_data::*;
use crate::theme;

fn quest_entry(current_widget: CurrentWidget, quest: &Quest) -> impl Bundle + Clone {
    let quest_id = quest.id;
    let badge_variant = match quest.kind {
        QuestKind::Main => BadgeVariant::Warning,
        QuestKind::Side => BadgeVariant::Neutral,
    };
    let badge_label = match quest.kind {
        QuestKind::Main => "Main",
        QuestKind::Side => "Side",
    };

    let mut children = WidgetChildren::default();
    children
        .add::<Element>((
            Element,
            WoodpeckerStyle {
                flex_direction: WidgetFlexDirection::Row,
                align_items: Some(WidgetAlignItems::Center),
                gap: (8.0.into(), 0.0.into()),
                margin: Edge::all(0.0).bottom(theme::SPACE_XS),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Badge>((Badge {
                    label: badge_label.into(),
                    variant: badge_variant,
                },))
                .with_key("kind")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 14.0,
                        color: theme::PARCHMENT_TEXT,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: quest.title.clone(),
                    },
                ))
                .with_key("title"),
        ))
        .add_key("header");

    for (i, objective) in quest.objectives.iter().enumerate() {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 12.0,
                color: theme::PARCHMENT_TEXT,
                text_wrap: TextWrap::None,
                margin: Edge::all(0.0).bottom(2.0),
                ..Default::default()
            },
            WidgetRender::Text {
                content: format!(
                    "{} ({}/{})",
                    objective.label, objective.current, objective.target
                ),
            },
        ));
        children.add_key(format!("objective{i}"));
    }

    children.add::<ProgressBar>((
        ProgressBar {
            value: quest.progress(),
        },
        ProgressBarStyles {
            track_color: theme::BG_INPUT,
            fill_color: if quest.completed {
                theme::STAMINA
            } else {
                theme::XP
            },
            height: 8.0,
        },
    ));
    children.add_key("progress");

    let mut actions = WidgetChildren::default();
    if quest.completed {
        actions.add::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 12.0,
                color: theme::STAMINA,
                margin: Edge::all(0.0).top(theme::SPACE_XS),
                ..Default::default()
            },
            WidgetRender::Text {
                content: "Completed".into(),
            },
        ));
        actions.add_key("completed");
    } else if quest.progress() >= 1.0 {
        actions.add::<WButton>((
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
                    content: "Turn In".into(),
                },
            )),
        ));
        actions.add_key("turn_in");
        actions.observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>,
                  mut quests: ResMut<Quests>,
                  mut vitals: ResMut<PlayerVitals>,
                  mut level_up: ResMut<LevelUpFlag>,
                  mut toasts: ResMut<ToastQueue>| {
                let Some(quest) = quests.0.iter_mut().find(|q| q.id == quest_id) else {
                    return;
                };
                if quest.completed {
                    return;
                }
                quest.completed = true;
                let title = quest.title.clone();
                toasts.push(format!("Quest complete: {title}"), BadgeVariant::Success);
                if let Some(new_level) = vitals.grant_xp(80.0) {
                    level_up.0 = Some(new_level);
                    toasts.push(format!("Level {new_level}!"), BadgeVariant::Info);
                }
            },
        );
    } else {
        actions.add::<WButton>((
            WButton,
            theme::secondary_button_styles(),
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: theme::TEXT_PRIMARY,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Advance".into(),
                },
            )),
        ));
        actions.add_key("advance");
        actions.observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut quests: ResMut<Quests>| {
                let Some(quest) = quests.0.iter_mut().find(|q| q.id == quest_id) else {
                    return;
                };
                for objective in quest.objectives.iter_mut() {
                    if objective.current < objective.target {
                        objective.current += 1;
                    }
                }
            },
        );
    }
    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            margin: Edge::all(0.0).top(theme::SPACE_SM),
            ..Default::default()
        },
        actions,
    ));
    children.add_key("actions");

    (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            padding: Edge::all(theme::SPACE_SM),
            margin: Edge::all(0.0).bottom(theme::SPACE_SM),
            background_color: theme::PARCHMENT,
            border_radius: Corner::all(theme::RADIUS_SM),
            ..Default::default()
        },
        WidgetRender::Quad,
        children,
    )
}

/// Watches `Quests`. Wraps entries in exactly one `ScrollContextProvider`.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(quest_log_render)]
#[require(WidgetChildren, WoodpeckerStyle = default_style(), WatchedResource<Quests>)]
pub struct QuestLog;

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: 420.0.into(),
        height: 360.0.into(),
        ..Default::default()
    }
}

fn quest_log_render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&WatchedResource<Quests>, &mut WidgetChildren)>,
) {
    let Ok((quests, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let mut list = WidgetChildren::default();
    for quest in quests.0.iter() {
        list.add::<Element>(quest_entry(*current_widget, quest));
        list.add_key(quest.id.to_string());
    }

    *children = theme::scrollable_panel(list);

    children.apply(current_widget.as_parent());
}
