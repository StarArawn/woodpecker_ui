use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use woodpecker_ui::prelude::*;

// We can derive widget here and pass in our systems
// passing in the widget_systems is optional and if we don't pass
// them in we need to call `app.add_widget_systems`!
#[derive(Component, Widget, Reflect, Clone)]
#[widget_systems(foo_update, foo_render)]
pub struct FooWidget;

fn foo_update(entity: Res<CurrentWidget>, query: Query<Entity, Changed<FooWidget>>) -> bool {
    query.contains(**entity)
}

fn foo_render(mut commands: Commands, entity: Res<CurrentWidget>) {
    // Handled creating children from bevy bundles.
    // Note: The order of the children is important!
    // You can think of this similar to entity "commands".
    // The actual entities are managed by Woodpecker to make sure the proper
    // hiarchy is setup. It also handles reactivity correctly as well.
    let mut foo_children = WidgetChildren::default();

    // Although not required for this exmaple..
    // We can define children of bar here and pass them down.
    let mut bar_children = WidgetChildren::default();
    bar_children.add::<BazWidget>(BazWidget { value: 3.1459 });

    // Now we can create children of "Foo"
    foo_children.add::<BarWidget>(BarWidgetBundle {
        bar_widget: BarWidget,
        children: bar_children,
    });

    // We tell the widget system runner that the children should be processed at this widget.
    foo_children.apply(entity.as_parent());
    // Don't forget to add to the entity as a component!
    commands.entity(**entity).insert(foo_children);
}

#[derive(Bundle, Default, Clone)]
pub struct BarWidgetBundle {
    pub bar_widget: BarWidget,
    pub children: WidgetChildren,
}

#[derive(Component, Widget, Reflect, Default, Clone)]
#[widget_systems(bar_update, bar_render)]
pub struct BarWidget;

fn bar_update(entity: Res<CurrentWidget>, query: Query<Entity, Changed<BarWidget>>) -> bool {
    query.contains(**entity)
}

fn bar_render(entity: Res<CurrentWidget>, mut query: Query<&mut WidgetChildren>) {
    info!("I am bar! {:?}, I use passed in children!", entity);
    let Ok(mut children) = query.get_mut(**entity) else {
        return;
    };

    // We tell the widget system runner that the children should be processed at this widget.
    // Optionally you can clone the children down the tree and process them at any point in the widget tree.
    children.apply(entity.as_parent());
}

#[derive(Component, Widget, Reflect, Default, Clone)]
#[widget_systems(baz_update, baz_render)]
pub struct BazWidget {
    pub value: f32,
}

fn baz_update(entity: Res<CurrentWidget>, query: Query<Entity, Changed<BazWidget>>) -> bool {
    query.contains(**entity)
}

fn baz_render(entity: Res<CurrentWidget>, query: Query<&BazWidget>) {
    let Ok(baz) = query.get(**entity) else {
        return;
    };
    info!("I am baz! {:?} my value is {:?}", entity, baz.value);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(DefaultPickingPlugins)
        .register_widget::<FooWidget>()
        .register_widget::<BarWidget>()
        .register_widget::<BazWidget>()
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    let root = commands.spawn(FooWidget).id();
    ui_context.set_root_widget(root);
}
