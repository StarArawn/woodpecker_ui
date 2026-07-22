use crate::{
    layout::system::{WidgetLayout, WidgetPreviousLayout},
    picking_backend::MouseWheelScroll,
    prelude::*,
};
use bevy::prelude::*;

use super::ScrollContext;

/// How many consecutive frames an auto-shown scrollbar's raw overflow reading must agree
/// before its shown/hidden state actually flips. A still-reflowing subtree (e.g. `Table`
/// columns mid-resize) can genuinely, if briefly, overflow its container for a frame or two
/// while layout converges -- see `layout::system::run`'s own `SETTLE_FRAMES` for the same
/// class of issue -- so trusting any single frame's reading flickers the scrollbar on and off
/// during a continuous window resize.
const SCROLLBAR_SETTLE_FRAMES: u32 = 5;

/// Per-instance debounce state for [`ScrollBox`]'s auto-shown scrollbars -- see
/// `SCROLLBAR_SETTLE_FRAMES`. Deliberately not `DiffableProp`: written unconditionally every
/// render, which would make it permanently "changed" and defeat the point of the generic diff
/// system (mirrors `VirtualListScrollState`'s identical reasoning).
#[derive(Component, Default, Clone, PartialEq)]
pub struct ScrollBarVisibilityState {
    horizontal_shown: bool,
    horizontal_streak: u32,
    vertical_shown: bool,
    vertical_streak: u32,
}

impl ScrollBarVisibilityState {
    /// Only flips `shown` once `wants_shown` has agreed for `SCROLLBAR_SETTLE_FRAMES`
    /// consecutive calls; any disagreement resets the streak.
    fn debounce(shown: &mut bool, streak: &mut u32, wants_shown: bool) -> bool {
        if wants_shown == *shown {
            *streak = 0;
        } else {
            *streak += 1;
            if *streak >= SCROLLBAR_SETTLE_FRAMES {
                *shown = wants_shown;
                *streak = 0;
            }
        }
        *shown
    }
}

/// A widget that renders a scrollable "box" of content.
/// Requires that itself be wrapped by the [`super::ScrollContextProvider`]
#[derive(Widget, Component, Reflect, Default, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, PassedChildren, WatchLayout)]
pub struct ScrollBox {
    /// If true, always shows scrollbars even when there's nothing to scroll
    ///
    /// Individual scrollbars can still be hidden via [`hide_horizontal`](Self::hide_horizontal)
    /// and [`hide_vertical`](Self::hide_vertical).
    pub always_show_scrollbar: bool,
    /// If true, disables horizontal scrolling
    pub disable_horizontal: bool,
    /// If true, disables vertical scrolling
    pub disable_vertical: bool,
    /// If true, hides the horizontal scrollbar
    pub hide_horizontal: bool,
    /// If true, hides the vertical scrollbar
    pub hide_vertical: bool,
    /// The thickness of the entire scrollbar in pixels
    pub scrollbar_thickness: Option<f32>,
    /// The thickness of the thumb in pixels
    pub thumb_thickness: Option<f32>,
    /// The step to scroll by when `ScrollUnit::Line`
    pub scroll_line: Option<f32>,
    /// The color of the scrollbar thumb
    pub thumb_color: Option<Color>,
    /// The styles of the scrollbar thumb
    pub thumb_styles: Option<WoodpeckerStyle>,
    /// The color of the scrollbar track
    pub track_color: Option<Color>,
    /// The styles of the scrollbar track
    pub track_styles: Option<WoodpeckerStyle>,
}

pub fn render(
    mut commands: Commands,
    mut context_helper: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &ScrollBox,
        &PassedChildren,
        &mut WidgetChildren,
        &mut WoodpeckerStyle,
        &WidgetLayout,
        &WidgetPreviousLayout,
    )>,
    mut context_query: Query<&mut ScrollContext>,
    mut visibility_state_query: Query<&mut ScrollBarVisibilityState>,
) {
    let Ok((scroll_box, passed_children, mut children, mut styles, layout, prev_layout)) =
        query.get_mut(**current_widget)
    else {
        return;
    };
    let context_entity =
        context_helper.use_context(&mut commands, *current_widget, ScrollContext::default());

    let Ok(mut context) = context_query.get_mut(context_entity) else {
        return;
    };

    let visibility_state_entity = context_helper.use_state(
        &mut commands,
        *current_widget,
        ScrollBarVisibilityState::default(),
    );
    let Ok(mut visibility) = visibility_state_query.get_mut(visibility_state_entity) else {
        return;
    };
    let visibility = &mut *visibility;

    // === Configuration === //
    let always_show_scrollbar = scroll_box.always_show_scrollbar;
    let disable_horizontal = scroll_box.disable_horizontal;
    let disable_vertical = scroll_box.disable_vertical;
    let hide_horizontal = scroll_box.hide_horizontal;
    let hide_vertical = scroll_box.hide_vertical;
    let scrollbar_thickness = scroll_box.scrollbar_thickness.unwrap_or(10.0);
    let scroll_line = scroll_box.scroll_line.unwrap_or(64.0);
    let thumb_color = scroll_box.thumb_color;
    let thumb_styles = scroll_box.thumb_styles;
    let track_color = scroll_box.track_color;
    let track_styles = scroll_box.track_styles;

    let scrollable_width = context.scrollable_width();
    let scrollable_height = context.scrollable_height();

    let hori_thickness = scrollbar_thickness;
    let vert_thickness = scrollbar_thickness;

    let auto_horizontal_visible = ScrollBarVisibilityState::debounce(
        &mut visibility.horizontal_shown,
        &mut visibility.horizontal_streak,
        scrollable_width >= f32::EPSILON,
    );
    let auto_vertical_visible = ScrollBarVisibilityState::debounce(
        &mut visibility.vertical_shown,
        &mut visibility.vertical_streak,
        scrollable_height >= f32::EPSILON,
    );

    let hide_horizontal = hide_horizontal || !always_show_scrollbar && !auto_horizontal_visible;
    let hide_vertical = hide_vertical || !always_show_scrollbar && !auto_vertical_visible;

    let pad_x = if hide_vertical { 0.0 } else { vert_thickness };
    let pad_y = if hide_horizontal { 0.0 } else { hori_thickness };

    if pad_x != context.pad_x || pad_y != context.pad_y {
        context.pad_x = pad_x;
        context.pad_y = pad_y;
    }

    // See the matching comment in `scroll/content.rs`: ignore a degenerate `(0, 0)` layout
    // rather than learning it as the real viewport size.
    if prev_layout != layout && layout.size != Vec2::ZERO {
        context.scrollbox_width = layout.width();
        context.scrollbox_height = layout.height();
    }

    *styles = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        margin: Edge::all(0.0).right(scrollbar_thickness / 2.0),
        ..*styles
    };

    let hbox_styles = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        ..Default::default()
    };
    let vbox_styles = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        ..Default::default()
    };

    let scroll_content_bundle = (ScrollContent, passed_children.0.clone());

    let mut vbox_children = WidgetChildren::default().with_child::<Clip>((
        Clip,
        WidgetChildren::default().with_child::<ScrollContent>(scroll_content_bundle),
    ));

    if !hide_horizontal {
        vbox_children.add::<ScrollBar>(ScrollBar {
            disabled: disable_horizontal,
            horizontal: true,
            thickness: hori_thickness,
            thumb_thickness: scroll_box.thumb_thickness,
            thumb_color,
            thumb_styles,
            track_color,
            track_styles,
        });
    }

    let mut element_wrapper_children =
        WidgetChildren::default().with_child::<Element>((Element, vbox_styles, vbox_children));

    if !hide_vertical {
        element_wrapper_children.add::<ScrollBar>(ScrollBar {
            disabled: disable_vertical,
            thickness: hori_thickness,
            thumb_thickness: scroll_box.thumb_thickness,
            thumb_color,
            thumb_styles,
            track_color,
            track_styles,
            ..Default::default()
        });
    }

    children
        .add::<Element>((
            Element,
            hbox_styles,
            element_wrapper_children,
            Pickable::default(),
        ))
        .observe(
            *current_widget,
            move |mut trigger: On<Pointer<MouseWheelScroll>>,
                  mut context_query: Query<&mut ScrollContext>| {
                let delta = trigger.pixel_delta(scroll_line);
                trigger.propagate(false);
                if let Ok(mut context) = context_query.get_mut(context_entity) {
                    let scroll_x = context.scroll_x();
                    let scroll_y = context.scroll_y();
                    if !disable_horizontal {
                        context.set_scroll_x(scroll_x - delta.x);
                    }
                    if !disable_vertical {
                        context.set_scroll_y(scroll_y + delta.y);
                    }
                }
            },
        );

    children.apply(current_widget.as_parent());
}
