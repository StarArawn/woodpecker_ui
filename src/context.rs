use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

#[bevy_trait_query::queryable]
pub trait Widget {
    fn get_name() -> String
    where
        Self: Sized,
    {
        std::any::type_name::<Self>().into()
    }

    fn get_name_local(&self) -> String {
        std::any::type_name::<Self>().into()
    }
}

type WidgetSystems = HashMap<
    String,
    (
        Box<dyn System<In = (), Out = bool>>,
        Box<dyn System<In = (), Out = ()>>,
    ),
>;

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

    /// Tells Woodpecker UI which entity is the root entity.
    /// This is mostly used so we can traverse the bevy hierarchy
    /// for layouting and rendering.
    pub fn set_root_widget(&mut self, root_widget: Entity) {
        self.root_widget = Some(root_widget);
    }

    /// Gets the root entity
    pub fn get_root_widget(&self) -> Entity {
        self.root_widget.expect("You must set a root entity!")
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
