use bevy::prelude::*;
use bevy_trait_query::RegisterExt;
use context::{Widget, WoodpeckerContext};
use entity_mapping::WidgetMapper;
use layout::WoodpeckerLayoutPlugin;
use widgets::WoodpeckerUIWidgetPlugin;

mod children;
mod context;
mod entity_mapping;
mod layout;
mod runner;
mod widgets;

/// A module that exports all publicly exposed types.
pub mod prelude {
    pub use crate::children::WidgetChildren;
    pub use crate::context::*;
    pub use crate::entity_mapping::*;
    pub use crate::layout::WoodpeckerStyle;
    pub use crate::widgets::*;
    pub use crate::{CurrentWidget, ParentWidget};
    pub use crate::{WidgetRegisterExt, WoodpeckerUIPlugin};
    pub use taffy::*;
}

/// Wraps an entity and lets woodpecker know its a parent.
#[derive(Resource, Debug, Clone, Copy, Deref, DerefMut, PartialEq, Eq, Hash)]
pub struct ParentWidget(pub Entity);

impl ParentWidget {
    pub fn as_current(&self) -> CurrentWidget {
        CurrentWidget(self.0)
    }
}

/// Wraps an entity and lets woodpecker know this is the current widget entity.
/// Note: This is used to pass the current widget entity to the update and render
/// systems.
#[derive(Resource, Debug, Clone, Copy, Deref, DerefMut, PartialEq, Eq, Hash)]
pub struct CurrentWidget(pub Entity);

impl CurrentWidget {
    pub fn as_parent(&self) -> ParentWidget {
        ParentWidget(self.0)
    }
}

/// The Woodpecker UI bevy Plugin
/// Add this to bevy to use.
pub struct WoodpeckerUIPlugin;
impl Plugin for WoodpeckerUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WoodpeckerLayoutPlugin)
            .add_plugins(WoodpeckerUIWidgetPlugin)
            .init_resource::<WoodpeckerContext>()
            .init_resource::<WidgetMapper>()
            .add_systems(Update, runner::system);
    }
}

pub trait WidgetRegisterExt {
    /// Registers a new widget
    /// This tells bevy-trait-query that this is a component, don't do it twice.
    fn register_widget<T: Component + Widget>(&mut self) -> &mut Self;

    /// Adds a new set of systems for a widget type.
    /// Update systems are ran every frame and return true or false depending on if the widget has "changed".
    /// Render systems are ran only if the widget has changed and are meant to re-render children and handle
    /// tree changes.
    fn add_widget_systems<Params, Params2>(
        &mut self,
        widget_name: String,
        update: impl IntoSystem<(), bool, Params>,
        render: impl IntoSystem<(), (), Params2>,
    ) -> &mut Self;
}

impl WidgetRegisterExt for App {
    fn register_widget<T: Component + Widget>(&mut self) -> &mut Self {
        self.register_component_as::<dyn Widget, T>();
        self
    }

    fn add_widget_systems<Params, Params2>(
        &mut self,
        widget_name: String,
        update: impl IntoSystem<(), bool, Params>,
        render: impl IntoSystem<(), (), Params2>,
    ) -> &mut Self {
        let mut context = self
            .world_mut()
            .get_resource_or_insert_with::<WoodpeckerContext>(|| WoodpeckerContext::default());
        context.add_widget_system(widget_name, update, render);
        self
    }
}
