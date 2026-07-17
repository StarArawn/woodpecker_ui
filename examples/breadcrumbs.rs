use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            padding: Edge::all(20.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Breadcrumbs>((Breadcrumbs {
                items: vec![
                    "Settings".into(),
                    "Billing".into(),
                    "Invoices".into(),
                    "INV-2049".into(),
                ],
            },))
            .with_key("trail")
            .with_observe(root_widget, |trigger: On<Change<BreadcrumbClicked>>| {
                info!("Breadcrumb index {} clicked", trigger.data.index);
            }),
    ));
}
