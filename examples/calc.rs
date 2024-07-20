use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
    DefaultPickingPlugins,
};
use calc::Context;
use taffy::style_helpers::{FromLength, FromPercent};
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
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, startup)
        .insert_resource(CalcOutput("".into()))
        .register_widget::<Output>()
        .add_widget_systems(Output::get_name(), update, render)
        .run();
}

pub const BUTTON_STYLES: WoodpeckerStyle = WoodpeckerStyle {
    width: Units::Pixels(BUTTON_SIZE),
    height: Units::Pixels(BUTTON_SIZE),
    justify_content: Some(WidgetAlignContent::Center),
    align_content: Some(WidgetAlignContent::Center),
    ..WoodpeckerStyle::DEFAULT
};

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut buttons = WidgetChildren::default();

    // Clear button
    buttons.add::<WButton>((
        WButtonBundle {
            button_styles: ButtonStyles {
                normal: (Srgba::hex("DE3161").unwrap().into(), BUTTON_STYLES),
                hovered: (Srgba::hex("b30033").unwrap().into(), BUTTON_STYLES),
            },
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        width: FONT_SIZE.into(),
                        height: FONT_SIZE.into(),
                        margin: Edge {
                            left: (FONT_SIZE / 2.0).into(),
                            top: (FONT_SIZE / 2.0).into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                WidgetRender::Text {
                    font: asset_server.load("Poppins/Poppins-Regular.ttf"),
                    size: FONT_SIZE,
                    color: Color::WHITE,
                    alignment: VelloTextAlignment::TopLeft,
                    content: "C".into(),
                    word_wrap: true,
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
                justify_content: Some(WidgetAlignContent::FlexStart),
                align_content: Some(WidgetAlignContent::Center),
                ..Default::default()
            },
            children: WidgetChildren::default().with_child::<Clip>(ClipBundle {
                children: WidgetChildren::default().with_child::<Output>((
                    Output,
                    SpatialBundle::default(),
                    WoodpeckerStyle {
                        width: (BUTTON_SIZE * 3. + GAP * 2.).into(),
                        height: FONT_SIZE.into(),
                        margin: Edge {
                            left: 15.0.into(),
                            top: (FONT_SIZE / 2.0).into(),
                            right: 0.0.into(),
                            bottom: 0.0.into(),
                        },
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        font: asset_server.load("Poppins/Poppins-Regular.ttf"),
                        size: FONT_SIZE,
                        color: Color::WHITE,
                        alignment: VelloTextAlignment::TopLeft,
                        content: "".into(),
                        word_wrap: false,
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        },
        WidgetRender::Quad {
            color: Srgba::hex("DE3161").unwrap().into(),
            border_radius: kurbo::RoundedRectRadii::from_single_radius(5.0),
        },
    ));

    for button in get_buttons() {
        buttons.add::<WButton>((
            WButtonBundle {
                button_styles: ButtonStyles {
                    normal: (Srgba::hex("DE3161").unwrap().into(), BUTTON_STYLES),
                    hovered: (Srgba::hex("b30033").unwrap().into(), BUTTON_STYLES),
                },
                children: WidgetChildren::default().with_child::<Element>((
                    ElementBundle {
                        styles: WoodpeckerStyle {
                            width: FONT_SIZE.into(),
                            height: FONT_SIZE.into(),
                            margin: Edge {
                                left: (FONT_SIZE / 2.0).into(),
                                top: (FONT_SIZE / 2.0).into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        font: asset_server.load("Poppins/Poppins-Regular.ttf"),
                        size: FONT_SIZE,
                        color: Color::WHITE,
                        alignment: VelloTextAlignment::TopLeft,
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
                    calc_output.0 += button.into();
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
                    WidgetRender::Quad {
                        color: Srgba::hex("FF007F").unwrap().into(),
                        border_radius: kurbo::RoundedRectRadii::from_single_radius(5.0),
                    },
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

#[derive(Component, Clone)]
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
