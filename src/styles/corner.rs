use bevy::reflect::Reflect;

use super::units::Units;

/// A struct for defining properties related to the corners of widgets
///
/// This is useful for things like border radii, etc.
#[derive(Debug, Default, Reflect, Copy, Clone, PartialEq)]
pub struct Corner {
    /// The value of the top-left corner
    pub top_left: Units,
    /// The value of the top-right corner
    pub top_right: Units,
    /// The value of the bottom-left corner
    pub bottom_left: Units,
    /// The value of the bottom-right corner
    pub bottom_right: Units,
}

impl Corner {
    /// Creates a new `Corner` with values individually specified for each corner
    ///
    /// # Arguments
    ///
    /// * `top_left`: The top-left corner value
    /// * `top_right`: The top_-right corner value
    /// * `bottom_left`: The bottom_-left corner value
    /// * `bottom_right`: The bottom_-right corner value
    ///
    pub fn new(top_left: Units, top_right: Units, bottom_left: Units, bottom_right: Units) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    /// Creates a new `Corner` with matching top corners and matching bottom corners
    ///
    /// # Arguments
    ///
    /// * `top`: The value of the top corners
    /// * `bottom`: The value of the bottom corners
    ///
    /// ```
    /// # use kayak_core::styles::Corner;
    /// // Creates a `Corner` with only the top corners rounded
    /// let corner_radius = Corner::vertical(10.0, 0.0);
    ///
    /// // Creates a `Corner` with only the bottom corners rounded
    /// let corner_radius = Corner::vertical(0.0, 10.0);
    /// ```
    pub fn vertical(top: Units, bottom: Units) -> Self {
        Self {
            top_left: top,
            top_right: top,
            bottom_left: bottom,
            bottom_right: bottom,
        }
    }

    /// Creates a new `Corner` with matching left corners and matching right corners
    ///
    /// # Arguments
    ///
    /// * `left`: The value of the left corners
    /// * `right`: The value of the right corners
    ///
    /// ```
    /// # use kayak_core::styles::Corner;
    /// // Creates a `Corner` with only the left corners rounded
    /// let corner_radius = Corner::horizontal(10.0, 0.0);
    ///
    /// // Creates a `Corner` with only the right corners rounded
    /// let corner_radius = Corner::horizontal(0.0, 10.0);
    /// ```
    pub fn horizontal(left: Units, right: Units) -> Self {
        Self {
            top_left: left,
            top_right: right,
            bottom_left: left,
            bottom_right: right,
        }
    }

    /// Creates a new `Corner` with all corners having the same value
    ///
    /// # Arguments
    ///
    /// * `value`: The value of all corners
    ///
    pub fn all(value: Units) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_left: value,
            bottom_right: value,
        }
    }

    /// Converts this `Corner` into a tuple matching `(Top Left, Top Right, Bottom Left, Bottom Right)`
    pub fn into_tuple(self) -> (Units, Units, Units, Units) {
        (
            self.top_left,
            self.top_right,
            self.bottom_left,
            self.bottom_right,
        )
    }
}

impl From<Corner> for (Units, Units, Units, Units) {
    /// Creates a tuple matching the pattern: `(Top Left, Top Right, Bottom Left, Bottom Right)`
    fn from(edge: Corner) -> Self {
        edge.into_tuple()
    }
}

impl From<(Units, Units, Units, Units)> for Corner {
    /// Converts the tuple according to the pattern: `(Top Left, Top Right, Bottom Left, Bottom Right)`
    fn from(value: (Units, Units, Units, Units)) -> Self {
        Corner::new(value.0, value.1, value.2, value.3)
    }
}
