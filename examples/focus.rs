use bevy::prelude::*;
use bevy_vello::render::VelloView;
use woodpecker_ui::prelude::*;

#[derive(Component, Widget, Reflect, Default, Clone, PartialEq)]
#[widget_systems(focus_update, focus_render)]
pub struct FocusWidget {
    focused: bool,
    hovered: bool,
}

fn focus_update(entity: Res<CurrentWidget>, query: Query<Entity, Changed<FocusWidget>>) -> bool {
    query.contains(**entity)
}

fn focus_render(
    entity: Res<CurrentWidget>,
    mut query: Query<(&FocusWidget, &mut WoodpeckerStyle)>,
    mut children_query: Query<&mut WidgetChildren>,
) {
    let Ok((widget, mut styles)) = query.get_mut(**entity) else {
        return;
    };

    let children = children_query.get_mut(**entity);

    let Ok(mut children) = children else {
        // Only change colors for widgets without children.
        match (widget.hovered, widget.focused) {
            (false, true) => styles.background_color = Color::srgba(1.0, 0.0, 1.0, 1.0),
            (true, false) => styles.background_color = Color::srgba(0.0, 1.0, 0.0, 1.0),
            _ => styles.background_color = Color::srgba(1.0, 0.0, 0.0, 1.0),
        }
        return;
    };

    // For loops just like regular rust syntax! No need for weirdness here.
    // You can also use iterators no issues!
    for _ in 0..5 {
        children
            .add::<FocusWidget>((
                WidgetRender::Quad,
                FocusWidget::default(),
                WoodpeckerStyle {
                    width: 100.0.into(),
                    height: 100.0.into(),
                    margin: Edge::all(50.0),
                    background_color: Color::srgba(1.0, 0.0, 0.0, 1.0),
                    border_radius: Corner::new(10.0.into(), 10.0.into(), 0.0.into(), 10.0.into()),
                    ..Default::default()
                },
                Pickable::default(),
                Focusable,
                // On::<WidgetKeyboardCharEvent>::target_component_mut::<FocusWidget>(
                //     |event, _vello_widget| info!("Widget {} got key {}!", event.target, event.c),
                // ),
            ))
            .observe(
                |trigger: Trigger<Pointer<Over>>, mut query: Query<&mut FocusWidget>| {
                    let Ok(mut widget) = query.get_mut(trigger.target) else {
                        return;
                    };

                    if !widget.focused {
                        widget.hovered = true;
                    }
                },
            )
            .observe(
                |trigger: Trigger<Pointer<Out>>, mut query: Query<&mut FocusWidget>| {
                    let Ok(mut widget) = query.get_mut(trigger.target) else {
                        return;
                    };

                    if !widget.focused {
                        widget.hovered = false;
                    }
                },
            )
            .observe(
                |trigger: Trigger<WidgetFocus>, mut query: Query<&mut FocusWidget>| {
                    let Ok(mut widget) = query.get_mut(trigger.target()) else {
                        return;
                    };
                    widget.focused = true;
                    widget.hovered = false;
                },
            )
            .observe(
                |trigger: Trigger<WidgetBlur>, mut query: Query<&mut FocusWidget>| {
                    let Ok(mut widget) = query.get_mut(trigger.target()) else {
                        return;
                    };
                    widget.focused = false;
                    widget.hovered = false;
                },
            )
            .observe(|trigger: Trigger<WidgetKeyboardCharEvent>| {
                info!("Widget {} got key {}!", trigger.target, trigger.c)
            });
    }
    children.apply(entity.as_parent());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<FocusWidget>()
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, VelloView));

    let mut root_children = WidgetChildren::default();

    root_children.add::<FocusWidget>((
        WidgetRender::Quad,
        FocusWidget::default(),
        WoodpeckerStyle {
            align_content: Some(WidgetAlignContent::SpaceEvenly),
            justify_content: Some(WidgetAlignContent::SpaceEvenly),
            margin: Edge::all(50.0),
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            background_color: Color::srgba(0.0, 0.0, 1.0, 1.0),
            border_radius: Corner::all(50.0),
            ..Default::default()
        },
        WidgetChildren::default(),
    ));

    let root = commands
        .spawn((WoodpeckerAppBundle {
            children: root_children,
            ..Default::default()
        },))
        .id();

    ui_context.set_root_widget(root);
}
