use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
    DefaultPickingPlugins,
};
use woodpecker_ui::prelude::*;

#[derive(Component, Widget, Clone, Default, Copy, PartialEq)]
#[widget_systems(update, render)]
struct MyWidget {
    show_modal: bool,
}

#[derive(Bundle, Default, Clone)]
struct MyWidgetBundle {
    count: MyWidget,
    styles: WoodpeckerStyle,
    children: WidgetChildren,
}

fn update(current_widget: Res<CurrentWidget>, query: Query<Entity, Changed<MyWidget>>) -> bool {
    query.contains(**current_widget)
}

fn render(current_widget: Res<CurrentWidget>, mut query: Query<(&MyWidget, &mut WidgetChildren)>) {
    let Ok((my_widget, mut widget_children)) = query.get_mut(**current_widget) else {
        return;
    };

    let my_widget_entity = **current_widget;
    widget_children.add::<WButton>((
        WButtonBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 20.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Open Modal".into(),
                    word_wrap: false,
                },
            )),
            ..Default::default()
        },
        On::<Pointer<Click>>::run(move |mut query: Query<&mut MyWidget>| {
            if let Ok(mut my_widget) = query.get_mut(my_widget_entity) {
                my_widget.show_modal = true;
            }
        }),
    ));

    widget_children.add::<Modal>(ModalBundle {
        modal: Modal {
            visible: my_widget.show_modal,
            title: "I am a modal".into(),
            overlay_alpha: 0.85,
            ..Default::default()
        },
        children: PassedChildren(
            WidgetChildren::default()
                .with_child::<Element>(ElementBundle {
                    styles: WoodpeckerStyle {
                        align_items: Some(WidgetAlignItems::Center),
                        flex_direction: WidgetFlexDirection::Column,
                        padding: Edge::all(10.0),
                        width: Units::Percentage(100.0),
                        ..Default::default()
                    },
                    children: WidgetChildren::default().with_child::<Element>((
                        ElementBundle {
                            styles: WoodpeckerStyle {
                                font_size: 20.0,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content:
                                "Hello World! I am Woodpecker UI! This is an example of a modal window!"
                                    .into(),
                            word_wrap: true,
                        },
                    ))
                    .with_child::<WButton>((
                        WButtonBundle {
                            children: WidgetChildren::default().with_child::<Element>((
                                ElementBundle {
                                    styles: WoodpeckerStyle {
                                        width: Units::Percentage(100.0),
                                        font_size: 20.0,
                                        text_alignment: Some(TextAlign::Center),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                WidgetRender::Text {
                                    content: "Close Modal".into(),
                                    word_wrap: true,
                                },
                            )),
                            ..Default::default()
                        },
                        On::<Pointer<Click>>::run(move |mut query: Query<&mut MyWidget>| {
                            if let Ok(mut my_widget) = query.get_mut(my_widget_entity) {
                                my_widget.show_modal = false;
                            }
                        }),
                    )),
                    ..Default::default()
                })
        ),
        ..Default::default()
    });

    widget_children.apply(current_widget.as_parent());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .register_widget::<MyWidget>()
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn(Camera2dBundle::default());

    let root = commands
        .spawn(WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<MyWidget>(MyWidgetBundle {
                styles: WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    justify_content: Some(WidgetAlignContent::Center),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
