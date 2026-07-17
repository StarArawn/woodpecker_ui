use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::mock_data::HotbarState;
use crate::theme;

/// A hotbar ability icon with a cooldown depletion overlay.
///
/// Cooldown remaining is shown as a numeric countdown plus a shrinking flat-color overlay,
/// since `woodpecker_ui` has no radial/pie progress primitive.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WidgetChildren, WoodpeckerStyle, WatchedResource<HotbarState>)]
pub struct AbilitySlot {
    pub index: usize,
}

const SLOT_SIZE: f32 = 56.0;

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &AbilitySlot,
        &WatchedResource<HotbarState>,
        &mut WidgetChildren,
    )>,
) {
    let Ok((slot, hotbar, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;
    let Some(data) = hotbar.0.get(slot.index) else {
        return;
    };

    let cooldown_remaining = data.cooldown_remaining;
    let cooldown_total = data.cooldown_total;
    let cooling = cooldown_remaining > 0.0;
    let fraction = if cooldown_total > 0.0 {
        (cooldown_remaining / cooldown_total).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let button_normal = WoodpeckerStyle {
        background_color: data.icon_color,
        border_color: theme::BORDER_GOLD,
        border: Edge::all(2.0),
        border_radius: Corner::all(theme::RADIUS_SM),
        ..Default::default()
    };
    let icon_button_styles = IconButtonStyles {
        normal: button_normal,
        hovered: WoodpeckerStyle {
            border_color: Color::WHITE,
            ..button_normal
        },
        width: SLOT_SIZE.into(),
        height: SLOT_SIZE.into(),
    };

    let mut inner = WidgetChildren::default();
    inner.add::<Element>((
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
                font_size: 22.0,
                color: Color::WHITE,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: data.glyph.to_string(),
            },
        )),
    ));
    inner.add_key("glyph");

    inner.add::<Element>((
        Element,
        WoodpeckerStyle {
            position: WidgetPosition::Absolute,
            left: 3.0.into(),
            top: 2.0.into(),
            font_size: 10.0,
            color: theme::TEXT_MUTED,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: (slot.index + 1).to_string(),
        },
    ));
    inner.add_key("keybind");

    if cooling {
        // No `Pickable`, so clicks bubble through to `IconButton`'s observer below.
        inner.add::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                left: 0.0.into(),
                bottom: 0.0.into(),
                width: Units::Percentage(100.0),
                height: Units::Percentage(fraction * 100.0),
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.65),
                ..Default::default()
            },
            WidgetRender::Quad,
        ));
        inner.add_key("cooldown_overlay");

        inner.add::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                justify_content: Some(WidgetAlignContent::Center),
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("{}", cooldown_remaining.ceil() as u32),
                },
            )),
        ));
        inner.add_key("countdown");
    }

    let index = slot.index;
    *children = WidgetChildren::default()
        .with_child::<IconButton>((IconButton, icon_button_styles, inner))
        .with_key("button")
        .with_observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut hotbar: ResMut<HotbarState>| {
                let Some(slot) = hotbar.0.get_mut(index) else {
                    return;
                };
                if slot.cooldown_remaining <= 0.0 {
                    slot.cooldown_remaining = slot.cooldown_total;
                }
            },
        );

    children.apply(current_widget.as_parent());
}
