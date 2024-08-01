use crate::prelude::OnChange;
use crate::WidgetRegisterExt;
use bevy::prelude::*;

mod app;
mod button;
mod clip;
mod element;
mod icon_button;
mod modal;
mod scroll;
mod text_box;
mod transition;
mod toggle;
pub mod colors;
mod slider;

pub use app::{WoodpeckerApp, WoodpeckerAppBundle};
use bevy_mod_picking::prelude::EventListenerPlugin;
pub use button::{ButtonStyles, WButton, WButtonBundle};
pub use clip::{Clip, ClipBundle};
pub use element::{Element, ElementBundle};
pub use icon_button::{IconButton, IconButtonBundle, IconButtonStyles};
pub use modal::{Modal, ModalBundle};
pub use scroll::content::{ScrollContent, ScrollContentBundle};
pub use scroll::scroll_bar::{ScrollBar, ScrollBarBundle};
pub use scroll::scroll_box::{ScrollBox, ScrollBoxBundle};
pub use scroll::{ScrollContextProvider, ScrollContextProviderBundle};
pub use text_box::{TextChanged, TextBox, TextBoxBundle, TextBoxState, TextboxStyles};
pub use toggle::{Toggle, ToggleBundle, ToggleState, ToggleWidgetStyles, ToggleStyles, ToggleChanged};
pub use slider::{Slider, SliderChanged, SliderState, SliderStyles, SliderBundle};
pub use transition::*;

/// A core set of UI widgets that Wookpecker UI provides.
// TODO: Make this optional? Expose publicly.
pub(crate) struct WoodpeckerUIWidgetPlugin;
impl Plugin for WoodpeckerUIWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EventListenerPlugin::<OnChange<TextChanged>>::default())
            .add_plugins(EventListenerPlugin::<OnChange<ToggleChanged>>::default())
            .add_plugins(EventListenerPlugin::<OnChange<SliderChanged>>::default())
            .register_widget::<WoodpeckerApp>()
            .register_widget::<Element>()
            .register_widget::<WButton>()
            .register_widget::<Clip>()
            .register_widget::<TextBox>()
            .register_widget::<Modal>()
            .register_widget::<ScrollContextProvider>()
            .register_widget::<ScrollContent>()
            .register_widget::<ScrollBox>()
            .register_widget::<ScrollBar>()
            .register_widget::<IconButton>()
            .register_widget::<Toggle>()
            .register_widget::<Slider>()
            .add_systems(
                Update,
                (
                    text_box::cursor_animation_system,
                    transition::update_transitions,
                ),
            );
    }
}
