use crate::prelude::*;
use bevy::prelude::*;
use bevy_mod_picking::{
    events::{Click, Drag, DragEnd, DragStart, Drop, Pointer},
    prelude::{ListenerMut, On, Pickable},
    PickableBundle,
};

use super::{map_range, ScrollContext};

/// [`ScrollBar`] widget
#[derive(Component, Widget, Default, Debug, PartialEq, Clone)]
#[widget_systems(update, render)]
pub struct ScrollBar {
    /// If true, disables the ability to drag
    pub disabled: bool,
    /// If true, displays a horizontal scrollbar instead of a vertical one
    pub horizontal: bool,
    /// The thickness of the scrollbar in pixels
    pub thickness: f32,
    /// The color of the scrollbar thumb
    pub thumb_color: Option<Color>,
    /// The styles of the scrollbar thumb
    pub thumb_styles: Option<WoodpeckerStyle>,
    /// The color of the scrollbar track
    pub track_color: Option<Color>,
    /// The styles of the scrollbar track
    pub track_styles: Option<WoodpeckerStyle>,
}

#[derive(Bundle, Clone, Default)]
pub struct ScrollBarBundle {
    pub scroll_bar: ScrollBar,
    pub styles: WoodpeckerStyle,
    pub children: WidgetChildren,
}

pub fn update(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut context_helper: ResMut<HookHelper>,
    query: Query<(
        Ref<ScrollBar>,
        Ref<WidgetChildren>,
        Ref<WidgetLayout>,
        Ref<WidgetPreviousLayout>,
    )>,
    context_query: Query<Entity, Changed<ScrollContext>>,
) -> bool {
    let Ok((sb, children, layout, prev_layout)) = query.get(**current_widget) else {
        return false;
    };

    let context_entity =
        context_helper.use_context::<ScrollContext>(&mut commands, *current_widget);

    sb.is_changed()
        || children.children_changed()
        || *prev_layout != *layout
        || context_query.contains(context_entity)
}

pub fn render(
    mut commands: Commands,
    mut context_helper: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &ScrollBar,
        &mut WidgetChildren,
        &mut WoodpeckerStyle,
        &WidgetLayout,
    )>,
    context_query: Query<&ScrollContext>,
) {
    let Ok((scrollbar, mut children, mut styles, layout)) = query.get_mut(**current_widget) else {
        return;
    };
    let context_entity =
        context_helper.use_context::<ScrollContext>(&mut commands, *current_widget);

    let Ok(context) = context_query.get(context_entity) else {
        return;
    };

    let layout = *layout;
    let content_width = context.content_width();
    let content_height = context.content_height();

    // === Configuration === //
    // let disabled = scrollbar.disabled;
    let horizontal = scrollbar.horizontal;
    let _thickness = scrollbar.thickness;
    let thickness = scrollbar.thickness;
    let thumb_color = scrollbar
        .thumb_color
        .unwrap_or_else(|| Color::srgba(0.239, 0.258, 0.337, 1.0));
    let thumb_styles = scrollbar.thumb_styles;
    let track_color = scrollbar
        .track_color
        .unwrap_or_else(|| Color::srgba(0.1581, 0.1758, 0.191, 0.15));
    let track_styles = scrollbar.track_styles.unwrap_or(WoodpeckerStyle {
        background_color: track_color,
        border_radius: Corner::all(thickness / 2.0),
        ..Default::default()
    });
    // The size of the thumb as a percentage
    let thumb_size_percent = (if scrollbar.horizontal {
        layout.width() / (content_width - thickness).max(1.0)
    } else {
        layout.height() / (content_height - thickness).max(1.0)
    })
    .clamp(0.1, 1.0);
    let percent_scrolled = if scrollbar.horizontal {
        context.percent_x()
    } else {
        context.percent_y()
    };
    // The thumb's offset as a percentage
    let thumb_offset = map_range(
        percent_scrolled * 100.0,
        (0.0, 100.0),
        (0.0, 100.0 - thumb_size_percent * 100.0),
    );

    *styles = WoodpeckerStyle {
        width: if horizontal {
            Units::Percentage(100.0)
        } else {
            Units::Pixels(thickness)
        },
        height: if horizontal {
            Units::Pixels(thickness)
        } else {
            Units::Percentage(100.0)
        },
        ..*styles
    };

    let mut border_color = thumb_color;
    if let Color::Srgba(srgba) = &mut border_color {
        srgba.alpha = (srgba.alpha - 0.2).max(0.0);
        srgba.red = (srgba.red + 0.1).min(1.0);
        srgba.green = (srgba.green + 0.1).min(1.0);
        srgba.blue = (srgba.blue + 0.1).min(1.0);
    }

    let mut thumb_style = WoodpeckerStyle {
        position: WidgetPosition::Absolute,
        ..thumb_styles.unwrap_or(WoodpeckerStyle {
            background_color: thumb_color,
            border_radius: Corner::all(thickness / 2.0),
            border: Edge::all(1.0),
            border_color,
            ..Default::default()
        })
    };

    let mut track_style = WoodpeckerStyle {
        background_color: track_color,
        border_radius: Corner::all(thickness / 2.0),
        ..track_styles
    };

    if scrollbar.horizontal {
        track_style = WoodpeckerStyle {
            height: Units::Pixels(thickness),
            width: Units::Percentage(100.0),
            ..track_style
        };
        thumb_style = WoodpeckerStyle {
            height: Units::Pixels(thickness),
            width: Units::Percentage(thumb_size_percent * 100.0),
            top: Units::Pixels(0.0),
            left: Units::Percentage(-thumb_offset),
            ..thumb_style
        };
    } else {
        track_style = WoodpeckerStyle {
            width: Units::Pixels(thickness),
            height: Units::Percentage(100.0),
            ..track_style
        };
        thumb_style = WoodpeckerStyle {
            width: Units::Pixels(thickness),
            height: Units::Percentage(thumb_size_percent * 100.0),
            left: Units::Pixels(0.0),
            top: Units::Percentage(-thumb_offset),
            ..thumb_style
        };
    }

    children.add::<Element>((
        ElementBundle {
            styles: track_style,
            children: WidgetChildren::default().with_child::<Clip>(ClipBundle {
                children: WidgetChildren::default().with_child::<Element>((
                    ElementBundle {
                        styles: thumb_style,
                        ..Default::default()
                    },
                    PickableBundle::default(),
                    On::<Pointer<Click>>::run(|mut event: ListenerMut<Pointer<Click>>| {
                        event.stop_propagation();
                    }),
                    On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
                    On::<Pointer<DragEnd>>::target_insert(Pickable::default()), // Re-enable picking
                    On::<Pointer<Drop>>::run(|| {}),
                    On::<Pointer<Drag>>::run(
                        move |event: ListenerMut<Pointer<Drag>>,
                              mut context_query: Query<&mut ScrollContext>| {
                            let Ok(mut context) = context_query.get_mut(context_entity) else {
                                return;
                            };

                            // --- Move Thumb --- //
                            // Positional difference (scaled by thumb size)
                            let pos_diff = (
                                (context.start_pos.x - (event.pointer_location.position.x - layout.location.x)) / thumb_size_percent,
                                (context.start_pos.y - (event.pointer_location.position.y - layout.location.y)) / thumb_size_percent,
                            );

                            let start_offset = context.start_offset;
                            if horizontal {
                                context.set_scroll_x(start_offset.x + pos_diff.0);
                            } else {
                                context.set_scroll_y(start_offset.y + pos_diff.1);
                            }
                        },
                    ),
                    WidgetRender::Quad,
                )),
                ..Default::default()
            }),
            ..Default::default()
        },
        PickableBundle::default(),
        On::<Pointer<Click>>::run(move |event: ListenerMut<Pointer<Click>>,
              mut context_query: Query<&mut ScrollContext>| {
                  let Ok(mut context) = context_query.get_mut(context_entity) else {
                      return;
                  };

                  // --- Move Thumb --- //
                  // Positional difference (scaled by thumb size)
                  let pos_diff = (
                      (context.start_pos.x - (event.pointer_location.position.x - layout.location.x)) / thumb_size_percent,
                      (context.start_pos.y - (event.pointer_location.position.y  - layout.location.y)) / thumb_size_percent,
                  );

                  let start_offset = context.start_offset;
                  if horizontal {
                      context.set_scroll_x(start_offset.x + pos_diff.0);
                  } else {
                      context.set_scroll_y(start_offset.y + pos_diff.1);
                  }
              }),
        WidgetRender::Quad,
    ));

    children.apply(current_widget.as_parent());
}
