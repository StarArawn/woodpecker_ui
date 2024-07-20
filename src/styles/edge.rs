use std::ops::{Mul, MulAssign};

use bevy::reflect::Reflect;

use super::Units;

/// A struct for defining properties related to the edges of widgets
///
/// This is useful for things like borders, padding, etc.
#[derive(Debug, Default, Reflect, Copy, Clone, PartialEq)]
pub struct Edge {
    /// The value of the top edge
    pub top: Units,
    /// The value of the right edge
    pub right: Units,
    /// The value of the bottom edge
    pub bottom: Units,
    /// The value of the left edge
    pub left: Units,
}

impl Edge {
    /// Creates a new `Edge` with values individually specified for each edge
    ///
    /// # Arguments
    ///
    /// * `top`: The top edge value
    /// * `right`: The right edge value
    /// * `bottom`: The bottom edge value
    /// * `left`: The left edge value
    ///
    pub fn new(top: Units, right: Units, bottom: Units, left: Units) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Creates a new `Edge` with matching vertical edges and matching horizontal edges
    ///
    /// # Arguments
    ///
    /// * `vertical`: The value of the vertical edges
    /// * `horizontal`: The value of the horizontal edges
    ///
    pub fn axis(vertical: Units, horizontal: Units) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Creates a new `Edge` with all edges having the same value
    ///
    /// # Arguments
    ///
    /// * `value`: The value of all edges
    ///
    pub fn all(value: Units) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Converts this `Edge` into a tuple matching `(Top, Right, Bottom, Left)`
    pub fn into_tuple(self) -> (Units, Units, Units, Units) {
        (self.top, self.right, self.bottom, self.left)
    }
}

impl From<Edge> for (Units, Units, Units, Units) {
    fn from(edge: Edge) -> Self {
        edge.into_tuple()
    }
}

impl From<(Units, Units)> for Edge {
    fn from(value: (Units, Units)) -> Self {
        Edge::axis(value.0, value.1)
    }
}

impl From<(Units, Units, Units, Units)> for Edge {
    fn from(value: (Units, Units, Units, Units)) -> Self {
        Edge::new(value.0, value.1, value.2, value.3)
    }
}
