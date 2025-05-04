use bevy::prelude::*;
use bevy_vello::render::VelloView;
use woodpecker_ui::prelude::*;

#[derive(Component, Clone, Default, Debug, Copy, PartialEq)]
pub struct MyWidgetState {
    show_modal: bool,
}

#[derive(Widget, Component, Clone, Default, Reflect, Copy, PartialEq)]
#[auto_update(render)]
#[props(MyWidget)]
#[state(MyWidgetState)]
struct MyWidget;

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
    asset_server: Res<AssetServer>,
) {
    let Ok((_, mut widget_children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        MyWidgetState { show_modal: true },
    );

    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    widget_children
        .add::<WButton>(WButtonBundle {
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
        })
        .observe(
            *current_widget,
            move |_: Trigger<Pointer<Click>>, mut query: Query<&mut MyWidgetState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.show_modal = true;
                }
            },
        );

    widget_children.add::<Modal>(ModalBundle {
        modal: Modal {
            visible: state.show_modal,
            title: "Image/SVG fits to content.".into(),
            ..Default::default()
        },
        children: PassedChildren(
            WidgetChildren::default().with_child::<Element>(ElementBundle {
                styles: WoodpeckerStyle {
                    align_items: Some(WidgetAlignItems::Center),
                    flex_direction: WidgetFlexDirection::Column,
                    padding: Edge::all(10.0),
                    width: Units::Percentage(100.0),
                    ..Default::default()
                },
                children: WidgetChildren::default()
                    .with_child::<Element>((
                        ElementBundle {
                            styles: WoodpeckerStyle {
                                width: Units::Auto,
                                height: Units::Pixels(200.0),
                                margin: Edge::all(0.0).bottom(10.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        WidgetRender::Image {
                            handle: asset_server.load("woodpecker.jpg"),
                        },
                    ))
                    .with_child::<Element>((
                        ElementBundle {
                            styles: WoodpeckerStyle {
                                width: Units::Auto,
                                height: Units::Pixels(200.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        WidgetRender::Svg {
                            handle: asset_server.load("woodpecker_svg/woodpecker.svg"),
                            color: Some(Srgba::RED.into()),
                        },
                    ))
                    .with_child::<WButton>((WButtonBundle {
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
                    },))
                    .with_observe(
                        *current_widget,
                        move |_: Trigger<Pointer<Click>>, mut query: Query<&mut MyWidgetState>| {
                            if let Ok(mut state) = query.get_mut(state_entity) {
                                state.show_modal = false;
                            }
                        },
                    ),
                ..Default::default()
            }),
        ),
        ..Default::default()
    });

    widget_children.apply(current_widget.as_parent());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<MyWidget>()
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, VelloView));

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
