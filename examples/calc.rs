use bevy::prelude::*;
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
        .add_systems(Startup, startup)
        .insert_resource(CalcOutput("".into()))
        .register_widget::<Output>()
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
    commands.spawn((Camera2d, WoodpeckerView));

    let mut buttons = WidgetChildren::default();

    let root = CurrentWidget(commands.spawn_empty().id());

    // Clear button
    buttons
        .add::<WButton>((
            WButton,
            ButtonStyles {
                normal: BUTTON_STYLES,
                hovered: BUTTON_STYLES_HOVER,
            },
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: FONT_SIZE,
                    color: Color::WHITE,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "C".into(),
                },
            )),
        ))
        .observe(
            root,
            |_: On<Pointer<Click>>, mut calc_output: ResMut<CalcOutput>| {
                calc_output.0 = "".into();
            },
        );

    // Text box
    buttons.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: (BUTTON_SIZE * 3. + GAP * 2.).into(),
            height: BUTTON_SIZE.into(),
            background_color: Srgba::hex("DE3161").unwrap().into(),
            border_radius: Corner::all(Units::Pixels(5.0)),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Clip>((
            Clip,
            WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<Output>((
                Output,
                WoodpeckerStyle {
                    margin: Edge::new(0.0, 0.0, 0.0, 15.0),
                    font_size: FONT_SIZE,
                    color: Color::WHITE,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text { content: "".into() },
            )),
        )),
        WidgetRender::Quad,
    ));

    for button in get_buttons() {
        buttons
            .add::<WButton>((
                WButton,
                ButtonStyles {
                    normal: BUTTON_STYLES,
                    hovered: BUTTON_STYLES_HOVER,
                },
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: FONT_SIZE,
                        color: Color::WHITE,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: button.into(),
                    },
                )),
            ))
            .observe(
                root,
                move |_: On<Pointer<Click>>, mut calc_output: ResMut<CalcOutput>| {
                    if button == "=" {
                        if let Ok(result) = Context::<f64>::default().evaluate(&calc_output.0) {
                            calc_output.0 = result.to_string();
                        }
                    } else {
                        calc_output.0 += button;
                    }
                },
            );
    }

    commands.entity(root.entity()).insert((
        WoodpeckerApp,
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
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
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
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
                buttons,
                WidgetRender::Quad,
            )),
        )),
    ));
    ui_context.set_root_widget(root.entity());
}

#[derive(Debug, Resource, PartialEq, Clone)]
pub struct CalcOutput(pub String);

#[derive(Widget, Component, Reflect, Clone, Default, PartialEq)]
#[auto_update(render)]
#[props(Output)]
#[resource(CalcOutput)]
pub struct Output;

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
