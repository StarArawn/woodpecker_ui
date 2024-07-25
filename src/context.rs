use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

/// A trait used to mark an entity as a widget.
#[bevy_trait_query::queryable]
pub trait Widget {
    /// Gets the type name of the widget.
    fn get_name() -> String
    where
        Self: Sized,
    {
        std::any::type_name::<Self>().into()
    }

    /// Same as the [`T::get_name`] just does it on &self.
    fn get_name_local(&self) -> String {
        std::any::type_name::<Self>().into()
    }

    /// Creates the update widget system.
    fn update() -> impl System<In = (), Out = bool>
    where
        Self: Sized,
    {
        IntoSystem::into_system(default_update)
    }

    /// Creates the Render widget system.
    fn render() -> impl System<In = (), Out = ()>
    where
        Self: Sized,
    {
        IntoSystem::into_system(default_render)
    }
}

fn default_update() -> bool {
    false
}
fn default_render() {}

type WidgetSystems = HashMap<
    String,
    (
        Box<dyn System<In = (), Out = bool>>,
        Box<dyn System<In = (), Out = ()>>,
    ),
>;

/// A Woodpecker UI context resource.
/// This primiarily exists to keep track of widget systems
/// and the root widget.
#[derive(Resource, Default, Debug)]
pub struct WoodpeckerContext {
    pub(crate) widgets: WidgetSystems,
    pub(crate) uninitialized_systems: HashSet<String>,
    pub(crate) root_widget: Option<Entity>,
}

impl WoodpeckerContext {
    pub(crate) fn add_widget_system<Params, Params2>(
        &mut self,
        widget_name: String,
        update: impl IntoSystem<(), bool, Params>,
        render: impl IntoSystem<(), (), Params2>,
    ) {
        let update_system = Box::new(IntoSystem::into_system(update));
        let render_system = Box::new(IntoSystem::into_system(render));
        self.widgets
            .insert(widget_name.clone(), (update_system, render_system));
        self.uninitialized_systems.insert(widget_name);
    }

    pub(crate) fn add_widget_systems_non_into(
        &mut self,
        widget_name: String,
        update_system: Box<dyn System<In = (), Out = bool>>,
        render_system: Box<dyn System<In = (), Out = ()>>,
    ) {
        self.widgets
            .insert(widget_name.clone(), (update_system, render_system));
        self.uninitialized_systems.insert(widget_name);
    }

    /// Tells Woodpecker UI which entity is the root entity.
    /// This is mostly used so we can traverse the bevy hierarchy
    /// for layouting and rendering.
    pub fn set_root_widget(&mut self, root_widget: Entity) {
        self.root_widget = Some(root_widget);
    }

    /// Gets the root entity
    pub fn get_root_widget(&self) -> Entity {
        self.root_widget
            .expect("Woodpecker UI: No root node found when requesting a root widget!")
    }

    pub(crate) fn get_update_system(
        &mut self,
        widget_name: String,
    ) -> Option<&mut Box<dyn System<In = (), Out = bool>>> {
        let update = self.widgets.get_mut(&widget_name).map(|(update, _)| update);
        update
    }

    pub(crate) fn get_render_system(
        &mut self,
        widget_name: String,
    ) -> Option<&mut Box<dyn System<In = (), Out = ()>>> {
        let render = self.widgets.get_mut(&widget_name).map(|(_, render)| render);
        render
    }

    pub(crate) fn get_uninitialized(&self, widget_name: String) -> bool {
        self.uninitialized_systems.contains(&widget_name)
    }

    pub(crate) fn remove_uninitialized(&mut self, widget_name: String) {
        self.uninitialized_systems.remove(&widget_name);
    }
}
