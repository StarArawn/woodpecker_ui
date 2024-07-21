use bevy::prelude::*;

use crate::prelude::*;

#[derive(Component, Widget, PartialEq, Clone, Debug)]
pub struct Modal {
    /// The text to display in the modal's title bar
    pub title: String,
    /// A set of styles to apply to the children element wrapper.
    pub children_styles: WoodpeckerStyle,
    /// Is the modal open?
    pub visible: bool,
    /// Animation timeout in milliseconds.
    pub timeout: f32,
    /// The overlay background alpha value
    pub overlay_alpha: f32,
}

impl Default for Modal {
    fn default() -> Self {
        Self {
            title: Default::default(),
            children_styles: Default::default(),
            visible: Default::default(),
            timeout: 250.0,
            overlay_alpha: 0.95,
        }
    }
}
