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
//! ## Example
//!
//! A slider that controls the alpha of a red circle mesh:
//!
//! ```rust
//! use bevy::{prelude::*, sprite_render::MeshMaterial2d};
//! use woodpecker_ui::prelude::*;
//!
//! fn startup(
//!     mut commands: Commands,
//!     mut ui_context: ResMut<WoodpeckerContext>,
//!     mut meshes: ResMut<Assets<Mesh>>,
//!     mut materials: ResMut<Assets<ColorMaterial>>,
//! ) {
//!     commands.spawn((Camera2d, WoodpeckerView));
//!
//!     let material_red = materials.add(Color::Srgba(Srgba::RED.with_alpha(0.5)));
//!     commands.spawn((
//!         Mesh2d(meshes.add(Circle { radius: 50.0 })),
//!         MeshMaterial2d(material_red),
//!         Transform::from_xyz(0.0, 0.0, 0.0),
//!     ));
//!
//!     let root_widget = ui_context.spawn_root(&mut commands);
//!     commands.entity(*root_widget).insert((
//!         WoodpeckerStyle {
//!             padding: Edge::all(10.0),
//!             ..default()
//!         },
//!         WidgetChildren::default()
//!             .with_child::<Slider>(Slider {
//!                 start: 0.0,
//!                 end: 1.0,
//!                 value: 0.5,
//!             })
//!             .with_observe(
//!                 root_widget,
//!                 |trigger: On<Change<SliderChanged>>,
//!                  mut material_assets: ResMut<Assets<ColorMaterial>>,
//!                  query: Query<&MeshMaterial2d<ColorMaterial>>| {
//!                     for material in query.iter() {
//!                         material_assets
//!                             .get_mut(material)
//!                             .unwrap()
//!                             .color
//!                             .set_alpha(trigger.data.value)
//!                     }
//!                 },
//!             ),
//!     ));
//! }
//! ```
//!
//! See `examples/` in the repository for complete, runnable apps.
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::{
    asset::embedded_asset, camera::visibility::RenderLayers, prelude::*,
    reflect::GetTypeRegistration,
};
// use bevy_mod_picking::{events::Pointer, prelude::EventListenerPlugin};
use bevy_trait_query::RegisterExt;
use bevy_vello::prelude::{UiVelloScene, VelloFont};
use bevy_vello::render::VelloView;
use bevy_vello::{vello::AaConfig, VelloPlugin};
use context::{Widget, WoodpeckerContext};
use convert_render_target::ConvertRenderTargetPlugin;
use entity_mapping::WidgetMapper;
use font::FontManager;
use hook_helper::HookHelper;
use image::ImageManager;
use layout::WoodpeckerLayoutPlugin;
use metrics::WidgetMetrics;
use observer_cache::ObserverCache;
// use picking_backend::MouseWheelScroll;
use svg::{SvgAsset, SvgLoader, SvgManager};
use watched_resource::{sync_watched_resource, WatchedResource};
use widgets::{WoodpeckerApp, WoodpeckerUIWidgetPlugin};

mod children;
mod context;
mod convert_render_target;
mod cursor;
#[cfg(feature = "devtools")]
pub mod devtools;
mod diffable_prop;
mod diffing;
mod drag_drop;
mod entity_mapping;
mod focus;
mod font;
#[cfg(feature = "gamepad-nav")]
mod gamepad_focus;
mod hook_helper;
pub mod icons;
mod image;
mod keyboard_input;
mod layout;
mod metrics;
mod observer_cache;
mod on_change;
mod picking_backend;
mod portal;
mod previous_snapshot;
mod render;
mod rich_text;
mod runner;
mod styles;
mod svg;
mod tab_focus;
mod theming;
mod vello_renderer;
mod vello_svg;
mod watched_resource;
mod widgets;

/// A module that exports all publicly exposed types.
pub mod prelude {
    pub use crate::children::{Mounted, PassedChildren, WidgetChildren};
    pub use crate::context::*;
    pub use crate::cursor::CursorSetter;
    pub use crate::diffable_prop::{DiffableProp, ReflectDiffableProp};
    pub use crate::drag_drop::DragPhase;
    pub use crate::entity_mapping::*;
    pub use crate::focus::*;
    pub use crate::font::{FontManager, TextAlign};
    pub use crate::hook_helper::{HookHelper, PreviousWidget};
    pub use crate::icons;
    pub use crate::keyboard_input::{WidgetKeyboardButtonEvent, WidgetKeyboardCharEvent};
    pub use crate::layout::system::{
        StackingContext, WatchLayout, WidgetLayout, WidgetPreviousLayout,
    };
    pub use crate::metrics::WidgetMetrics;
    pub use crate::on_change::Change;
    pub use crate::picking_backend::{compute_letterboxed_transform, PointerWorldPosition};
    pub use crate::portal::{LogicalParent, OverlayRoot, Portal, SkipPortalLint};
    pub use crate::render::{WidgetRender, WidgetRenderCustom};
    pub use crate::rich_text::*;
    pub use crate::styles::*;
    pub use crate::svg::SvgAsset;
    pub use crate::tab_focus::FocusTrapRoot;
    pub use crate::theming::{Theme, ThemeOverride, ThemeRegisterExt, ThemedStyle};
    pub use crate::watched_resource::{sync_watched_resource, WatchedResource};
    pub use crate::widgets::*;
    pub use crate::WoodpeckerView;
    pub use crate::{
        CurrentWidget, DefaultFont, IconFont, ParentWidget, RenderSettings, WidgetRegisterExt,
        WoodpeckerUIPlugin,
    };
    pub use bevy::window::SystemCursorIcon;
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

/// A bevy resource used for icon glyphs -- set `WoodpeckerStyle::font` to
/// `Some(icon_font.0.id())` on any `WidgetRender::Text` whose `content` is one of the
/// [`crate::icons`] constants, rather than letting it fall back to [`DefaultFont`] (a body-text
/// face with no icon glyphs).
///
/// Defaults to the bundled Phosphor font, but can be swapped for a custom icon font via
/// [`WoodpeckerUIPlugin::with_icon_font`] -- pass any asset path your `AssetServer` can load
/// (e.g. an icon font you ship in your own `assets/` folder). Widgets throughout the crate only
/// ever reference glyphs by plain `&str`/`String`, so a custom font just needs its own module of
/// glyph constants (mirroring [`crate::icons`]) mapping names to that font's codepoints.
#[derive(Resource)]
pub struct IconFont(pub Handle<VelloFont>);

impl FromWorld for IconFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        IconFont(asset_server.load("embedded://woodpecker_ui/embedded_assets/Phosphor.ttf"))
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

/// Setup-time convenience extension for [`WoodpeckerContext`], kept out of `context.rs` (the
/// widget-system dispatch table itself) since it's pure app-wiring sugar, not part of running
/// widgets.
impl WoodpeckerContext {
    /// Spawns a root `WoodpeckerApp` entity, registers it as this context's root widget, and
    /// returns a `CurrentWidget` handle for building the initial tree. Collapses the spawn +
    /// `WoodpeckerApp` insert + `set_root_widget` ritual every example's `startup()` otherwise
    /// repeats by hand.
    pub fn spawn_root(&mut self, commands: &mut Commands) -> CurrentWidget {
        let root = commands.spawn(WoodpeckerApp).id();
        self.set_root_widget(root);
        CurrentWidget(root)
    }
}

/// A marker for woodpecker UI views.
#[derive(Component, Clone, Copy, Debug)]
#[require(VelloView = vello_view())]
pub struct WoodpeckerView;

fn vello_view() -> VelloView {
    VelloView
}

/// The Woodpecker UI bevy Plugin
/// Add this to bevy to use.
#[derive(Default)]
pub struct WoodpeckerUIPlugin {
    /// The render settings
    /// These settings are used to tell bevy_vello how to render.
    pub render_settings: RenderSettings,
    /// An asset path to load as the [`IconFont`] instead of the bundled Phosphor font. Set via
    /// [`Self::with_icon_font`].
    pub icon_font: Option<String>,
}

impl WoodpeckerUIPlugin {
    /// Uses the font at `path` (loaded through the normal `AssetServer`, so it can live in your
    /// own `assets/` folder) as the [`IconFont`] instead of the bundled Phosphor font. Icon
    /// glyphs are just `&str`/`String` content rendered with `WoodpeckerStyle::font` set to
    /// `Some(icon_font.0.id())`, so pair this with your own module of codepoint constants for
    /// the font you're loading (see `examples/custom_icon_font.rs`).
    pub fn with_icon_font(mut self, path: impl Into<String>) -> Self {
        self.icon_font = Some(path.into());
        self
    }
}

impl Plugin for WoodpeckerUIPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "embedded_assets/Poppins-Regular.ttf");
        embedded_asset!(app, "embedded_assets/Phosphor.ttf");
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
            .add_plugins(ExtractResourcePlugin::<ImageManager>::default())
            .add_plugins(ConvertRenderTargetPlugin)
            .insert_resource(focus::CurrentFocus::new(Entity::PLACEHOLDER))
            .init_resource::<drag_drop::DragAnchors>()
            .init_resource::<theming::Theme>()
            .init_resource::<ObserverCache>()
            .init_resource::<FontManager>()
            .init_resource::<HookHelper>()
            .init_resource::<WoodpeckerContext>()
            .init_resource::<WidgetMapper>()
            .init_resource::<DefaultFont>();
        match &self.icon_font {
            Some(path) => {
                let handle = app.world().resource::<AssetServer>().load(path.clone());
                app.insert_resource(IconFont(handle));
            }
            None => {
                app.init_resource::<IconFont>();
            }
        }
        app.init_resource::<WidgetMetrics>()
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
                    tab_focus::tab_navigate.after(keyboard_input::runner),
                    hook_helper::HookHelper::update_context_helper,
                    // Gated behind the `gamepad-nav` feature (which pulls in `bevy/gamepad`)
                    // -- ordered after `click_focus` so a same-frame mouse click always wins
                    // over a stale gamepad press.
                    #[cfg(feature = "gamepad-nav")]
                    (gamepad_focus::navigate, gamepad_focus::activate)
                        .after(focus::CurrentFocus::click_focus),
                )
                    .run_if(has_root()),
            )
            .add_systems(
                Update,
                (
                    font::load_fonts,
                    picking_backend::mouse_wheel_system,
                    #[cfg(feature = "metrics")]
                    metrics::WidgetMetrics::print_metrics_x_seconds,
                ),
            )
            // Runs in PostUpdate, after layout::system::run -- `.after()` only orders within
            // a schedule, so this must share PostUpdate to read this frame's fresh layout.
            .add_systems(
                PostUpdate,
                (
                    vello_renderer::run.after(layout::system::run),
                    picking_backend::system.after(layout::system::run),
                )
                    .run_if(has_root()),
            )
            // bevy_picking's own Hover/Click resolution (`PickingSystems::Hover`/`PostHover`/
            // `Last`) is hard-coded to `PreUpdate` inside `PickingPlugin::build` -- it is not
            // reconfigurable to another schedule from outside bevy_picking. Our own backend
            // above only emits `PointerHits` in `PostUpdate`, one schedule *after* the one
            // bevy_picking reads them in, which leaves a real gap: content that first becomes
            // clickable this frame (e.g. a freshly-opened Dropdown list) can have its very
            // first click silently swallowed. Also registering the same system in `PreUpdate`,
            // in the same `PickingSystems::Backend` set every other picking backend uses (see
            // `bevy_picking::mesh_picking`), closes that gap using the layout already
            // committed by last frame's `PostUpdate` -- the freshest data available at that
            // point, and exactly what a normal picking backend is expected to do. The
            // `PostUpdate` registration stays as-is; this doesn't replace it, just adds
            // same-frame closure on top.
            .add_systems(
                PreUpdate,
                picking_backend::system
                    .in_set(bevy::picking::PickingSystems::Backend)
                    .run_if(has_root()),
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
            .register_type::<Option<styles::WidgetPosition>>()
            .register_type::<styles::GridTemplate>()
            .register_type::<styles::GridTrackSize>()
            .register_type::<styles::WidgetGridAutoFlow>()
            .register_type::<styles::WidgetGridPlacement>()
            .register_type::<Vec<styles::GridTrackSize>>()
            .register_type::<styles::WidgetBoxShadow>()
            .register_type::<Option<styles::WidgetBoxShadow>>();
    }
}

fn has_root() -> impl SystemCondition<(), ()> {
    IntoSystem::into_system(|context: Res<WoodpeckerContext>| context.root_widget.is_some())
}

fn startup(mut commands: Commands, render_settings: Res<RenderSettings>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        // Required so bevy_ui's default picking backend doesn't treat this full-screen node
        // as hoverable and steal hits from Woodpecker's own picking_backend.
        Pickable::IGNORE,
        UiVelloScene::default(),
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

    /// Registers a `Resource` type `T` for reactive watching by widgets via
    /// `WatchedResource<T>`. Collapses the two calls this otherwise takes
    /// (`register_type::<WatchedResource<T>>()` + `add_systems(PreUpdate,
    /// sync_watched_resource::<T>)`) into one.
    fn register_watched_resource<T>(&mut self) -> &mut Self
    where
        T: Resource
            + Reflect
            + FromReflect
            + TypePath
            + bevy::reflect::Typed
            + GetTypeRegistration
            + Clone
            + PartialEq;
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

    fn register_watched_resource<T>(&mut self) -> &mut Self
    where
        T: Resource
            + Reflect
            + FromReflect
            + TypePath
            + bevy::reflect::Typed
            + GetTypeRegistration
            + Clone
            + PartialEq,
    {
        self.register_type::<WatchedResource<T>>();
        self.add_systems(PreUpdate, sync_watched_resource::<T>);
        self
    }
}

mod test_proc_macro {
    #[test]
    #[allow(
        dead_code,
        reason = "only exists to exercise the derive macro's expansion"
    )]
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
        pub struct MyStruct2 {}

        fn render() {}
    }
}
