use bevy::prelude::*;
use bevy_mod_picking::{
    debug::DebugPickingMode,
    events::{Out, Over, Pointer},
    prelude::On,
    DefaultPickingPlugins, PickableBundle,
};
use woodpecker_ui::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct VelloWidget {
    focused: bool,
    hovered: bool,
}
impl Widget for VelloWidget {}

fn vello_update(entity: Res<CurrentWidget>, query: Query<Entity, Changed<VelloWidget>>) -> bool {
    query.contains(**entity)
}

fn vello_render(
    entity: Res<CurrentWidget>,
    mut query: Query<(&VelloWidget, &mut WidgetRender)>,
    mut children_query: Query<&mut WidgetChildren>,
) {
    let Ok((widget, mut widget_render)) = query.get_mut(**entity) else {
        return;
    };

    let children = children_query.get_mut(**entity);

    let Ok(mut children) = children else {
        // Only change colors for widgets without children.
        match (widget.hovered, widget.focused) {
            (false, true) => widget_render.set_color(Color::srgba(1.0, 0.0, 1.0, 1.0)),
            (true, false) => widget_render.set_color(Color::srgba(0.0, 1.0, 0.0, 1.0)),
            _ => widget_render.set_color(Color::srgba(1.0, 0.0, 0.0, 1.0)),
        }
        return;
    };

    for _ in 0..5 {
        children.add::<VelloWidget>((
            WidgetRender::Quad {
                color: Color::srgba(1.0, 0.0, 0.0, 1.0),
                border_radius: woodpecker_ui::prelude::kurbo::RoundedRectRadii::new(
                    10.0, 10.0, 0.0, 10.0,
                ),
            },
            VelloWidget::default(),
            WoodpeckerStyle {
                width: 100.0.into(),
                height: 100.0.into(),
                margin: Edge::all(50.0.into()),
                ..Default::default()
            },
            SpatialBundle::default(),
            PickableBundle::default(),
            Focusable,
            On::<Pointer<Over>>::listener_component_mut::<VelloWidget>(|_, vello_widget| {
                if !vello_widget.focused {
                    vello_widget.hovered = true;
                }
            }),
            On::<Pointer<Out>>::listener_component_mut::<VelloWidget>(|_, vello_widget| {
                if !vello_widget.focused {
                    vello_widget.hovered = false;
                }
            }),
            On::<WidgetFocus>::target_component_mut::<VelloWidget>(|_event, vello_widget| {
                vello_widget.focused = true;
                vello_widget.hovered = false;
            }),
            On::<WidgetBlur>::target_component_mut::<VelloWidget>(|_event, vello_widget| {
                vello_widget.focused = false;
                vello_widget.hovered = false;
            }),
            On::<WidgetKeyboardEvent>::target_component_mut::<VelloWidget>(
                |event, _vello_widget| info!("Widget {} got key {}!", event.target, event.c),
            ),
        ));
    }
    children.process(entity.as_parent());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(DebugPickingMode::Normal)
        .register_widget::<VelloWidget>()
        .add_systems(Startup, startup)
        .add_widget_systems(VelloWidget::get_name(), vello_update, vello_render)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn(Camera2dBundle::default());

    let root = commands
        .spawn((WoodpeckerAppBundle {
            ..Default::default()
        },))
        .id();

    let mut root_children = WidgetChildren::default();

    root_children.add::<VelloWidget>((
        WidgetRender::Quad {
            color: Color::srgba(0.0, 0.0, 1.0, 1.0),
            border_radius: woodpecker_ui::prelude::kurbo::RoundedRectRadii::from_single_radius(
                50.0,
            ),
        },
        VelloWidget::default(),
        WoodpeckerStyle {
            align_content: Some(WidgetAlignContent::SpaceEvenly),
            justify_content: Some(WidgetAlignContent::SpaceEvenly),
            margin: Edge::all(50.0.into()),
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            ..Default::default()
        },
        SpatialBundle::default(),
        WidgetChildren::default(),
    ));

    commands.entity(root).insert(root_children);

    ui_context.set_root_widget(root);
}
