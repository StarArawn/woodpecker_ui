use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut font_manager: ResMut<FontManager>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, WoodpeckerView));

    let font = asset_server.load("Outfit/static/Outfit-Regular.ttf");
    font_manager.add(&font);

    let root = commands
        .spawn((
            WoodpeckerApp,
            WidgetChildren::default().with_child::<Modal>((
                Modal {
                    visible: true,
                    title: "Code Editor example".into(),
                    min_size: Vec2::new(640.0, 480.0),
                    ..Default::default()
                },
                PassedChildren(
                    WidgetChildren::default().with_child::<ScrollContextProvider>((
                        ScrollContextProvider::default(),
                        WoodpeckerStyle {
                            margin: Edge::all(0.0).left(10.0).right(10.0).bottom(10.0),
                            width: Units::Percentage(100.0),
                            height: Units::Percentage(100.0),
                            ..Default::default()
                        },
                        WidgetChildren::default().with_child::<ScrollBox>((
                            ScrollBox::default(),
                            PassedChildren(WidgetChildren::default().with_child::<TextBox>(
                                TextBox {
                                    initial_value: CODE_BLOCK.to_string(),
                                    multi_line: true,
                                    text_highlighting: ApplyHighlighting::new(|txt| {
                                        Some(highlight("rust", txt, "dracula"))
                                    }),
                                },
                            )),
                        )),
                    )),
                ),
            )),
        ))
        .id();
    ui_context.set_root_widget(root);
}

pub const CODE_BLOCK: &str = r#"use bevy::prelude::*;
use woodpecker_ui::prelude::*;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    mut font_manager: ResMut<FontManager>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, WoodpeckerView));

    let font = asset_server.load("Outfit/static/Outfit-Regular.ttf");
    font_manager.add(&font);

    let root = commands
        .spawn((
            WoodpeckerApp,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 50.0,
                    color: Srgba::RED.into(),
                    margin: Edge::all(10.0),
                    font: Some(font.id()),
                    ..Default::default()
                },
                WidgetRender::RichText {
                    content: RichText::new()
                        .with_color_text("Hello World! ", Srgba::BLUE.into())
                        .with_color_text("I am Woodpecker UI!", Srgba::RED.into()),
                },
            )),
        ))
        .id();
    ui_context.set_root_widget(root);
}
"#;
