use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
    DefaultPickingPlugins,
};
use calc::Context;
use style_helpers::{FromLength, FromPercent};
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

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(VelloSceneBundle {
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..Default::default()
    });

    let mut buttons = WidgetChildren::default();

    // Clear button
    buttons.add::<WButton>((
        WButtonBundle {
            button_styles: ButtonStyles {
                normal: (
                    Srgba::hex("DE3161").unwrap().into(),
                    WoodpeckerStyle::new()
                        .with_size(Size::from_lengths(BUTTON_SIZE, BUTTON_SIZE))
                        .with_justify_content(Some(taffy::AlignContent::Center))
                        .with_align_content(Some(taffy::AlignContent::Center)),
                ),
                hovered: (
                    Srgba::hex("b30033").unwrap().into(),
                    WoodpeckerStyle::new()
                        .with_size(Size::from_lengths(BUTTON_SIZE, BUTTON_SIZE))
                        .with_justify_content(Some(taffy::AlignContent::Center))
                        .with_align_content(Some(taffy::AlignContent::Center)),
                ),
            },
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle::new()
                        .with_size(Size::from_lengths(FONT_SIZE, FONT_SIZE))
                        .with_margin(taffy::Rect {
                            left: LengthPercentageAuto::from_length(FONT_SIZE / 2.0),
                            top: LengthPercentageAuto::from_length(FONT_SIZE / 2.0),
                            right: LengthPercentageAuto::from_length(0.0),
                            bottom: LengthPercentageAuto::from_length(0.0),
                        }),
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
            styles: WoodpeckerStyle::new()
                .with_size(Size::from_lengths(BUTTON_SIZE * 3. + GAP * 2., BUTTON_SIZE))
                .with_justify_content(Some(taffy::AlignContent::FlexStart))
                .with_align_content(Some(taffy::AlignContent::Center)),
            children: WidgetChildren::default().with_child::<Clip>(ClipBundle {
                children: WidgetChildren::default().with_child::<Output>((
                    Output,
                    SpatialBundle::default(),
                    WoodpeckerStyle::new()
                        .with_size(Size::from_lengths(BUTTON_SIZE * 3. + GAP * 2., FONT_SIZE))
                        .with_margin(taffy::Rect {
                            left: LengthPercentageAuto::from_length(15.0),
                            top: LengthPercentageAuto::from_length(FONT_SIZE / 2.0),
                            right: LengthPercentageAuto::from_length(0.0),
                            bottom: LengthPercentageAuto::from_length(0.0),
                        }),
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
                    normal: (
                        Srgba::hex("DE3161").unwrap().into(),
                        WoodpeckerStyle::new()
                            .with_size(Size::from_lengths(BUTTON_SIZE, BUTTON_SIZE))
                            .with_justify_content(Some(taffy::AlignContent::Center))
                            .with_align_content(Some(taffy::AlignContent::Center)),
                    ),
                    hovered: (
                        Srgba::hex("b30033").unwrap().into(),
                        WoodpeckerStyle::new()
                            .with_size(Size::from_lengths(BUTTON_SIZE, BUTTON_SIZE))
                            .with_justify_content(Some(taffy::AlignContent::Center))
                            .with_align_content(Some(taffy::AlignContent::Center)),
                    ),
                },
                children: WidgetChildren::default().with_child::<Element>((
                    ElementBundle {
                        styles: WoodpeckerStyle::new()
                            .with_size(Size::from_lengths(FONT_SIZE, FONT_SIZE))
                            .with_margin(taffy::Rect {
                                left: LengthPercentageAuto::from_length(FONT_SIZE / 2.0),
                                top: LengthPercentageAuto::from_length(FONT_SIZE / 2.0),
                                right: LengthPercentageAuto::from_length(0.0),
                                bottom: LengthPercentageAuto::from_length(0.0),
                            }),
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
                styles: WoodpeckerStyle::new()
                    .with_size(taffy::Size {
                        width: Dimension::from_percent(1.0),
                        height: Dimension::from_percent(1.0),
                    })
                    .with_justify_content(Some(taffy::AlignContent::Center))
                    .with_align_content(Some(taffy::AlignContent::Center))
                    .with_padding(taffy::Rect {
                        left: LengthPercentage::Length(0.0),
                        right: LengthPercentage::Length(0.0),
                        top: LengthPercentage::Length(25.0),
                        bottom: LengthPercentage::Length(0.0),
                    }),
                children: WidgetChildren::default().with_child::<Element>((
                    ElementBundle {
                        styles: WoodpeckerStyle::new()
                            .with_size(Size::from_lengths(WIDTH, HEIGHT))
                            .with_gap(taffy::Size {
                                width: LengthPercentage::from_length(GAP),
                                height: LengthPercentage::from_length(GAP),
                            })
                            .with_justify_content(Some(taffy::AlignContent::Center))
                            .with_align_content(Some(taffy::AlignContent::Center))
                            .with_flex_wrap(taffy::FlexWrap::Wrap),
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
