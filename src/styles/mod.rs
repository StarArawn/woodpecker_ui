use bevy::prelude::*;
use bevy_vello::text::VelloFont;
pub use corner::Corner;
pub use edge::Edge;
pub use layout::*;
pub use units::Units;

use crate::font::TextAlign;

mod corner;
mod edge;
mod layout;
mod units;

/// A struct used to pass styles into a widget.
#[derive(Component, Reflect, Default, Debug, Clone, PartialEq)]
#[reflect(Component)]
pub struct WoodpeckerStyleProp(pub WoodpeckerStyle);

// A struct used to define the look of a widget
///
/// All fields are `pub`, so you can simply define your styles.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Copy)]
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
    /// Font handle if none is set the [`crate::DefaultFont`] is used.
    /// We use AssetId here because it can be copied thus it makes styles easier.
    pub font: Option<AssetId<VelloFont>>,
    /// The font size for this widget, in pixels
    ///
    /// Only applies to [`RenderCommand::Text`]
    pub font_size: f32,
    /// The layout method for children of this widget
    /// The line height for this widget, in pixels
    pub line_height: Option<f32>,
    /// The opacity of the widget and it's children
    /// Note: This will spawn a new UI render layer so use sparingly.
    pub opacity: f32,
    /// Alignent for text rendering
    /// If none is set it uses right for RTL and left for LTR text.
    pub text_alignment: Option<TextAlign>,
}

impl Default for WoodpeckerStyle {
    fn default() -> Self {
        WoodpeckerStyle::DEFAULT
    }
}

impl WoodpeckerStyle {
    /// Same as Default::default but constant.
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
        left: Units::Auto,
        right: Units::Auto,
        top: Units::Auto,
        bottom: Units::Auto,
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
        background_color: Color::Srgba(Srgba {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        }),
        border_color: Color::Srgba(Srgba {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        }),
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
        line_height: None,
        opacity: 1.0,
        font: None,
        text_alignment: None,
    };

    /// Lerps between two styles.
    ///
    /// Note: Only lerps: border_color, color, font_size, height, max_height, width,
    /// max_width, min_width, min_height, left, bottom, right, top, and opacity currrently.
    pub fn lerp(&self, b: &WoodpeckerStyle, x: f32) -> WoodpeckerStyle {
        let mut new_styles = *self; // Default to A styles.

        new_styles.background_color = hsv_lerp(&self.background_color, &b.background_color, x);

        // new_styles.border = Edge::new(
        //     lerp_units(self.border.top, b.top, x),
        //     lerp_units(self.border.right, b.right, x),
        //     lerp_units(self.border.bottom, b.bottom, x),
        //     lerp_units(self.border.left, b.left, x),
        // );

        new_styles.border_color = hsv_lerp(&self.border_color, &b.border_color, x);

        // new_styles.border_radius = Corner::new(
        //     lerp_units(self.border_radius.top_left, b.border_radius.top_left, x),
        //     lerp_units(self.border_radius.top_right, b.border_radius.top_right, x),
        //     lerp_units(
        //         self.border_radius.bottom_left,
        //         b.border_radius.bottom_left,
        //         x,
        //     ),
        //     lerp_units(
        //         self.border_radius.bottom_right,
        //         b.border_radius.bottom_right,
        //         x,
        //     ),
        // );

        new_styles.color = hsv_lerp(&self.color, &b.color, x);

        new_styles.font_size = lerp(self.font_size, b.font_size, x);
        new_styles.height = lerp_units(self.height, b.height, x);
        new_styles.max_height = lerp_units(self.max_height, b.max_height, x);
        new_styles.max_width = lerp_units(self.max_width, b.max_width, x);
        new_styles.min_height = lerp_units(self.min_height, b.min_height, x);
        new_styles.min_width = lerp_units(self.min_width, b.min_width, x);

        // new_styles.padding = Edge::new(
        //     lerp_units(self.padding.top, b.padding.top, x),
        //     lerp_units(self.padding.right, b.padding.right, x),
        //     lerp_units(self.padding.bottom, b.padding.bottom, x),
        //     lerp_units(self.padding.left, b.padding.left, x),
        // );

        new_styles.left = lerp_units(self.left, b.left, x);
        new_styles.right = lerp_units(self.right, b.right, x);
        new_styles.top = lerp_units(self.top, b.top, x);
        new_styles.bottom = lerp_units(self.bottom, b.bottom, x);
        new_styles.width = lerp_units(self.width, b.width, x);
        new_styles.opacity = lerp(self.opacity, b.opacity, x);

        new_styles
    }
}

impl From<&WoodpeckerStyle> for taffy::Style {
    fn from(val: &WoodpeckerStyle) -> taffy::Style {
        (*val).into()
    }
}

impl From<WoodpeckerStyle> for taffy::Style {
    fn from(val: WoodpeckerStyle) -> taffy::Style {
        taffy::Style {
            display: val.display.into(),
            overflow: taffy::Point {
                x: val.overflow.into(),
                y: val.overflow.into(),
            },
            position: val.position.into(),
            inset: Edge::new(val.top, val.right, val.bottom, val.left).into(),
            size: taffy::Size {
                width: val.width.into(),
                height: val.height.into(),
            },
            min_size: taffy::Size {
                width: val.min_width.into(),
                height: val.min_height.into(),
            },
            max_size: taffy::Size {
                width: val.max_width.into(),
                height: val.max_height.into(),
            },
            margin: val.margin.into(),
            padding: val.padding.into(),
            border: val.border.into(),
            align_items: val.align_items.map(|i| i.into()),
            align_self: val.align_self.map(|i| i.into()),
            justify_items: val.justify_items.map(|i| i.into()),
            justify_self: val.justify_self.map(|i| i.into()),
            align_content: val.align_content.map(|i| i.into()),
            justify_content: val.justify_content.map(|i| i.into()),
            gap: taffy::Size {
                width: val.gap.0.into(),
                height: val.gap.1.into(),
            },
            flex_direction: val.flex_direction.into(),
            flex_wrap: val.flex_wrap.into(),
            flex_basis: val.flex_basis.into(),
            flex_grow: val.flex_grow,
            flex_shrink: val.flex_shrink,
            ..Default::default()
        }
    }
}

fn lerp_units(prop_a: Units, prop_b: Units, x: f32) -> Units {
    match (prop_a, prop_b) {
        (Units::Pixels(a), Units::Pixels(b)) => Units::Pixels(lerp(a, b, x)),
        (Units::Percentage(a), Units::Percentage(b)) => Units::Percentage(lerp(a, b, x)),
        _ => {
            bevy::prelude::trace!(
                "Cannot lerp between non-matching units! Unit_A: {:?}, Unit_B: {:?}",
                prop_a,
                prop_b
            );
            prop_b
        }
    }
}

// fn lerp_ang(a: f32, b: f32, x: f32) -> f32 {
//     let ang = ((((a - b) % std::f32::consts::TAU) + std::f32::consts::PI * 3.)
//         % std::f32::consts::TAU)
//         - std::f32::consts::PI;
//     ang * x + b
// }

fn rgb_to_hsv(from: Srgba) -> Vec3 {
    // xyz <-> hsv
    let r = from.red;
    let g = from.green;
    let b = from.blue;

    let mut res = Vec3::ZERO;

    let min = r.min(g).min(b);
    let max = r.max(g).max(b);

    // Value
    res.z = max;

    let delta = max - min;
    // calc Saturation
    if max != 0.0 {
        res.y = delta / max;
    } else {
        res.x = -1.0;
        res.y = 0.0;

        return res;
    }

    // calc Hue
    if r == max {
        // between Yellow & Magenta
        res.x = (g - b) / delta;
    } else if g == max {
        // cyan to yellow
        res.x = 2.0 + (b - r) / delta;
    } else {
        // b == max // Megnta to cyan
        res.x = 4.0 + (r - g) / delta;
    }

    res.x *= 60.0; // Convert to degrees
    if res.x < 0.0 {
        res.x += 360.0; // Unwrap angle in case of negative
    }

    res
}

fn hsv_to_rgb(from: &Vec3) -> Srgba {
    let h = from.x;
    let s = from.y;
    let v = from.z;

    // Calc base values
    let c = s * v;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    let mut res = Vec4::new(0.0, 0.0, 0.0, 1.0);

    if (0.0..60.0).contains(&h) {
        res.x = c;
        res.y = x;
        res.z = 0.0;
    } else if (60.0..120.0).contains(&h) {
        res.x = x;
        res.y = c;
        res.z = 0.0;
    } else if (120.0..180.0).contains(&h) {
        res.x = 0.0;
        res.y = c;
        res.z = x;
    } else if (180.0..240.0).contains(&h) {
        res.x = 0.0;
        res.y = x;
        res.z = c;
    } else if (240.0..300.0).contains(&h) {
        res.x = x;
        res.y = 0.0;
        res.z = c;
    } else {
        res.x = c;
        res.y = 0.0;
        res.z = x;
    }

    res += Vec4::new(m, m, m, 0.0);

    Srgba::from_f32_array(res.to_array())
}

fn hsv_lerp(from: &Color, to: &Color, amount: f32) -> Color {
    let from_a = from.alpha();
    let to_a = to.alpha();
    let from = rgb_to_hsv(from.to_srgba());
    let to = rgb_to_hsv(to.to_srgba());
    let mut res = from.lerp(to, amount);

    if from.x < 0.0 {
        res.x = to.x;
    }
    let mut color: Color = hsv_to_rgb(&res).into();
    color.set_alpha(lerp(from_a, to_a, amount).clamp(0.0, 1.0));
    color
}

pub(crate) fn lerp(a: f32, b: f32, x: f32) -> f32 {
    a * (1.0 - x) + b * x
}
