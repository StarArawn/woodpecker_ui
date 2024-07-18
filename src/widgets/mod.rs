use bevy::prelude::*;
mod app;
use crate::{context::Widget, WidgetRegisterExt};
pub use app::{WoodpeckerApp, WoodpeckerAppBundle};

/// A core set of UI widgets that Wookpecker UI provides.
pub(crate) struct WoodpeckerUIWidgetPlugin;
impl Plugin for WoodpeckerUIWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<WoodpeckerApp>().add_widget_systems(
            WoodpeckerApp::get_name(),
            app::update,
            app::render,
        );
    }
}
