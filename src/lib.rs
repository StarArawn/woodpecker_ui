use bevy::{asset::embedded_asset, prelude::*};
use bevy_mod_picking::prelude::EventListenerPlugin;
use bevy_trait_query::RegisterExt;
use bevy_vello::{text::VelloFont, CoordinateSpace, VelloPlugin, VelloSceneBundle};
use context::{Widget, WoodpeckerContext};
use context_helper::ContextHelper;
use entity_mapping::WidgetMapper;
use layout::WoodpeckerLayoutPlugin;
use widgets::WoodpeckerUIWidgetPlugin;

mod children;
mod context;
mod context_helper;
mod entity_mapping;
mod focus;
mod font;
mod keyboard_input;
mod layout;
mod picking_backend;
mod render;
mod runner;
mod styles;
mod widgets;

/// A module that exports all publicly exposed types.
pub mod prelude {
    pub use crate::children::WidgetChildren;
    pub use crate::context::*;
    pub use crate::entity_mapping::*;
    pub use crate::focus::*;
    pub use crate::keyboard_input::WidgetKeyboardCharEvent;
    pub use crate::render::WidgetRender;
    pub use crate::styles::*;
    pub use crate::widgets::*;
    pub use crate::{CurrentWidget, ParentWidget};
    pub use crate::{WidgetRegisterExt, WoodpeckerUIPlugin};
    pub use bevy_vello::prelude::*;
    pub use woodpecker_ui_macros::*;
}

/// A bevy resource used as the default font.
#[derive(Resource)]
pub struct DefaultFont(pub Handle<VelloFont>);

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
        embedded_asset!(app, "embedded_assets/Poppins-Regular.ttf");
        app.add_plugins(WoodpeckerLayoutPlugin)
            .add_plugins(VelloPlugin)
            .add_plugins(WoodpeckerUIWidgetPlugin)
            .add_plugins(EventListenerPlugin::<focus::WidgetFocus>::default())
            .add_plugins(EventListenerPlugin::<focus::WidgetBlur>::default())
            .add_plugins(EventListenerPlugin::<keyboard_input::WidgetKeyboardCharEvent>::default())
            .add_plugins(EventListenerPlugin::<
                keyboard_input::WidgetKeyboardButtonEvent,
            >::default())
            .insert_resource(focus::CurrentFocus::new(Entity::PLACEHOLDER))
            .init_resource::<ContextHelper>()
            .init_resource::<WoodpeckerContext>()
            .init_resource::<WidgetMapper>()
            .add_systems(
                Update,
                (
                    runner::system,
                    focus::CurrentFocus::click_focus,
                    keyboard_input::runner,
                    context_helper::ContextHelper::update_context_helper,
                )
                    .run_if(has_root()),
            )
            .add_systems(
                Update,
                picking_backend::system.after(crate::layout::system::run),
            )
            .add_systems(Startup, startup);
    }
}

fn has_root() -> impl Condition<(), ()> {
    IntoSystem::into_system(|context: Res<WoodpeckerContext>| context.root_widget.is_some())
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(VelloSceneBundle {
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..Default::default()
    });

    // TODO: Run this before startup..
    commands.insert_resource(DefaultFont(
        asset_server.load("embedded://woodpecker_ui/embedded_assets/Poppins-Regular.ttf"),
    ))
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
        let mut context = self
            .world_mut()
            .get_resource_or_insert_with::<WoodpeckerContext>(WoodpeckerContext::default);
        context.add_widget_systems_non_into(
            T::get_name(),
            Box::new(T::update()),
            Box::new(T::render()),
        );
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
            .get_resource_or_insert_with::<WoodpeckerContext>(WoodpeckerContext::default);
        context.add_widget_system(widget_name, update, render);
        self
    }
}

mod test_proc_macro {
    use crate::prelude::Widget;
    #[derive(Widget)]
    #[widget_systems(update, render)]
    pub struct MyStruct {}

    fn update() -> bool {
        false
    }
    fn render() {}

    #[test]
    fn test_widget_macro() {}
}
