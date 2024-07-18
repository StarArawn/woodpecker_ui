use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use woodpecker_ui::prelude::*;

#[derive(Component, Clone)]
pub struct SpriteWidget;
impl Widget for SpriteWidget {}

fn sprite_update(entity: Res<CurrentWidget>, query: Query<Entity, Changed<SpriteWidget>>) -> bool {
    query.contains(**entity)
}

// Nothing to do..
fn sprite_render() {}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .register_widget::<SpriteWidget>()
        .add_systems(Startup, startup)
        .add_widget_systems(SpriteWidget::get_name(), sprite_update, sprite_render)
        .run();
}

fn startup(
    mut commands: Commands,
    mut ui_context: ResMut<WoodpeckerContext>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut children = WidgetChildren::default();

    for _ in 0..5 {
        children.add::<SpriteWidget>((
            SpriteWidget,
            WoodpeckerStyle::default()
                .with_size(taffy::Size::from_lengths(64.0, 64.0))
                .with_position(taffy::Position::Relative),
            SpriteBundle {
                texture: asset_server.load("woodpecker.jpg"),
                ..Default::default()
            },
        ));
    }

    let root = commands
        .spawn(WoodpeckerAppBundle {
            styles: WoodpeckerStyle::new().with_display(taffy::Display::Flex),
            ..Default::default()
        })
        .insert(SpriteBundle {
            texture: asset_server.load("woodpecker.jpg"),
            ..Default::default()
        })
        .id();

    children.process(ParentWidget(root));
    commands.entity(root).insert(children);

    ui_context.set_root_widget(root);
}
