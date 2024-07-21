use crate::WidgetRegisterExt;
use bevy::prelude::*;

mod app;
mod button;
mod clip;
mod element;
mod modal;
mod text_box;
mod transition;

pub use app::{WoodpeckerApp, WoodpeckerAppBundle};
pub use button::{ButtonStyles, WButton, WButtonBundle};
pub use clip::{Clip, ClipBundle};
pub use element::{Element, ElementBundle};
pub use text_box::{TextBox, TextBoxBundle, TextboxStyles};
pub use transition::*;

/// A core set of UI widgets that Wookpecker UI provides.
// TODO: Make this optional? Expose publicly.
pub(crate) struct WoodpeckerUIWidgetPlugin;
impl Plugin for WoodpeckerUIWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<WoodpeckerApp>()
            .register_widget::<Element>()
            .register_widget::<WButton>()
            .register_widget::<Clip>()
            .register_widget::<TextBox>()
            .add_systems(
                Update,
                (
                    text_box::cursor_animation_system,
                    transition::update_transitions,
                ),
            );
    }
}
