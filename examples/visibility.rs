use bevy::{color::palettes::tailwind::*, prelude::*};
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<MyWidget>()
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root = commands
        .spawn(WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<MyWidget>(MyWidgetBundle {
                styles: WoodpeckerStyle {
                    display: WidgetDisplay::Flex,
                    flex_direction: WidgetFlexDirection::Column,
                    ..default()
                },
                my_widget: MyWidget,
                ..default()
            }),
            ..default()
        })
        .id();
    ui_context.set_root_widget(root);
}

#[derive(Component, Clone, Default, Debug, Copy, PartialEq)]
pub struct MyWidgetState {
    text: WidgetVisibility,
    image: WidgetVisibility,
    quad: WidgetVisibility,
    svg: WidgetVisibility,
    nine_patch: WidgetVisibility,
    layer: WidgetVisibility,
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
    asset_server: ResMut<AssetServer>,
) {
    let Ok((_, mut widget_children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        MyWidgetState {
            text: WidgetVisibility::Visible,
            image: WidgetVisibility::Visible,
            quad: WidgetVisibility::Visible,
            svg: WidgetVisibility::Visible,
            nine_patch: WidgetVisibility::Visible,
            layer: WidgetVisibility::Visible,
        },
    );

    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    let buttons = WidgetChildren::default()
        .with_child::<WButton>(WButtonBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 20.0,
                        ..default()
                    },
                    ..default()
                },
                WidgetRender::Text {
                    content: "Text".into(),
                    word_wrap: false,
                },
            )),
            ..default()
        })
        .with_observe(
            *current_widget,
            move |_trigger: Trigger<Pointer<Click>>, mut query: Query<&mut MyWidgetState>| {
                let Ok(mut input) = query.get_mut(state_entity) else {
                    return;
                };
                input.text = match input.text {
                    WidgetVisibility::Visible => WidgetVisibility::Hidden,
                    WidgetVisibility::Hidden => WidgetVisibility::Visible,
                };
            },
        )
        .with_child::<WButton>(WButtonBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 20.0,
                        ..default()
                    },
                    ..default()
                },
                WidgetRender::Text {
                    content: "Image".into(),
                    word_wrap: false,
                },
            )),
            ..default()
        })
        .with_observe(
            *current_widget,
            move |_trigger: Trigger<Pointer<Click>>, mut query: Query<&mut MyWidgetState>| {
                let Ok(mut input) = query.get_mut(state_entity) else {
                    return;
                };
                input.image = match input.image {
                    WidgetVisibility::Visible => WidgetVisibility::Hidden,
                    WidgetVisibility::Hidden => WidgetVisibility::Visible,
                };
            },
        )
        .with_child::<WButton>(WButtonBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 20.0,
                        ..default()
                    },
                    ..default()
                },
                WidgetRender::Text {
                    content: "Quad".into(),
                    word_wrap: false,
                },
            )),
            ..default()
        })
        .with_observe(
            *current_widget,
            move |_trigger: Trigger<Pointer<Click>>, mut query: Query<&mut MyWidgetState>| {
                let Ok(mut input) = query.get_mut(state_entity) else {
                    return;
                };
                input.quad = match input.quad {
                    WidgetVisibility::Visible => WidgetVisibility::Hidden,
                    WidgetVisibility::Hidden => WidgetVisibility::Visible,
                };
            },
        )
        .with_child::<WButton>(WButtonBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 20.0,
                        ..default()
                    },
                    ..default()
                },
                WidgetRender::Text {
                    content: "SVG".into(),
                    word_wrap: false,
                },
            )),
            ..default()
        })
        .with_observe(
            *current_widget,
            move |_trigger: Trigger<Pointer<Click>>, mut query: Query<&mut MyWidgetState>| {
                let Ok(mut input) = query.get_mut(state_entity) else {
                    return;
                };
                input.svg = match input.svg {
                    WidgetVisibility::Visible => WidgetVisibility::Hidden,
                    WidgetVisibility::Hidden => WidgetVisibility::Visible,
                };
            },
        )
        .with_child::<WButton>(WButtonBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 20.0,
                        ..default()
                    },
                    ..default()
                },
                WidgetRender::Text {
                    content: "nine_patch".into(),
                    word_wrap: false,
                },
            )),
            ..default()
        })
        .with_observe(
            *current_widget,
            move |_trigger: Trigger<Pointer<Click>>, mut query: Query<&mut MyWidgetState>| {
                let Ok(mut input) = query.get_mut(state_entity) else {
                    return;
                };
                input.nine_patch = match input.nine_patch {
                    WidgetVisibility::Visible => WidgetVisibility::Hidden,
                    WidgetVisibility::Hidden => WidgetVisibility::Visible,
                };
            },
        )
        .with_child::<WButton>(WButtonBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 20.0,
                        ..default()
                    },
                    ..default()
                },
                WidgetRender::Text {
                    content: "layer".into(),
                    word_wrap: false,
                },
            )),
            ..default()
        })
        .with_observe(
            *current_widget,
            move |_trigger: Trigger<Pointer<Click>>, mut query: Query<&mut MyWidgetState>| {
                let Ok(mut input) = query.get_mut(state_entity) else {
                    return;
                };
                input.layer = match input.layer {
                    WidgetVisibility::Visible => WidgetVisibility::Hidden,
                    WidgetVisibility::Hidden => WidgetVisibility::Visible,
                };
            },
        );

    widget_children
        .add::<Element>((ElementBundle {
            styles: WoodpeckerStyle {
                margin: Edge::all(10.0),
                display: WidgetDisplay::Flex,
                gap: (Units::Pixels(5.), Units::Pixels(5.)),
                width: Units::Pixels(100.),
                ..default()
            },
            children: buttons,
            ..default()
        },))
        .add::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    visibility: state.text,
                    font_size: 50.0,
                    color: Srgba::RED.into(),
                    margin: Edge::all(10.0),
                    ..default()
                },
                ..default()
            },
            WidgetRender::Text {
                content: "Hello World! I am Woodpecker UI!".into(),
                word_wrap: false,
            },
            Pickable::default(),
        ))
        .observe(*current_widget, |_trigger: Trigger<Pointer<Click>>| {
            info!("Clicked!");
        })
        .add::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    visibility: state.image,
                    height: Units::Pixels(100.),
                    ..default()
                },
                ..default()
            },
            WidgetRender::Image {
                handle: asset_server.load("woodpecker.jpg"),
            },
            Pickable::default(),
        ))
        .observe(*current_widget, |_trigger: Trigger<Pointer<Click>>| {
            info!("Clicked!");
        })
        .add::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    visibility: state.quad,
                    height: Units::Pixels(100.),
                    background_color: BLUE_400.into(),
                    ..default()
                },
                ..default()
            },
            WidgetRender::Quad,
            Pickable::default(),
        ))
        .observe(*current_widget, |_trigger: Trigger<Pointer<Click>>| {
            info!("Clicked!");
        })
        .add::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    visibility: state.svg,
                    height: Units::Pixels(100.),
                    ..default()
                },
                ..default()
            },
            WidgetRender::Svg {
                handle: asset_server.load("woodpecker_svg/woodpecker.svg"),
                color: Some(Srgba::GREEN.into()),
            },
            Pickable::default(),
        ))
        .observe(*current_widget, |_trigger: Trigger<Pointer<Click>>| {
            info!("Clicked!");
        })
        .add::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    visibility: state.nine_patch,
                    width: 100.0.into(),
                    height: 200.0.into(),
                    ..default()
                },
                ..default()
            },
            WidgetRender::NinePatch {
                handle: asset_server.load("slice.png"),
                scale_mode: SpriteImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(135.),
                    center_scale_mode: SliceScaleMode::Stretch,
                    ..default()
                }),
            },
            Pickable::default(),
        ))
        .observe(*current_widget, |_trigger: Trigger<Pointer<Click>>| {
            info!("Clicked!");
        })
        .add::<Element>((
            ElementBundle {
                styles: WoodpeckerStyle {
                    visibility: state.layer,
                    width: 100.0.into(),
                    height: 50.0.into(),
                    ..default()
                },
                children: WidgetChildren::default().with_child::<Element>((
                    ElementBundle {
                        styles: WoodpeckerStyle {
                            height: Units::Pixels(100.),
                            ..default()
                        },
                        ..default()
                    },
                    WidgetRender::Svg {
                        handle: asset_server.load("woodpecker_svg/woodpecker.svg"),
                        color: Some(Srgba::RED.into()),
                    },
                )),
                ..default()
            },
            WidgetRender::Layer,
            Pickable::default(),
        ))
        .observe(*current_widget, |_trigger: Trigger<Pointer<Click>>| {
            info!("Clicked!");
        });

    widget_children.apply(current_widget.as_parent());
}
