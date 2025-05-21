use crate::prelude::Change;
use crate::WidgetRegisterExt;
use bevy::prelude::*;

mod app;
mod button;
mod checkbox;
mod clip;
mod color_picker;
/// A set of default colors used by Woodpecker UI.
pub mod colors;
mod dropdown;
mod element;
mod icon_button;
mod modal;
mod scroll;
mod slider;
mod tab;
// mod text_box;
mod text_box;
mod toggle;
mod transition;
mod window;

pub use app::WoodpeckerApp;
// use bevy_mod_picking::prelude::EventListenerPlugin;
pub use button::{ButtonStyles, WButton};
pub use checkbox::{
    Checkbox, CheckboxChanged, CheckboxState, CheckboxStyles, CheckboxWidgetStyles,
};
pub use clip::Clip;
pub use color_picker::{ColorPicker, ColorPickerChanged};
pub use dropdown::{Dropdown, DropdownChanged, DropdownStyles};
pub use element::Element;
pub use icon_button::{IconButton, IconButtonStyles};
pub use modal::Modal;
pub use scroll::content::ScrollContent;
pub use scroll::scroll_bar::ScrollBar;
pub use scroll::scroll_box::ScrollBox;
pub use scroll::{ScrollContext, ScrollContextProvider, TaggedContext};
pub use slider::{Slider, SliderChanged, SliderState, SliderStyles};
pub use tab::*;
pub use text_box::{TextBox, TextBoxState, TextChanged, TextboxStyles};
pub use toggle::{Toggle, ToggleChanged, ToggleState, ToggleStyles, ToggleWidgetStyles};
pub use transition::*;
pub use window::{WindowState, WoodpeckerWindow};

/// A core set of UI widgets that Woodpecker UI provides.
// TODO: Make this optional? Expose publicly.
pub(crate) struct WoodpeckerUIWidgetPlugin;
impl Plugin for WoodpeckerUIWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Change<TextChanged>>()
            .add_event::<Change<ToggleChanged>>()
            .add_event::<Change<CheckboxChanged>>()
            .add_event::<Change<SliderChanged>>()
            .add_event::<Change<DropdownChanged>>()
            .add_event::<Change<ColorPickerChanged>>()
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
            .register_widget::<WoodpeckerWindow>()
            .register_widget::<Dropdown>()
            .register_widget::<TabButton>()
            .register_widget::<TabContextProvider>()
            .register_widget::<TabContent>()
            .register_widget::<Checkbox>()
            .register_widget::<ColorPicker>()
            .add_systems(
                Update,
                (
                    text_box::cursor_animation_system,
                    transition::update_transitions,
                ),
            );
    }
}
