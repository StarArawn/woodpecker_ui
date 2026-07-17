use bevy::prelude::*;
use woodpecker_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .insert_resource(SpringTargets {
            points: vec![
                Vec2::new(600.0, 50.0),
                Vec2::new(600.0, 300.0),
                Vec2::new(850.0, 300.0),
                Vec2::new(850.0, 50.0),
            ],
            index: 0,
        })
        .add_systems(Startup, startup)
        .add_systems(Update, apply_spring_to_style)
        .run();
}

#[derive(Component, Clone)]
struct SpringBox;

#[derive(Resource)]
struct SpringTargets {
    points: Vec<Vec2>,
    index: usize,
}

/// Only writes `left`/`top` (and so only marks `WoodpeckerStyle` as `Changed`) while the spring
/// hasn't yet settled at its target
fn apply_spring_to_style(mut query: Query<(&Spring, &mut WoodpeckerStyle), With<SpringBox>>) {
    for (spring, mut style) in query.iter_mut() {
        let next_left = Units::Pixels(spring.value.x);
        let next_top = Units::Pixels(spring.value.y);
        if style.left != next_left || style.top != next_top {
            style.left = next_left;
            style.top = next_top;
        }
    }
}

fn animation_timeline_track(
    index: usize,
    quad_styles: WoodpeckerStyle,
) -> (WoodpeckerStyle, AnimationTimeline) {
    let top = 50.0 + index as f32 * 130.0;
    let base_style = WoodpeckerStyle {
        top: top.into(),
        ..quad_styles
    };

    let width_track = AnimationTrack {
        easing: TransitionEasing::QuadraticInOut,
        delay: 0.0,
        duration: 700.0,
        style_a: WoodpeckerStyle {
            width: 80.0.into(),
            ..base_style
        },
        style_b: WoodpeckerStyle {
            width: 260.0.into(),
            ..base_style
        },
    };
    let color_track = AnimationTrack {
        easing: TransitionEasing::CubicInOut,
        delay: 200.0,
        duration: 700.0,
        style_a: WoodpeckerStyle {
            background_color: Srgba::new(0.2, 0.8, 0.4, 1.0).into(),
            ..base_style
        },
        style_b: WoodpeckerStyle {
            background_color: Srgba::new(0.8, 0.2, 0.6, 1.0).into(),
            ..base_style
        },
    };

    let timeline = AnimationTimeline {
        playing: true,
        looping: true,
        tracks: vec![width_track, color_track],
        start: AnimationGroup::staggered_start(index, 300.0),
    };

    (base_style, timeline)
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let quad_styles = WoodpeckerStyle {
        position: WidgetPosition::Absolute,
        left: 50.0.into(),
        top: 50.0.into(),
        width: 80.0.into(),
        height: 80.0.into(),
        ..Default::default()
    };

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            font_size: 24.0,
            color: Srgba::WHITE.into(),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>({
                let (style, timeline) = animation_timeline_track(0, quad_styles);
                (Element, WidgetRender::Quad, style, timeline)
            })
            .with_key("timeline_0")
            .with_child::<Element>({
                let (style, timeline) = animation_timeline_track(1, quad_styles);
                (Element, WidgetRender::Quad, style, timeline)
            })
            .with_key("timeline_1")
            .with_child::<Element>({
                let (style, timeline) = animation_timeline_track(2, quad_styles);
                (Element, WidgetRender::Quad, style, timeline)
            })
            .with_key("timeline_2")
            .with_child::<Element>((
                Element,
                WidgetRender::Quad,
                SpringBox,
                Spring {
                    value: Vec2::new(600.0, 50.0),
                    target: Vec2::new(600.0, 50.0),
                    ..Default::default()
                },
                WoodpeckerStyle {
                    background_color: Srgba::new(0.3, 0.5, 0.9, 1.0).into(),
                    ..quad_styles
                },
            ))
            .with_key("spring_box")
            .with_child::<WButton>((
                WButton,
                WoodpeckerStyle {
                    position: WidgetPosition::Absolute,
                    left: 50.0.into(),
                    top: 450.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Send spring box to next corner".into(),
                    },
                )),
            ))
            .with_observe(
                root_widget,
                |_: On<Pointer<Click>>,
                 mut targets: ResMut<SpringTargets>,
                 mut spring_query: Query<&mut Spring, With<SpringBox>>| {
                    targets.index = (targets.index + 1) % targets.points.len();
                    let next = targets.points[targets.index];
                    for mut spring in spring_query.iter_mut() {
                        spring.set_target(next);
                    }
                },
            )
            .with_key("spring_button"),
    ));
}
