use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn(Camera2dBundle::default());

    let root = commands
        .spawn(WoodpeckerAppBundle {
            children: WidgetChildren::default().with_child::<Element>((
                ElementBundle {
                    styles: WoodpeckerStyle {
                        font_size: 50.0,
                        color: Srgba::RED.into(),
                        margin: Edge::all(10.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Hello World! I am Woodpecker UI!".into(),
                    word_wrap: false,
                },
            )),
            ..Default::default()
        })
        .id();
    ui_context.set_root_widget(root);
}
