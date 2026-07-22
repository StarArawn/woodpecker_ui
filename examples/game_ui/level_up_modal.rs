use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::mock_data::LevelUpFlag;
use crate::theme;

/// `Option<u32>`-drives-`Modal::visible`, the same shape as
/// `examples/dashboard/mock_data.rs`'s `EditingUser`/`UserEditorModal`.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(level_up_celebration_render)]
#[require(WidgetChildren, WoodpeckerStyle, WatchedResource<LevelUpFlag>)]
pub struct LevelUpCelebration;

fn level_up_celebration_render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&WatchedResource<LevelUpFlag>, &mut WidgetChildren)>,
) {
    let Ok((flag, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;
    let level = flag.get();

    let mut content = WidgetChildren::default();
    content.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 36.0,
            color: theme::XP,
            margin: Edge::all(0.0).bottom(theme::SPACE_SM),
            ..Default::default()
        },
        WidgetRender::Text {
            content: format!("Level {}!", level.unwrap_or(0)),
        },
    ));
    content.add_key("level_text");
    content.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 13.0,
            color: theme::TEXT_MUTED,
            margin: Edge::all(0.0).bottom(theme::SPACE_MD),
            ..Default::default()
        },
        WidgetRender::Text {
            content: "Your power grows.".into(),
        },
    ));
    content.add_key("flavor");
    content.add::<WButton>((
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
                content: "Continue".into(),
            },
        )),
    ));
    content.add_key("continue");
    content.observe(
        current_widget,
        |_trigger: On<Pointer<Click>>, mut flag: ResMut<LevelUpFlag>| {
            flag.0 = None;
        },
    );

    *children = WidgetChildren::default().with_child::<Modal>((
        Modal {
            visible: level.is_some(),
            title: "Level Up!".into(),
            ..Default::default()
        },
        PassedChildren(WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                flex_direction: WidgetFlexDirection::Column,
                padding: Edge::all(theme::SPACE_LG),
                width: Units::Percentage(100.0),
                ..Default::default()
            },
            content,
        ))),
    ));

    children.apply(current_widget.as_parent());
}
