#![warn(missing_docs)]
//!
//! # Woodpecker UI
//!
//! Woodpecker UI is a Bevy ECS driven user interface crate. Its designed to be easy to use and work seamlessly with the bevy game engine.
//!
//! ## Features
//! - ECS **first** UI
//! - Easy to use widget systems
//! - Flexable UI rendering using [vello](https://github.com/linebender/bevy_vello)
//! - [Taffy](https://github.com/DioxusLabs/taffy) layouting
//! - [Cosmic Text](https://github.com/pop-os/cosmic-text) for text layouting
//! - A few helper widgets to get you started
//!
//! ## Example:
//! ```rust
//! fn startup(
//!     mut commands: Commands,
//!     mut ui_context: ResMut<WoodpeckerContext>,
//!     mut meshes: ResMut<Assets<Mesh>>,
//!     mut materials: ResMut<Assets<ColorMaterial>>,
//! ) {
//!     commands.spawn(Camera2dBundle::default());
//!     
//!     let material_red = materials.add(Color::Srgba(Srgba::RED.with_alpha(0.5)));
//!     
//!     commands.spawn(MaterialMesh2dBundle {
//!         mesh: Mesh2dHandle(meshes.add(Circle { radius: 50.0 })),
//!         material: material_red,
//!         transform: Transform::from_xyz(0.0, 0.0, 0.0),
//!         ..default()
//!     });
//!     
//!     let root = commands
//!         .spawn((WoodpeckerAppBundle {
//!             styles: WoodpeckerStyle {
//!                 padding: Edge::all(10.0),
//!                 ..default()
//!             },
//!             children: WidgetChildren::default().with_child::<Slider>(SliderBundle {
//!                 slider: Slider {
//!                     start: 0.0,
//!                     end: 1.0,
//!                     value: 0.5,
//!                 },
//!                 on_changed: On::run(
//!                     |event: Listener<OnChange<SliderChanged>>,
//!                     mut material_assets: ResMut<Assets<ColorMaterial>>,
//!                      query: Query<&Handle<ColorMaterial>>| {
//!                         for material in query.iter() {
//!                             material_assets.get_mut(material).unwrap().color.set_alpha(event.data.value)
//!                         }
//!                     },
//!                 ),
//!                 ..default()
//!             }),
//!             ..default()
//!         },))
//!         .id();
//!     ui_context.set_root_widget(root);
//! }
//!
//! ```
use bevy::{
    asset::embedded_asset, prelude::*, reflect::GetTypeRegistration, render::view::RenderLayers,
};
use bevy_mod_picking::{events::Pointer, prelude::EventListenerPlugin};
use bevy_trait_query::RegisterExt;
use bevy_vello::{
    text::VelloFont, vello::AaConfig, CoordinateSpace, VelloPlugin, VelloSceneBundle,
};
use context::{Widget, WoodpeckerContext};
use entity_mapping::WidgetMapper;
use font::FontManager;
use hook_helper::HookHelper;
use image::ImageManager;
use layout::WoodpeckerLayoutPlugin;
use metrics::WidgetMetrics;
use picking_backend::MouseWheelScroll;
use svg::{SvgAsset, SvgLoader, SvgManager};
use widgets::WoodpeckerUIWidgetPlugin;

mod children;
mod context;
mod entity_mapping;
mod focus;
mod font;
mod hook_helper;
mod image;
mod keyboard_input;
mod layout;
mod metrics;
mod on_change;
mod picking_backend;
mod render;
mod runner;
mod styles;
mod svg;
mod vello_svg;
mod vello_renderer;
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
    pub use crate::on_change::Change;
    pub use crate::render::{WidgetRender, WidgetRenderCustom};
    pub use crate::styles::*;
    pub use crate::widgets::*;
    pub use crate::{
        CurrentWidget, ParentWidget, RenderSettings, WidgetRegisterExt, WoodpeckerUIPlugin,
    };
    pub use bevy_vello::vello;
    pub use bevy_vello::vello::AaConfig;
    pub use woodpecker_ui_macros::*;
}

/// Defines useful render settings
#[derive(Resource, Clone)]
pub struct RenderSettings {
    /// The bevy render layer to use
    pub layer: RenderLayers,
    /// Is antialiased? Default: True
    pub antialiasing: AaConfig,
    /// Use CPU to render. Warning can be slow.
    pub use_cpu: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            layer: Default::default(),
            antialiasing: AaConfig::Area,
            use_cpu: Default::default(),
        }
    }
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
    /// Converts a ParentWidget into a CurrentWidget
    ///
    /// Note: Really just a convince function.
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
    /// Converts a CurrentWidget into a ParentWidget.
    ///
    /// Note: Really just a convince function.
    pub fn as_parent(&self) -> ParentWidget {
        ParentWidget(self.0)
    }
}

/// The Woodpecker UI bevy Plugin
/// Add this to bevy to use.
#[derive(Default)]
pub struct WoodpeckerUIPlugin {
    /// The render settings
    /// These settings are used to tell bevy_vello how to render.
    pub render_settings: RenderSettings,
}

impl Plugin for WoodpeckerUIPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "embedded_assets/Poppins-Regular.ttf");
        embedded_asset!(app, "embedded_assets/icons/arrow-down.svg");
        embedded_asset!(app, "embedded_assets/icons/arrow-up.svg");
        embedded_asset!(app, "embedded_assets/icons/checkmark.svg");
        embedded_asset!(app, "embedded_assets/icons/copy-outline.svg");
        app.add_plugins(WoodpeckerLayoutPlugin)
            .add_plugins(VelloPlugin {
                canvas_render_layers: self.render_settings.layer.clone(),
                use_cpu: self.render_settings.use_cpu,
                antialiasing: self.render_settings.antialiasing,
            })
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
            .init_resource::<SvgManager>()
            .init_resource::<ImageManager>()
            .insert_resource(self.render_settings.clone())
            .init_asset::<SvgAsset>()
            .init_asset_loader::<SvgLoader>()
            .add_systems(
                Update,
                (
                    runner::system,
                    focus::CurrentFocus::click_focus,
                    #[cfg(not(target_arch = "wasm32"))]
                    keyboard_input::runner,
                    #[cfg(target_arch = "wasm32")]
                    (keyboard_input::runner, keyboard_input::read_paste_events).chain(),
                    hook_helper::HookHelper::update_context_helper,
                    vello_renderer::run.after(layout::system::run),
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
                    metrics::WidgetMetrics::print_metrics_x_seconds,
                ),
            )
            .add_systems(Startup, startup)
            // Reflection registration
            .register_type::<render::WidgetRender>()
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
            .register_type::<styles::WidgetPosition>()
            .register_type::<Option<styles::WidgetAlignContent>>()
            .register_type::<Option<styles::WidgetAlignItems>>()
            .register_type::<Option<styles::WidgetDisplay>>()
            .register_type::<Option<styles::WidgetFlexDirection>>()
            .register_type::<Option<styles::WidgetFlexWrap>>()
            .register_type::<Option<styles::WidgetOverflow>>()
            .register_type::<Option<styles::WidgetPosition>>();
    }
}

fn has_root() -> impl Condition<(), ()> {
    IntoSystem::into_system(|context: Res<WoodpeckerContext>| context.root_widget.is_some())
}

fn startup(mut commands: Commands, render_settings: Res<RenderSettings>) {
    commands.spawn((
        VelloSceneBundle {
            coordinate_space: CoordinateSpace::ScreenSpace,
            transform: Transform::from_xyz(0.0, 0.0, f32::MAX),
            ..Default::default()
        },
        render_settings.layer.clone(),
    ));
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
