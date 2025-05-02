use crate::{
    layout::system::{WidgetLayout, WidgetPreviousLayout},
    picking_backend::MouseWheelScroll,
    prelude::*,
};
use bevy::prelude::*;
// use bevy_mod_picking::{
//     events::Pointer,
//     prelude::{ListenerInput, On},
//     PickableBundle,
// };

use super::ScrollContext;

/// A widget that renders a scrollable "box" of content.
/// Requires that itself be wrapped by the [`super::ScrollContextProvider`]
#[derive(Widget, Component, Reflect, Default, Clone, PartialEq)]
#[auto_update(render)]
#[props(ScrollBox, PassedChildren, WidgetLayout)]
#[context(ScrollContext)]
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
    /// The thickness of the scrollbar
    pub scrollbar_thickness: Option<f32>,
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

/// A bundle for convince when creating the widget.
#[derive(Bundle, Default, Clone)]
pub struct ScrollBoxBundle {
    /// The scrollbox itself
    pub scroll_box: ScrollBox,
    /// Internal Styles
    ///
    /// Hint: To set the styles use fields in [`ScrollBox`]
    pub styles: WoodpeckerStyle,
    /// The internal children built by this widget.
    pub internal_children: WidgetChildren,
    /// The widgets you'd like to be scrollable.
    pub children: PassedChildren,
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

    let hide_horizontal =
        hide_horizontal || !always_show_scrollbar && scrollable_width < f32::EPSILON;
    let hide_vertical = hide_vertical || !always_show_scrollbar && scrollable_height < f32::EPSILON;

    let pad_x = if hide_vertical { 0.0 } else { vert_thickness };
    let pad_y = if hide_horizontal { 0.0 } else { hori_thickness };

    if pad_x != context.pad_x || pad_y != context.pad_y {
        context.pad_x = pad_x;
        context.pad_y = pad_y;
    }

    if prev_layout != layout {
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

    let scroll_content_bundle = ScrollContentBundle {
        children: passed_children.0.clone(),
        ..Default::default()
    };

    let mut vbox_children = WidgetChildren::default().with_child::<Clip>(ClipBundle {
        children: WidgetChildren::default().with_child::<ScrollContent>(scroll_content_bundle),
        ..Default::default()
    });

    if !hide_horizontal {
        vbox_children.add::<ScrollBar>(ScrollBarBundle {
            scroll_bar: ScrollBar {
                disabled: disable_horizontal,
                horizontal: true,
                thickness: hori_thickness,
                thumb_color,
                thumb_styles,
                track_color,
                track_styles,
            },
            ..Default::default()
        });
    }

    let mut element_wrapper_children =
        WidgetChildren::default().with_child::<Element>(ElementBundle {
            styles: vbox_styles,
            children: vbox_children,
            ..Default::default()
        });

    if !hide_vertical {
        element_wrapper_children.add::<ScrollBar>(ScrollBarBundle {
            scroll_bar: ScrollBar {
                disabled: disable_vertical,
                thickness: hori_thickness,
                thumb_color,
                thumb_styles,
                track_color,
                track_styles,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    children
        .add::<Element>((
            ElementBundle {
                styles: hbox_styles,
                children: element_wrapper_children,
                ..Default::default()
            },
            Pickable::default(),
        ))
        .observe(
            move |mut trigger: Trigger<Pointer<MouseWheelScroll>>,
                  mut context_query: Query<&mut ScrollContext>| {
                let x = trigger.scroll.x;
                let y = trigger.scroll.y;
                trigger.propagate(false);
                if let Ok(mut context) = context_query.get_mut(context_entity) {
                    let scroll_x = context.scroll_x();
                    let scroll_y = context.scroll_y();
                    if !disable_horizontal {
                        context.set_scroll_x(scroll_x - x * scroll_line);
                    }
                    if !disable_vertical {
                        context.set_scroll_y(scroll_y + y * scroll_line);
                    }
                }
            },
        );

    children.apply(current_widget.as_parent());
}
