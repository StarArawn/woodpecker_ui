use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use woodpecker_ui::prelude::*;

#[derive(Component, Clone)]
pub struct FooWidget;
impl Widget for FooWidget {
    // These are optional trait implementaions for passing in systems.
    // The `register_widget` function can see these and automatically
    // add the widget systems. Optionally you can also use:
    // `app.add_widget_systems` if desired.
    fn update() -> impl System<In = (), Out = bool>
    where
        Self: Sized,
    {
        IntoSystem::into_system(foo_update)
    }

    fn render() -> impl System<In = (), Out = ()>
    where
        Self: Sized,
    {
        IntoSystem::into_system(foo_render)
    }
}

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
    foo_children.process(entity.as_parent());
    // Don't forget to add to the entity as a component!
    commands.entity(**entity).insert(foo_children);
}

#[derive(Bundle, Default, Clone)]
pub struct BarWidgetBundle {
    pub bar_widget: BarWidget,
    pub children: WidgetChildren,
}

#[derive(Component, Default, Clone)]
pub struct BarWidget;
impl Widget for BarWidget {
    fn update() -> impl System<In = (), Out = bool>
    where
        Self: Sized,
    {
        IntoSystem::into_system(bar_update)
    }

    fn render() -> impl System<In = (), Out = ()>
    where
        Self: Sized,
    {
        IntoSystem::into_system(bar_render)
    }
}

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
    children.process(entity.as_parent());
}

#[derive(Component, Clone)]
pub struct BazWidget {
    pub value: f32,
}
impl Widget for BazWidget {
    fn update() -> impl System<In = (), Out = bool>
    where
        Self: Sized,
    {
        IntoSystem::into_system(baz_update)
    }

    fn render() -> impl System<In = (), Out = ()>
    where
        Self: Sized,
    {
        IntoSystem::into_system(baz_render)
    }
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
        .add_plugins(WoodpeckerUIPlugin)
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
