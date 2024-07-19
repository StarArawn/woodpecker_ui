use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use woodpecker_ui::prelude::*;

#[derive(Component, Clone)]
pub struct FooWidget;
impl Widget for FooWidget {}

fn foo_update(
    entity: Res<CurrentWidget>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, Changed<FooWidget>>,
) -> bool {
    keyboard_input.just_pressed(KeyCode::Space) || query.contains(**entity)
}

fn foo_render(
    mut commands: Commands,
    entity: Res<CurrentWidget>,
    mut is_a: Local<bool>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        *is_a = !*is_a;
    }
    let mut foo_children = WidgetChildren::default();

    // In this A vs B test we have two trees:
    // A: foo > (bar, bar) > bar: (baz, baz)
    // B: foo > (bar) > bar: (baz, baz)
    // This results in 3 widgets being spawned or despawned when we
    // swap between a or b. However the first Bar widget stays the same.
    // Because it has not changed!
    if *is_a {
        info!("I am A!");
        foo_children.add::<BarWidget>(BarWidget);
        foo_children.add::<BarWidget>(BarWidget);
    } else {
        info!("I am B!");
        foo_children.add::<BarWidget>(BarWidget);
    }

    // We tell the widget system runner that the children should be processed at this widget.
    foo_children.process(entity.as_parent());
    // Don't forget to add to the entity as a component!
    commands.entity(**entity).insert(foo_children);
}

#[derive(Component, Clone)]
pub struct BarWidget;
impl Widget for BarWidget {}

fn bar_update(entity: Res<CurrentWidget>, query: Query<Entity, Changed<BarWidget>>) -> bool {
    query.contains(**entity)
}

fn bar_render(mut commands: Commands, entity: Res<CurrentWidget>) {
    info!("I am bar! {:?}", entity);

    let mut bar_children = WidgetChildren::default();

    bar_children.add::<BazWidget>(BazWidget);
    bar_children.add::<BazWidget>(BazWidget);

    // We tell the widget system runner that the children should be processed at this widget.
    bar_children.process(entity.as_parent());
    // Don't forget to add to the entity as a component!
    commands.entity(**entity).insert(bar_children);
}

#[derive(Component, Clone)]
pub struct BazWidget;
impl Widget for BazWidget {}

fn baz_update(entity: Res<CurrentWidget>, query: Query<Entity, Changed<BazWidget>>) -> bool {
    query.contains(**entity)
}

fn baz_render(entity: Res<CurrentWidget>) {
    info!("I am baz! {:?}", entity);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .register_widget::<FooWidget>()
        .register_widget::<BarWidget>()
        .register_widget::<BazWidget>()
        .add_systems(Startup, startup)
        .add_widget_systems(FooWidget::get_name(), foo_update, foo_render)
        .add_widget_systems(BarWidget::get_name(), bar_update, bar_render)
        .add_widget_systems(BazWidget::get_name(), baz_update, baz_render)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    let root = commands.spawn(FooWidget).id();
    ui_context.set_root_widget(root);
}
