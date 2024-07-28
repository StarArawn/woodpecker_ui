use bevy::prelude::*;
use bevy_mod_picking::{
    debug::DebugPickingMode,
    events::{Click, Pointer},
    prelude::On,
    DefaultPickingPlugins,
};
use woodpecker_ui::prelude::*;

#[derive(Component, Clone, Default, Debug, Copy, PartialEq)]
pub struct MyWidgetState {
    show_modal: bool,
}

#[derive(Widget, Component, Clone, Default, Reflect, Copy, PartialEq)]
#[auto_update(render)]
#[props(MyWidget)]
#[state(MyWidgetState)]
struct MyWidget {
    depth: usize,
    total: usize,
}

#[derive(Bundle, Default, Clone)]
struct MyWidgetBundle {
    my_widget: MyWidget,
    styles: WoodpeckerStyle,
    children: WidgetChildren,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&MyWidget, &mut WidgetChildren)>,
    state_query: Query<&mut MyWidgetState>,
) {
    let Ok((my_widget, mut widget_children)) = query.get_mut(**current_widget) else {
        return;
    };

    if my_widget.depth == 0 {
        return;
    }

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        MyWidgetState { show_modal: false },
    );

    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

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
                    content: format!("Open Modal {}", my_widget.total - my_widget.depth),
                    word_wrap: false,
                },
            )),
            ..Default::default()
        },
        On::<Pointer<Click>>::run(move |mut query: Query<&mut MyWidgetState>| {
            if let Ok(mut state) = query.get_mut(state_entity) {
                state.show_modal = true;
            }
        }),
    ));

    widget_children.add::<Modal>(ModalBundle {
        modal: Modal {
            visible: state.show_modal,
            title: "I am a modal".into(),
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
                                    content: format!("Close Modal {}", my_widget.total - my_widget.depth),
                                    word_wrap: true,
                                },
                            )),
                            ..Default::default()
                        },
                        On::<Pointer<Click>>::run(move |mut query: Query<&mut MyWidgetState>| {
                            if let Ok(mut state) = query.get_mut(state_entity) {
                                state.show_modal = false;
                            }
                        }),
                    ))
                    .with_child::<MyWidget>(MyWidgetBundle {
                        my_widget: MyWidget { depth: my_widget.depth - 1, total: my_widget.total },
                        styles: WoodpeckerStyle {
                            width: Units::Percentage(100.0),
                            justify_content: Some(WidgetAlignContent::Center),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
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
        .insert_resource(DebugPickingMode::Normal)
        .register_widget::<MyWidget>()
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn(Camera2dBundle::default());

    let number_of_modals = 5;

    let root = commands
        .spawn(WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<MyWidget>(MyWidgetBundle {
                styles: WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    justify_content: Some(WidgetAlignContent::Center),
                    ..Default::default()
                },
                my_widget: MyWidget {
                    depth: number_of_modals,
                    total: number_of_modals,
                },
                ..Default::default()
            }),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
