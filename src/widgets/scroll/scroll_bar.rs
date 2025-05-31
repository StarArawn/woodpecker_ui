use crate::{picking_backend::compute_letterboxed_transform, prelude::*};
use bevy::{prelude::*, window::PrimaryWindow};
// use bevy_mod_picking::{
//     events::{Click, Drag, DragEnd, DragStart, Drop, Pointer},
//     prelude::{ListenerMut, On, Pickable},
//     PickableBundle,
// };

use super::{map_range, ScrollContext};

/// [`ScrollBar`] widget
#[derive(Component, Widget, Reflect, Default, Debug, PartialEq, Clone)]
#[auto_update(render)]
#[props(ScrollBar, WidgetLayout)]
#[context(ScrollContext)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub struct ScrollBar {
    /// If true, disables the ability to drag
    pub disabled: bool,
    /// If true, displays a horizontal scrollbar instead of a vertical one
    pub horizontal: bool,
    /// The thickness of the entire scrollbar in pixels
    pub thickness: f32,
    /// The thickness of the bar in pixels
    pub thumb_thickness: Option<f32>,
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
        context_helper.use_context(&mut commands, *current_widget, ScrollContext::default());

    let Ok(context) = context_query.get(context_entity) else {
        return;
    };

    let layout = *layout;
    let content_width = context.content_width();
    let content_height = context.content_height();

    // === Configuration === //
    // let disabled = scrollbar.disabled;
    let horizontal = scrollbar.horizontal;
    let thickness = scrollbar.thickness;
    let thumb_color = scrollbar
        .thumb_color
        .unwrap_or_else(|| Color::srgba(0.239, 0.258, 0.337, 1.0));
    let thumb_styles = scrollbar.thumb_styles;
    let thumb_thickness = scrollbar.thumb_thickness;
    let track_color = scrollbar
        .track_color
        .unwrap_or_else(|| Color::srgba(0.1581, 0.1758, 0.191, 0.15));
    let track_styles = scrollbar.track_styles.unwrap_or(WoodpeckerStyle {
        background_color: track_color,
        border_radius: Corner::all(thickness / 2.0),
        ..Default::default()
    });

    let mut border_color = thumb_color;
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

    // The size of the thumb as a percentage
    let thumb_size_percent = (if scrollbar.horizontal {
        layout.width() / (content_width - thickness).max(1.0)
    } else {
        layout.height() / (content_height - thumb_thickness.unwrap_or(thickness)).max(1.0)
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
        (0.0, 99.5 - thumb_size_percent * 100.0),
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

    if let Color::Srgba(srgba) = &mut border_color {
        srgba.alpha = (srgba.alpha - 0.2).max(0.0);
        srgba.red = (srgba.red + 0.1).min(1.0);
        srgba.green = (srgba.green + 0.1).min(1.0);
        srgba.blue = (srgba.blue + 0.1).min(1.0);
    }

    let mut track_style = WoodpeckerStyle { ..track_styles };

    if scrollbar.horizontal {
        track_style = WoodpeckerStyle {
            height: Units::Pixels(thickness),
            width: Units::Percentage(100.0),
            ..track_style
        };
        thumb_style = WoodpeckerStyle {
            height: Units::Pixels(thumb_thickness.unwrap_or(thickness)),
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
            width: Units::Pixels(thumb_thickness.unwrap_or(thickness)),
            height: Units::Percentage(thumb_size_percent * 100.0),
            left: Units::Pixels(0.0),
            top: Units::Percentage(-thumb_offset),
            ..thumb_style
        };
    }

    let current_widget = *current_widget;
    children.add::<Element>((
        Element,
            track_style,
            WidgetChildren::default().with_child::<Clip>((
                Clip,
                WidgetChildren::default()
                    .with_child::<Element>((
                        Element,
                        thumb_style,
                        Pickable::default(),
                        WidgetRender::Quad,
                    ))
                    .with_observe(
                        current_widget,|mut trigger: Trigger<Pointer<Click>>| {
                        trigger.propagate(false);
                    })
                    .with_observe(
                        current_widget,
                        |trigger: Trigger<Pointer<DragStart>>, mut commands: Commands| {
                            commands.entity(trigger.target).insert(Pickable::IGNORE);
                        },
                    )
                    .with_observe(
                        current_widget,
                        |trigger: Trigger<Pointer<DragEnd>>, mut commands: Commands| {
                            commands.entity(trigger.target).insert(Pickable::default());
                        },
                    )
                    .with_observe(
                        current_widget,
                        move |trigger: Trigger<Pointer<Drag>>,
                        layout_query: Query<&WidgetLayout>,
                        window: Single<&Window, With<PrimaryWindow>>,
                        camera: Query<&Camera, With<WoodpeckerView>>,
                         mut context_query: Query<&mut ScrollContext>| {
                            let Ok(mut context) = context_query.get_mut(context_entity) else {
                                return;
                            };

                            let Ok(layout) = layout_query.get(*current_widget) else {
                                return;
                            };

                            let Some(camera) = camera.iter().next() else {
                                return;
                            };

                            let (offset, size, _scale) = compute_letterboxed_transform(
                                window.size(),
                                camera.logical_target_size().unwrap(),
                            );

                            let cursor_pos_world =
                                ((trigger.pointer_location.position - offset) / size) * camera.logical_target_size().unwrap();


                            // The size of the thumb as a percentage
                            let content_width = context.content_width();
                            let content_height = context.content_height();
                            let thumb_size_percent = if horizontal {
                                layout.width() / (content_width - thickness).max(1.0)
                            } else {
                                layout.height() / (content_height - thickness).max(1.0)
                            };

                            // --- Move Thumb --- //
                            // Positional difference (scaled by thumb size)
                            let pos_diff = (
                                (context.start_pos.x
                                    - (cursor_pos_world.x - layout.location.x))
                                    / thumb_size_percent,
                                (context.start_pos.y
                                    - (cursor_pos_world.y - layout.location.y))
                                    / thumb_size_percent,
                            );

                            let start_offset = context.start_offset;
                            if horizontal {
                                context.set_scroll_x(start_offset.x + pos_diff.0);
                            } else {
                                context.set_scroll_y(start_offset.y + pos_diff.1);
                            }
                        },
                    ),
            )),
        Pickable::default(),
        WidgetRender::Quad,
    )).observe(
        current_widget,move |
            trigger: Trigger<Pointer<Click>>, 
            layout_query: Query<&WidgetLayout>,
            window: Single<&Window, With<PrimaryWindow>>,
            camera: Query<&Camera, With<WoodpeckerView>>,
            mut context_query: Query<&mut ScrollContext>| {
        let Ok(mut context) = context_query.get_mut(context_entity) else {
            return;
        };

        let Ok(layout) = layout_query.get(*current_widget) else {
            return;
        };

        let Some(camera) = camera.iter().next() else {
            return;
        };

        let (offset, size, _scale) = compute_letterboxed_transform(
            window.size(),
            camera.logical_target_size().unwrap(),
        );

        let cursor_pos_world =
            ((trigger.pointer_location.position - offset) / size) * camera.logical_target_size().unwrap();

        // --- Move Thumb --- //
        // Positional difference (scaled by thumb size)
        let pos_diff = (
            (context.start_pos.x - (cursor_pos_world.x - layout.location.x))
                / thumb_size_percent,
            (context.start_pos.y - (cursor_pos_world.y - layout.location.y))
                / thumb_size_percent,
        );

        let start_offset = context.start_offset;
        if horizontal {
            context.set_scroll_x(start_offset.x + pos_diff.0);
        } else {
            context.set_scroll_y(start_offset.y + pos_diff.1);
        }
    });

    children.apply(current_widget.as_parent());
}
