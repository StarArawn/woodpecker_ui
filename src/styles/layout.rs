use bevy::reflect::Reflect;

/// Sets the layout used for the children of this node
///
/// The default values depends on on which feature flags are enabled. The order of precedence is: Flex, Grid, Block, None.
/// Sets the layout used for the children of this node
///
/// The default values depends on on which feature flags are enabled. The order of precedence is: Flex, Grid, Block, None.
#[derive(Default, Reflect, Copy, Clone, PartialEq, Eq, Debug)]
pub enum WidgetDisplay {
    /// The children will follow the block layout algorithm
    Block,
    /// The children will follow the flexbox layout algorithm
    #[default]
    Flex,
    /// The children will follow the CSS Grid layout algorithm
    Grid,
    /// The children will not be laid out, and will follow absolute positioning
    None,
}

impl From<WidgetDisplay> for taffy::Display {
    fn from(val: WidgetDisplay) -> taffy::Display {
        match val {
            WidgetDisplay::Block => taffy::Display::Block,
            WidgetDisplay::Flex => taffy::Display::Flex,
            WidgetDisplay::Grid => taffy::Display::Grid,
            WidgetDisplay::None => taffy::Display::None,
        }
    }
}

/// How children overflowing their container should affect layout
///
/// In CSS the primary effect of this property is to control whether contents of a parent container that overflow that container should
/// be displayed anyway, be clipped, or trigger the container to become a scroll container. However it also has secondary effects on layout,
/// the main ones being:
///
///   - The automatic minimum size Flexbox/CSS Grid items with non-`Visible` overflow is `0` rather than being content based
///   - `Overflow::Scroll` nodes have space in the layout reserved for a scrollbar (width controlled by the `scrollbar_width` property)
///
/// In Taffy, we only implement the layout related secondary effects as we are not concerned with drawing/painting. The amount of space reserved for
/// a scrollbar is controlled by the `scrollbar_width` property. If this is `0` then `Scroll` behaves identically to `Hidden`.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/overflow>
#[derive(Copy, Reflect, Clone, PartialEq, Eq, Debug, Default)]
pub enum WidgetOverflow {
    /// The automatic minimum size of this node as a flexbox/grid item should be based on the size of its content.
    /// Content that overflows this node *should* contribute to the scroll region of its parent.
    #[default]
    Visible,
    /// The automatic minimum size of this node as a flexbox/grid item should be based on the size of its content.
    /// Content that overflows this node should *not* contribute to the scroll region of its parent.
    Clip,
    /// The automatic minimum size of this node as a flexbox/grid item should be `0`.
    /// Content that overflows this node should *not* contribute to the scroll region of its parent.
    Hidden,
    /// The automatic minimum size of this node as a flexbox/grid item should be `0`. Additionally, space should be reserved
    /// for a scrollbar. The amount of space reserved is controlled by the `scrollbar_width` property.
    /// Content that overflows this node should *not* contribute to the scroll region of its parent.
    Scroll,
}

impl From<WidgetOverflow> for taffy::Overflow {
    fn from(val: WidgetOverflow) -> taffy::Overflow {
        match val {
            WidgetOverflow::Visible => taffy::Overflow::Visible,
            WidgetOverflow::Clip => taffy::Overflow::Clip,
            WidgetOverflow::Hidden => taffy::Overflow::Hidden,
            WidgetOverflow::Scroll => taffy::Overflow::Scroll,
        }
    }
}

/// The positioning strategy for this item.
///
/// This controls both how the origin is determined for the [`Style::position`] field,
/// and whether or not the item will be controlled by flexbox's layout algorithm.
///
/// WARNING: this enum follows the behavior of [CSS's `position` property](https://developer.mozilla.org/en-US/docs/Web/CSS/position),
/// which can be unintuitive.
///
/// [`Position::Relative`] is the default value, in contrast to the default behavior in CSS.
#[derive(Default, Reflect, Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WidgetPosition {
    /// The offset is computed relative to the final position given by the layout algorithm.
    /// Offsets do not affect the position of any other items; they are effectively a correction factor applied at the end.
    #[default]
    Relative,
    /// The offset is computed relative to this item's closest positioned ancestor, if any.
    /// Otherwise, it is placed relative to the origin.
    /// No space is created for the item in the page layout, and its size will not be altered.
    ///
    /// WARNING: to opt-out of layouting entirely, you must use [`Display::None`] instead on your [`Style`] object.
    Absolute,
    /// A fixed position that will match the size of the camera viewport.
    Fixed,
}

impl From<WidgetPosition> for taffy::Position {
    fn from(val: WidgetPosition) -> taffy::Position {
        match val {
            WidgetPosition::Relative => taffy::Position::Relative,
            WidgetPosition::Absolute => taffy::Position::Absolute,
            _ => taffy::Position::Absolute,
        }
    }
}

impl From<super::Units> for taffy::Dimension {
    fn from(val: super::Units) -> taffy::Dimension {
        match val {
            super::Units::Pixels(pixels) => taffy::Dimension::Length(pixels),
            super::Units::Percentage(percentage) => taffy::Dimension::Percent(percentage / 100.0),
            super::Units::Auto => taffy::Dimension::Auto,
        }
    }
}

impl From<super::Units> for taffy::LengthPercentageAuto {
    fn from(val: super::Units) -> taffy::LengthPercentageAuto {
        match val {
            super::Units::Pixels(pixels) => taffy::LengthPercentageAuto::Length(pixels),
            super::Units::Percentage(percentage) => {
                taffy::LengthPercentageAuto::Percent(percentage / 100.0)
            }
            super::Units::Auto => taffy::LengthPercentageAuto::Auto,
        }
    }
}

impl From<super::Units> for taffy::LengthPercentage {
    fn from(val: super::Units) -> taffy::LengthPercentage {
        match val {
            super::Units::Pixels(pixels) => taffy::LengthPercentage::Length(pixels),
            super::Units::Percentage(percentage) => {
                taffy::LengthPercentage::Percent(percentage / 100.0)
            }
            super::Units::Auto => taffy::LengthPercentage::Percent(1.0),
        }
    }
}

impl From<super::Edge> for taffy::Rect<taffy::LengthPercentageAuto> {
    fn from(val: super::Edge) -> taffy::Rect<taffy::LengthPercentageAuto> {
        taffy::Rect {
            left: val.left.into(),
            right: val.right.into(),
            top: val.top.into(),
            bottom: val.bottom.into(),
        }
    }
}

impl From<super::Edge> for taffy::Rect<taffy::LengthPercentage> {
    fn from(val: super::Edge) -> taffy::Rect<taffy::LengthPercentage> {
        taffy::Rect {
            left: val.left.into(),
            right: val.right.into(),
            top: val.top.into(),
            bottom: val.bottom.into(),
        }
    }
}

/// Used to control how child nodes are aligned.
/// For Flexbox it controls alignment in the cross axis
/// For Grid it controls alignment in the block axis
///
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/align-items)
#[derive(Default, Reflect, Copy, Clone, PartialEq, Eq, Debug)]
pub enum WidgetAlignItems {
    /// Items are packed toward the start of the axis
    Start,
    /// Items are packed toward the end of the axis
    End,
    /// Items are packed towards the flex-relative start of the axis.
    ///
    /// For flex containers with flex_direction RowReverse or ColumnReverse this is equivalent
    /// to End. In all other cases it is equivalent to Start.
    #[default]
    FlexStart,
    /// Items are packed towards the flex-relative end of the axis.
    ///
    /// For flex containers with flex_direction RowReverse or ColumnReverse this is equivalent
    /// to Start. In all other cases it is equivalent to End.
    FlexEnd,
    /// Items are packed along the center of the cross axis
    Center,
    /// Items are aligned such as their baselines align
    Baseline,
    /// Stretch to fill the container
    Stretch,
}

impl From<WidgetAlignItems> for taffy::AlignItems {
    fn from(val: WidgetAlignItems) -> taffy::AlignItems {
        match val {
            WidgetAlignItems::Start => taffy::AlignItems::Start,
            WidgetAlignItems::End => taffy::AlignItems::End,
            WidgetAlignItems::FlexStart => taffy::AlignItems::FlexStart,
            WidgetAlignItems::FlexEnd => taffy::AlignItems::FlexEnd,
            WidgetAlignItems::Center => taffy::AlignItems::Center,
            WidgetAlignItems::Baseline => taffy::AlignItems::Baseline,
            WidgetAlignItems::Stretch => taffy::AlignItems::Stretch,
        }
    }
}

/// Used to control how the specified nodes is aligned.
/// Overrides the parent Node's `AlignItems` property.
/// For Flexbox it controls alignment in the cross axis
/// For Grid it controls alignment in the block axis
///
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/align-self)
pub type WidgetAlignSelf = WidgetAlignItems;

/// Sets the distribution of space between and around content items
/// For Flexbox it controls alignment in the cross axis
/// For Grid it controls alignment in the block axis
///
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/align-content)
#[derive(Default, Reflect, Copy, Clone, PartialEq, Eq, Debug)]
pub enum WidgetAlignContent {
    /// Items are packed toward the start of the axis
    Start,
    /// Items are packed toward the end of the axis
    End,
    /// Items are packed towards the flex-relative start of the axis.
    ///
    /// For flex containers with flex_direction RowReverse or ColumnReverse this is equivalent
    /// to End. In all other cases it is equivalent to Start.
    #[default]
    FlexStart,
    /// Items are packed towards the flex-relative end of the axis.
    ///
    /// For flex containers with flex_direction RowReverse or ColumnReverse this is equivalent
    /// to Start. In all other cases it is equivalent to End.
    FlexEnd,
    /// Items are centered around the middle of the axis
    Center,
    /// Items are stretched to fill the container
    Stretch,
    /// The first and last items are aligned flush with the edges of the container (no gap)
    /// The gap between items is distributed evenly.
    SpaceBetween,
    /// The gap between the first and last items is exactly THE SAME as the gap between items.
    /// The gaps are distributed evenly
    SpaceEvenly,
    /// The gap between the first and last items is exactly HALF the gap between items.
    /// The gaps are distributed evenly in proportion to these ratios.
    SpaceAround,
}

impl From<WidgetAlignContent> for taffy::AlignContent {
    fn from(val: WidgetAlignContent) -> taffy::AlignContent {
        match val {
            WidgetAlignContent::Start => taffy::AlignContent::Start,
            WidgetAlignContent::End => taffy::AlignContent::End,
            WidgetAlignContent::FlexStart => taffy::AlignContent::FlexStart,
            WidgetAlignContent::FlexEnd => taffy::AlignContent::FlexEnd,
            WidgetAlignContent::Center => taffy::AlignContent::Center,
            WidgetAlignContent::Stretch => taffy::AlignContent::Stretch,
            WidgetAlignContent::SpaceBetween => taffy::AlignContent::SpaceBetween,
            WidgetAlignContent::SpaceEvenly => taffy::AlignContent::SpaceEvenly,
            WidgetAlignContent::SpaceAround => taffy::AlignContent::SpaceAround,
        }
    }
}

/// Sets the distribution of space between and around content items
/// For Flexbox it controls alignment in the main axis
/// For Grid it controls alignment in the inline axis
///
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/justify-content)
pub type WidgetJustifyContent = WidgetAlignContent;

/// Used to control how child nodes are aligned.
/// Does not apply to Flexbox, and will be ignored if specified on a flex container
/// For Grid it controls alignment in the inline axis
///
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/justify-items)
pub type WidgetJustifyItems = WidgetAlignItems;
/// Used to control how the specified nodes is aligned.
/// Overrides the parent Node's `JustifyItems` property.
/// Does not apply to Flexbox, and will be ignored if specified on a flex child
/// For Grid it controls alignment in the inline axis
///
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/justify-self)
pub type WidgetJustifySelf = WidgetAlignItems;
/// The direction of the flexbox layout main axis.
///
/// There are always two perpendicular layout axes: main (or primary) and cross (or secondary).
/// Adding items will cause them to be positioned adjacent to each other along the main axis.
/// By varying this value throughout your tree, you can create complex axis-aligned layouts.
///
/// Items are always aligned relative to the cross axis, and justified relative to the main axis.
///
/// The default behavior is [`FlexDirection::Row`].
///
/// [Specification](https://www.w3.org/TR/css-flexbox-1/#flex-direction-property)
#[derive(Default, Reflect, Copy, Clone, PartialEq, Eq, Debug)]
pub enum WidgetFlexDirection {
    /// Defines +x as the main axis
    ///
    /// Items will be added from left to right in a row.
    #[default]
    Row,
    /// Defines +y as the main axis
    ///
    /// Items will be added from top to bottom in a column.
    Column,
    /// Defines -x as the main axis
    ///
    /// Items will be added from right to left in a row.
    RowReverse,
    /// Defines -y as the main axis
    ///
    /// Items will be added from bottom to top in a column.
    ColumnReverse,
}

impl From<WidgetFlexDirection> for taffy::FlexDirection {
    fn from(val: WidgetFlexDirection) -> taffy::FlexDirection {
        match val {
            WidgetFlexDirection::Row => taffy::FlexDirection::Row,
            WidgetFlexDirection::Column => taffy::FlexDirection::Column,
            WidgetFlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
            WidgetFlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
        }
    }
}

/// Controls whether flex items are forced onto one line or can wrap onto multiple lines.
///
/// Defaults to [`FlexWrap::NoWrap`]
///
/// [Specification](https://www.w3.org/TR/css-flexbox-1/#flex-wrap-property)
#[derive(Default, Reflect, Copy, Clone, PartialEq, Eq, Debug)]
pub enum WidgetFlexWrap {
    /// Items will not wrap and stay on a single line
    #[default]
    NoWrap,
    /// Items will wrap according to this item's [`FlexDirection`]
    Wrap,
    /// Items will wrap in the opposite direction to this item's [`FlexDirection`]
    WrapReverse,
}

impl From<WidgetFlexWrap> for taffy::FlexWrap {
    fn from(val: WidgetFlexWrap) -> taffy::FlexWrap {
        match val {
            WidgetFlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
            WidgetFlexWrap::Wrap => taffy::FlexWrap::Wrap,
            WidgetFlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
        }
    }
}
