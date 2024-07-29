use bevy::{asset::embedded_asset, prelude::*, reflect::GetTypeRegistration};
use bevy_mod_picking::{events::Pointer, prelude::EventListenerPlugin};
use bevy_trait_query::RegisterExt;
use bevy_vello::{text::VelloFont, CoordinateSpace, VelloPlugin, VelloSceneBundle};
use context::{Widget, WoodpeckerContext};
use entity_mapping::WidgetMapper;
use font::FontManager;
use hook_helper::HookHelper;
use layout::WoodpeckerLayoutPlugin;
use metrics::WidgetMetrics;
use picking_backend::MouseWheelScroll;
use widgets::WoodpeckerUIWidgetPlugin;

mod children;
mod context;
mod entity_mapping;
mod focus;
mod font;
mod hook_helper;
mod keyboard_input;
mod layout;
mod metrics;
mod on_change;
mod picking_backend;
mod render;
mod runner;
mod styles;
mod widgets;

/// A module that exports all publicly exposed types.
pub mod prelude {
    pub use crate::children::{Mounted, PassedChildren, WidgetChildren};
    pub use crate::context::*;
    pub use crate::entity_mapping::*;
    pub use crate::focus::*;
    pub use crate::font::{FontManager, TextAlign};
    pub use crate::hook_helper::{HookHelper, PreviousWidget};
    pub use crate::keyboard_input::WidgetKeyboardCharEvent;
    pub use crate::layout::system::{WidgetLayout, WidgetPreviousLayout};
    pub use crate::metrics::WidgetMetrics;
    pub use crate::on_change::OnChange;
    pub use crate::render::WidgetRender;
    pub use crate::styles::*;
    pub use crate::widgets::*;
    pub use crate::{CurrentWidget, ParentWidget};
    pub use crate::{WidgetRegisterExt, WoodpeckerUIPlugin};
    pub use woodpecker_ui_macros::*;
}

/// A bevy resource used as the default font.
#[derive(Resource)]
pub struct DefaultFont(pub Handle<VelloFont>);

impl FromWorld for DefaultFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        DefaultFont(
            asset_server.load("embedded://woodpecker_ui/embedded_assets/Poppins-Regular.ttf"),
        )
    }
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
            .add_plugins(EventListenerPlugin::<Pointer<MouseWheelScroll>>::default())
            .add_plugins(EventListenerPlugin::<keyboard_input::WidgetPasteEvent>::default())
            .insert_resource(focus::CurrentFocus::new(Entity::PLACEHOLDER))
            .init_resource::<FontManager>()
            .init_resource::<HookHelper>()
            .init_resource::<WoodpeckerContext>()
            .init_resource::<WidgetMapper>()
            .init_resource::<DefaultFont>()
            .init_resource::<WidgetMetrics>()
            .add_systems(
                Update,
                (
                    runner::system,
                    focus::CurrentFocus::click_focus,
                    keyboard_input::runner,
                    hook_helper::HookHelper::update_context_helper,
                )
                    .run_if(has_root()),
            )
            .add_systems(
                Update,
                (
                    font::load_fonts,
                    picking_backend::mouse_wheel_system,
                    picking_backend::system.after(crate::layout::system::run),
                    #[cfg(feature = "metrics")]
                    metrics::WidgetMetrics::print_metrics_x_seconds
                ),
            )
            .add_systems(Startup, startup)
            // Reflection registration
            .register_type::<crate::prelude::WidgetLayout>()
            .register_type::<styles::WoodpeckerStyle>()
            .register_type::<styles::Corner>()
            .register_type::<styles::Edge>()
            .register_type::<styles::Units>()
            .register_type::<styles::WidgetAlignContent>()
            .register_type::<styles::WidgetAlignItems>()
            .register_type::<styles::WidgetDisplay>()
            .register_type::<styles::WidgetFlexDirection>()
            .register_type::<styles::WidgetFlexWrap>()
            .register_type::<styles::WidgetOverflow>()
            .register_type::<styles::WidgetPosition>();
    }
}

fn has_root() -> impl Condition<(), ()> {
    IntoSystem::into_system(|context: Res<WoodpeckerContext>| context.root_widget.is_some())
}

fn startup(mut commands: Commands) {
    commands.spawn(VelloSceneBundle {
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..Default::default()
    });
}

/// A trait that gives us some extra functionality for register widgets
/// in bevy.
pub trait WidgetRegisterExt {
    /// Registers a new widget
    /// This tells bevy-trait-query that this is a component, don't do it twice.
    fn register_widget<T: Component + Widget + GetTypeRegistration>(&mut self) -> &mut Self;

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
    fn register_widget<T: Component + Widget + GetTypeRegistration>(&mut self) -> &mut Self {
        self.register_component_as::<dyn Widget, T>();
        self.register_type::<T>();
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
    #[test]
    fn test_widget_macro() {
        use crate::prelude::*;
        use bevy::prelude::*;

        #[derive(Widget)]
        #[widget_systems(update, render)]
        pub struct MyStruct {}

        fn update() -> bool {
            false
        }

        #[derive(Widget, Component, PartialEq, Clone)]
        #[auto_update(render)]
        #[props(MyStruct2)]
        pub struct MyStruct2 {}

        fn render() {}
    }
}
