use bevy::prelude::*;
pub use corner::Corner;
pub use edge::Edge;
pub use layout::*;
pub use units::Units;

mod corner;
mod edge;
mod layout;
mod units;

// A struct used to define the look of a widget
///
/// All fields are `pub`, so you can simply define your styles.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct WoodpeckerStyle {
    /************************ Layout ************************/
    /// The height of this widget
    pub height: Units,
    /// The maximum height of this widget
    pub max_height: Units,
    /// The maximum width of this widget
    pub max_width: Units,
    /// The minimum height of this widget
    pub min_height: Units,
    /// The minimum width of this widget
    pub min_width: Units,
    /// The inner padding between the edges of this widget and its children
    pub padding: Edge,
    /// The width of this widget
    pub width: Units,
    /// Sets the layout used for the children of this node
    ///
    /// The default values depends on on which feature flags are enabled. The order of precedence is: Flex, Grid, Block, None.
    pub display: WidgetDisplay,
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
    pub overflow: WidgetOverflow,
    /// The positioning strategy for this item.
    ///
    /// This controls both how the origin is determined for the [`Style::position`] field,
    /// and whether or not the item will be controlled by flexbox's layout algorithm.
    ///
    /// WARNING: this enum follows the behavior of [CSS's `position` property](https://developer.mozilla.org/en-US/docs/Web/CSS/position),
    /// which can be unintuitive.
    ///
    /// [`Position::Relative`] is the default value, in contrast to the default behavior in CSS.
    pub position: WidgetPosition,
    /// Position Left
    pub left: Units,
    /// Position Left
    pub right: Units,
    /// Position Left
    pub top: Units,
    /// Position Left
    pub bottom: Units,
    /// How large should the margin be on each side?
    pub margin: Edge,
    /// Used to control how child nodes are aligned.
    /// For Flexbox it controls alignment in the cross axis
    /// For Grid it controls alignment in the block axis
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/align-items)
    pub align_items: Option<WidgetAlignItems>,
    /// Used to control how the specified nodes is aligned.
    /// Overrides the parent Node's `AlignItems` property.
    /// For Flexbox it controls alignment in the cross axis
    /// For Grid it controls alignment in the block axis
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/align-self)
    pub align_self: Option<WidgetAlignSelf>,
    /// Used to control how child nodes are aligned.
    /// Does not apply to Flexbox, and will be ignored if specified on a flex container
    /// For Grid it controls alignment in the inline axis
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/justify-items)
    pub justify_items: Option<WidgetJustifyItems>,
    /// Used to control how the specified nodes is aligned.
    /// Overrides the parent Node's `JustifyItems` property.
    /// Does not apply to Flexbox, and will be ignored if specified on a flex child
    /// For Grid it controls alignment in the inline axis
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/justify-self)
    pub justify_self: Option<WidgetJustifySelf>,
    /// Sets the distribution of space between and around content items
    /// For Flexbox it controls alignment in the cross axis
    /// For Grid it controls alignment in the block axis
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/align-content)
    pub align_content: Option<WidgetAlignContent>,
    /// Sets the distribution of space between and around content items
    /// For Flexbox it controls alignment in the main axis
    /// For Grid it controls alignment in the inline axis
    ///
    /// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/justify-content)
    pub justify_content: Option<WidgetJustifyContent>,
    /// How large should the gaps between items in a grid or flex container be?
    pub gap: (Units, Units),
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
    pub flex_direction: WidgetFlexDirection,
    /// Controls whether flex items are forced onto one line or can wrap onto multiple lines.
    ///
    /// Defaults to [`FlexWrap::NoWrap`]
    ///
    /// [Specification](https://www.w3.org/TR/css-flexbox-1/#flex-wrap-property)
    pub flex_wrap: WidgetFlexWrap,
    /// Sets the initial main axis size of the item
    pub flex_basis: Units,
    /// The relative rate at which this item grows when it is expanding to fill space
    ///
    /// 0.0 is the default value, and this value must be positive.
    pub flex_grow: f32,
    /// The relative rate at which this item shrinks when it is contracting to fit into space
    ///
    /// 1.0 is the default value, and this value must be positive.
    pub flex_shrink: f32,
    // TODO: Add grid support..
    /************************ Rendering ************************/
    /// The background color of this widget
    ///
    /// Only applies to widgets marked [`RenderCommand::Quad`]
    pub background_color: Color,
    /// The color of the border around this widget
    ///
    /// Currently, this controls all border sides.
    ///
    /// Only applies to widgets marked [`RenderCommand::Quad`]
    pub border_color: Color,
    /// The radius of the corners (in pixels)
    ///
    /// The order is (Top, Right, Bottom, Left).
    ///
    /// Only applies to widgets marked [`RenderCommand::Quad`] and [`RenderCommand::Image`]
    pub border_radius: Corner,
    /// The widths of the borders (in pixels)
    ///
    /// The order is (Top, Right, Bottom, Left).
    ///
    /// Only applies to widgets marked [`RenderCommand::Quad`]
    pub border: Edge,
    /// The text color for this widget
    ///
    /// This property defaults to [`StyleProp::Inherit`] meaning that setting this field to some value will
    /// cause all descendents to receive that value, up to the next set value.
    ///
    /// Only applies to widgets marked [`RenderCommand::Text`]
    pub color: Color,
    /// The font size for this widget, in pixels
    ///
    /// Only applies to [`RenderCommand::Text`]
    pub font_size: f32,
    /// The layout method for children of this widget
    /// The line height for this widget, in pixels
    pub line_height: f32,
    /// The opacity of the widget and it's children
    /// Note: This will spawn a new UI render layer so use sparingly.
    pub opacity: f32,
}

impl Default for WoodpeckerStyle {
    fn default() -> Self {
        WoodpeckerStyle::DEFAULT
    }
}

impl WoodpeckerStyle {
    pub const DEFAULT: WoodpeckerStyle = WoodpeckerStyle {
        width: Units::Auto,
        height: Units::Auto,
        max_height: Units::Auto,
        max_width: Units::Auto,
        min_height: Units::Auto,
        min_width: Units::Auto,
        padding: Edge {
            left: Units::Pixels(0.0),
            right: Units::Pixels(0.0),
            top: Units::Pixels(0.0),
            bottom: Units::Pixels(0.0),
        },
        display: WidgetDisplay::Flex,
        overflow: WidgetOverflow::Visible,
        position: WidgetPosition::Relative,
        left: Units::Pixels(0.0),
        right: Units::Pixels(0.0),
        top: Units::Pixels(0.0),
        bottom: Units::Pixels(0.0),
        margin: Edge {
            left: Units::Pixels(0.0),
            right: Units::Pixels(0.0),
            top: Units::Pixels(0.0),
            bottom: Units::Pixels(0.0),
        },
        align_items: None,
        align_self: None,
        justify_items: None,
        justify_self: None,
        align_content: None,
        justify_content: None,
        gap: (Units::Pixels(0.0), Units::Pixels(0.0)),
        flex_direction: WidgetFlexDirection::Row,
        flex_wrap: WidgetFlexWrap::NoWrap,
        flex_basis: Units::Auto,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        background_color: Color::WHITE,
        border_color: Color::WHITE,
        border_radius: Corner {
            top_left: Units::Pixels(0.0),
            top_right: Units::Pixels(0.0),
            bottom_left: Units::Pixels(0.0),
            bottom_right: Units::Pixels(0.0),
        },
        border: Edge {
            left: Units::Pixels(0.0),
            right: Units::Pixels(0.0),
            top: Units::Pixels(0.0),
            bottom: Units::Pixels(0.0),
        },
        color: Color::WHITE,
        font_size: 18.0,
        line_height: 18.0,
        opacity: 1.0,
    };
}

impl Into<taffy::Style> for &WoodpeckerStyle {
    fn into(self) -> taffy::Style {
        (*self).into()
    }
}

impl Into<taffy::Style> for WoodpeckerStyle {
    fn into(self) -> taffy::Style {
        taffy::Style {
            display: self.display.into(),
            overflow: taffy::Point {
                x: self.overflow.into(),
                y: self.overflow.into(),
            },
            position: self.position.into(),
            inset: Edge::new(self.top, self.right, self.bottom, self.left).into(),
            size: taffy::Size {
                width: self.width.into(),
                height: self.height.into(),
            },
            min_size: taffy::Size {
                width: self.min_width.into(),
                height: self.min_height.into(),
            },
            max_size: taffy::Size {
                width: self.max_width.into(),
                height: self.max_height.into(),
            },
            margin: self.margin.into(),
            padding: self.padding.into(),
            border: self.border.into(),
            align_items: self.align_items.map(|i| i.into()),
            align_self: self.align_self.map(|i| i.into()),
            justify_items: self.justify_items.map(|i| i.into()),
            justify_self: self.justify_self.map(|i| i.into()),
            align_content: self.align_content.map(|i| i.into()),
            justify_content: self.justify_content.map(|i| i.into()),
            gap: taffy::Size {
                width: self.gap.0.into(),
                height: self.gap.1.into(),
            },
            flex_direction: self.flex_direction.into(),
            flex_wrap: self.flex_wrap.into(),
            flex_basis: self.flex_basis.into(),
            flex_grow: self.flex_grow.into(),
            flex_shrink: self.flex_shrink.into(),
            ..Default::default()
        }
    }
}
