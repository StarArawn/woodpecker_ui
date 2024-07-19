use bevy::prelude::*;
mod app;
mod button;
mod clip;
mod element;
use crate::{context::Widget, WidgetRegisterExt};
pub use app::{WoodpeckerApp, WoodpeckerAppBundle};
pub use button::{ButtonStyles, WButton, WButtonBundle};
pub use clip::{Clip, ClipBundle};
pub use element::{Element, ElementBundle};

/// A core set of UI widgets that Wookpecker UI provides.
// TODO: Make this optional? Expose publicly.
pub(crate) struct WoodpeckerUIWidgetPlugin;
impl Plugin for WoodpeckerUIWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<WoodpeckerApp>()
            .add_widget_systems(WoodpeckerApp::get_name(), app::update, app::render)
            .register_widget::<Element>()
            .add_widget_systems(Element::get_name(), element::update, element::render)
            .register_widget::<WButton>()
            .add_widget_systems(WButton::get_name(), button::update, button::render)
            .register_widget::<Clip>()
            .add_widget_systems(Clip::get_name(), clip::update, clip::render);
    }
}
