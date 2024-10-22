use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
    DefaultPickingPlugins,
};
use calc::Context;
use woodpecker_ui::prelude::*;

const FONT_SIZE: f32 = 60.0;
const WIDTH: f32 = 500.;
const BUTTON_SIZE: f32 = WIDTH / 5.;
const GAP: f32 = BUTTON_SIZE / 5.;
const HEIGHT: f32 = BUTTON_SIZE * 5. + GAP * 6.;

#[rustfmt::skip]
fn get_buttons() -> [&'static str; 16] {
    [
        "7", "8", "9", "/",
        "4", "5", "6", "*",
        "1", "2", "3", "-",
        "0", ".", "=", "+",
    ]
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, startup)
        .insert_resource(CalcOutput("".into()))
        .register_widget::<Output>()
        .add_widget_systems(Output::get_name(), update, render)
        .run();
}

pub const BUTTON_STYLES: WoodpeckerStyle = WoodpeckerStyle {
    background_color: Color::Srgba(Srgba::rgb(0.871, 0.192, 0.38)),
    width: Units::Pixels(BUTTON_SIZE),
    height: Units::Pixels(BUTTON_SIZE),
    justify_content: Some(WidgetAlignContent::Center),
    align_items: Some(WidgetAlignItems::Center),
    ..WoodpeckerStyle::DEFAULT
};

pub const BUTTON_STYLES_HOVER: WoodpeckerStyle = WoodpeckerStyle {
    background_color: Color::Srgba(Srgba::rgb(0.702, 0.0, 0.2)),
    ..BUTTON_STYLES
};

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn(Camera2dBundle::default());

    let mut buttons = WidgetChildren::default();

    // Clear button
    buttons.add::<WButton>((
        WButtonBundle {
            button_styles: ButtonStyles {
                normal: BUTTON_STYLES,
                hovered: BUTTON_STYLES_HOVER,
            },
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: FONT_SIZE,
                        color: Color::WHITE,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "C".into(),
                    word_wrap: false,
                },
            )),
            ..Default::default()
        },
        On::<Pointer<Click>>::run(|mut calc_output: ResMut<CalcOutput>| {
            calc_output.0 = "".into();
        }),
    ));

    // Text box
    buttons.add::<Element>((
        ElementBundle {
            styles: WoodpeckerStyle {
                width: (BUTTON_SIZE * 3. + GAP * 2.).into(),
                height: BUTTON_SIZE.into(),
                background_color: Srgba::hex("DE3161").unwrap().into(),
                border_radius: Corner::all(Units::Pixels(5.0)),
                ..Default::default()
            },
            children: WidgetChildren::default().with_child::<Clip>(ClipBundle {
                styles: WoodpeckerStyle {
                    align_items: Some(WidgetAlignItems::Center),
                    ..ClipBundle::default().styles
                },
                children: WidgetChildren::default().with_child::<Output>((
                    Output,
                    WoodpeckerStyle {
                        margin: Edge::new(0.0, 0.0, 0.0, 15.0),
                        font_size: FONT_SIZE,
                        color: Color::WHITE,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "".into(),
                        word_wrap: false,
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        },
        WidgetRender::Quad,
    ));

    for button in get_buttons() {
        buttons.add::<WButton>((
            WButtonBundle {
                button_styles: ButtonStyles {
                    normal: BUTTON_STYLES,
                    hovered: BUTTON_STYLES_HOVER,
                },
                children: WidgetChildren::default().with_child::<Element>((
                    ElementBundle {
                        styles: WoodpeckerStyle {
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: button.into(),
                        word_wrap: true,
                    },
                )),
                ..Default::default()
            },
            On::<Pointer<Click>>::run(move |mut calc_output: ResMut<CalcOutput>| {
                if button == "=" {
                    if let Ok(result) = Context::<f64>::default().evaluate(&calc_output.0) {
                        calc_output.0 = result.to_string();
                    }
                } else {
                    calc_output.0 += button;
                }
            }),
        ));
    }

    let root = commands
        .spawn(WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<Element>(ElementBundle {
                styles: WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    height: Units::Percentage(100.0),
                    justify_content: Some(WidgetAlignContent::Center),
                    align_content: Some(WidgetAlignContent::Center),
                    padding: Edge {
                        left: 0.0.into(),
                        right: 0.0.into(),
                        top: 25.0.into(),
                        bottom: 0.0.into(),
                    },
                    ..Default::default()
                },
                children: WidgetChildren::default().with_child::<Element>((
                    ElementBundle {
                        styles: WoodpeckerStyle {
                            background_color: Srgba::hex("FF007F").unwrap().into(),
                            border_radius: Corner::all(Units::Pixels(5.0)),
                            width: WIDTH.into(),
                            height: HEIGHT.into(),
                            gap: (GAP.into(), GAP.into()),
                            justify_content: Some(WidgetAlignContent::Center),
                            align_content: Some(WidgetAlignContent::Center),
                            flex_wrap: WidgetFlexWrap::Wrap,
                            ..Default::default()
                        },
                        children: buttons,
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                )),
                ..Default::default()
            }),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}

#[derive(Debug, Resource)]
pub struct CalcOutput(pub String);

#[derive(Component, Reflect, Clone)]
pub struct Output;
impl Widget for Output {}

fn update(output: Res<CalcOutput>) -> bool {
    output.is_changed()
}

fn render(
    current_entity: Res<CurrentWidget>,
    output: Res<CalcOutput>,
    mut query: Query<&mut WidgetRender>,
) {
    let Ok(mut render) = query.get_mut(**current_entity) else {
        return;
    };

    match &mut *render {
        WidgetRender::Text { content, .. } => *content = output.0.clone(),
        _ => {}
    }
}
