use std::time::Duration;

use bevy::{platform::time::Instant, prelude::*};
use interpolation::Ease;

use crate::prelude::*;

/// A single interpolated segment of an [`AnimationTimeline`].
///
/// Unlike [`Transition`], a track only ever plays forward once per timeline run: `delay`
/// (in milliseconds) is how long the track waits before it starts easing from `style_a` to
/// `style_b` over `duration` milliseconds. Multiple tracks on the same timeline compose by
/// each only touching the style fields where its own `style_a` and `style_b` differ, so
/// disjoint tracks (e.g. one animating `width`, another `opacity`) never clobber each other.
#[derive(Debug, Reflect, Clone, Copy, PartialEq)]
pub struct AnimationTrack {
    /// The easing function that dictates the interpolation factor.
    pub easing: TransitionEasing,
    /// How long (in milliseconds) this track waits before it starts playing.
    pub delay: f32,
    /// How long (in milliseconds) this track takes to play once its delay has elapsed.
    pub duration: f32,
    /// The starting styles of this track.
    pub style_a: WoodpeckerStyle,
    /// The ending styles of this track.
    pub style_b: WoodpeckerStyle,
}

impl AnimationTrack {
    fn progress(&self, elapsed_ms: f32) -> f32 {
        if self.duration <= 0.0 {
            return if elapsed_ms >= self.delay { 1.0 } else { 0.0 };
        }

        let local = ((elapsed_ms - self.delay) / self.duration).clamp(0.0, 1.0);
        if let Some(easing) = self.easing.try_into_easing_function() {
            Ease::calc(local, easing)
        } else {
            local
        }
    }

    fn merge_into(&self, base: WoodpeckerStyle, elapsed_ms: f32) -> WoodpeckerStyle {
        let x = self.progress(elapsed_ms);
        let resolved = self.style_a.lerp(&self.style_b, x);
        let mut merged = base;

        if self.style_a.background_color != self.style_b.background_color {
            merged.background_color = resolved.background_color;
        }
        if self.style_a.border_color != self.style_b.border_color {
            merged.border_color = resolved.border_color;
        }
        if self.style_a.color != self.style_b.color {
            merged.color = resolved.color;
        }
        if self.style_a.font_size != self.style_b.font_size {
            merged.font_size = resolved.font_size;
        }
        if self.style_a.height != self.style_b.height {
            merged.height = resolved.height;
        }
        if self.style_a.max_height != self.style_b.max_height {
            merged.max_height = resolved.max_height;
        }
        if self.style_a.max_width != self.style_b.max_width {
            merged.max_width = resolved.max_width;
        }
        if self.style_a.min_height != self.style_b.min_height {
            merged.min_height = resolved.min_height;
        }
        if self.style_a.min_width != self.style_b.min_width {
            merged.min_width = resolved.min_width;
        }
        if self.style_a.left != self.style_b.left {
            merged.left = resolved.left;
        }
        if self.style_a.right != self.style_b.right {
            merged.right = resolved.right;
        }
        if self.style_a.top != self.style_b.top {
            merged.top = resolved.top;
        }
        if self.style_a.bottom != self.style_b.bottom {
            merged.bottom = resolved.bottom;
        }
        if self.style_a.width != self.style_b.width {
            merged.width = resolved.width;
        }
        if self.style_a.opacity != self.style_b.opacity {
            merged.opacity = resolved.opacity;
        }

        merged
    }

    fn total_duration(&self) -> f32 {
        self.delay + self.duration
    }
}

/// A bevy component that animates a widget's [`WoodpeckerStyle`] across multiple, independently
/// timed and delayed [`AnimationTrack`]s.
///
/// Where [`Transition`] tweens the whole style between two endpoints, `AnimationTimeline` lets
/// several tracks play in parallel (optionally staggered via `delay`, see [`AnimationGroup`])
/// and fold together onto whatever [`WoodpeckerStyle`] the widget already has.
#[derive(Component, Debug, Reflect, Clone)]
pub struct AnimationTimeline {
    /// Indicates the current playing status.
    pub playing: bool,
    /// Does the animation loop once every track finishes?
    pub looping: bool,
    /// The tracks that make up this timeline.
    pub tracks: Vec<AnimationTrack>,
    /// The start time of the animation.
    pub start: Instant,
}

impl PartialEq for AnimationTimeline {
    fn eq(&self, other: &Self) -> bool {
        self.playing == other.playing && self.looping == other.looping && self.tracks == other.tracks
    }
}

impl Default for AnimationTimeline {
    fn default() -> Self {
        Self {
            playing: true,
            looping: false,
            tracks: Vec::new(),
            start: Instant::now(),
        }
    }
}

impl AnimationTimeline {
    fn total_duration(&self) -> f32 {
        self.tracks
            .iter()
            .map(AnimationTrack::total_duration)
            .fold(0.0, f32::max)
    }

    pub(crate) fn update(&mut self, current: WoodpeckerStyle) -> WoodpeckerStyle {
        let elapsed_time = self.start.elapsed().as_secs_f32() * 1000.0;
        let total = self.total_duration();

        let elapsed_for_fold = if self.playing && elapsed_time < total {
            elapsed_time
        } else if self.looping && self.playing {
            self.start = Instant::now();
            0.0
        } else {
            self.playing = false;
            total
        };

        self.tracks
            .iter()
            .fold(current, |style, track| track.merge_into(style, elapsed_for_fold))
    }

    /// Is the animation currently playing?
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// (Re)starts the animation from its first frame.
    pub fn start(&mut self) {
        self.start = Instant::now();
        self.playing = true;
    }
}

pub(crate) fn update_animation_timelines(
    mut query: Query<(&mut AnimationTimeline, &mut WoodpeckerStyle)>,
) {
    for (mut timeline, mut styles) in query.iter_mut() {
        let current = *styles;
        *styles = timeline.update(current);
    }
}

/// Snapshot of the last-diffed [`AnimationTimeline::is_playing`] value for a widget entity.
///
/// Mirrors [`crate::widgets::transition::PreviousTransitionPlaying`]: `AnimationTimeline`
/// deliberately never opts into [`crate::diffable_prop::DiffableProp`] since its `start: Instant`
/// field would otherwise make it compare as "changed" every frame while playing.
#[derive(Component, Default)]
pub(crate) struct PreviousAnimationTimelinePlaying(bool);

/// Returns `true` (and updates the snapshot) if `entity` has an `AnimationTimeline` component
/// whose playing-state changed since the last check. A widget with no `AnimationTimeline` at
/// all is simply never affected by this check.
pub(crate) fn diff_animation_timeline(world: &mut World, entity: Entity) -> bool {
    let Some(timeline) = world.get::<AnimationTimeline>(entity) else {
        return false;
    };
    let is_playing = timeline.is_playing();

    let changed = match world.get::<PreviousAnimationTimelinePlaying>(entity) {
        Some(previous) => previous.0 != is_playing,
        None => true,
    };

    if changed {
        world
            .entity_mut(entity)
            .insert(PreviousAnimationTimelinePlaying(is_playing));
    }

    changed
}

/// Marks an entity as continuously animating outside the normal prop-diffing render path, so
/// `vello_renderer::run`'s frame-skip cache knows it can't skip a frame while this is present --
/// e.g. a not-yet-settled [`Spring`] or an indeterminate [`crate::widgets::spinner::Spinner`],
/// both of which read live state (physics integration, wall-clock time) from inside a render
/// closure or update system rather than from a value `Changed<WoodpeckerStyle>` would catch.
/// Any future primitive with the same shape (state that changes every frame without touching a
/// diffable component) should insert/remove this on itself rather than the frame-skip cache
/// growing another bespoke per-type query.
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq, Default)]
pub struct Animating;

/// A critically damped-ish spring simulation over a [`Vec2`], driven by [`update_springs`] every
/// frame. Unlike [`Transition`]/[`AnimationTimeline`], a spring has no fixed duration: it simply
/// integrates towards `target` until it settles, which makes it suited to things like
/// pointer-follow or drag-release physics rather than authored keyframe animation.
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq)]
pub struct Spring {
    /// The value the spring is moving towards.
    pub target: Vec2,
    /// The spring's current value.
    pub value: Vec2,
    /// The spring's current velocity.
    pub velocity: Vec2,
    /// How strongly the spring pulls `value` towards `target`.
    pub stiffness: f32,
    /// How strongly the spring resists its own velocity.
    pub damping: f32,
    /// The simulated mass of the value being sprung.
    pub mass: f32,
}

impl Default for Spring {
    fn default() -> Self {
        Self {
            target: Vec2::ZERO,
            value: Vec2::ZERO,
            velocity: Vec2::ZERO,
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
        }
    }
}

impl Spring {
    const SETTLE_EPSILON: f32 = 0.01;

    /// Instantly moves the spring to `value`, clearing its target and velocity.
    pub fn snap_to(&mut self, value: Vec2) {
        self.target = value;
        self.value = value;
        self.velocity = Vec2::ZERO;
    }

    /// Sets a new target for the spring to move towards.
    pub fn set_target(&mut self, target: Vec2) {
        self.target = target;
    }

    /// Is the spring close enough to its target, and slow enough, to be considered at rest?
    ///
    /// The velocity threshold scales with the spring's own natural frequency
    /// (`sqrt(stiffness / mass)`) rather than using `SETTLE_EPSILON` directly: a stiffer spring
    /// covering the same positional epsilon necessarily passes through it at a proportionally
    /// higher velocity, so a fixed absolute velocity threshold made stiffer springs take
    /// *longer* to register as settled, not shorter, despite converging positionally faster.
    pub fn is_settled(&self) -> bool {
        let natural_frequency = (self.stiffness / self.mass.max(f32::EPSILON)).sqrt();
        let velocity_epsilon = Self::SETTLE_EPSILON * natural_frequency;
        (self.target - self.value).length_squared() < Self::SETTLE_EPSILON * Self::SETTLE_EPSILON
            && self.velocity.length_squared() < velocity_epsilon * velocity_epsilon
    }

    fn step(&mut self, dt: f32) {
        if self.mass <= 0.0 || dt <= 0.0 {
            return;
        }

        let displacement = self.value - self.target;
        let spring_force = -self.stiffness * displacement;
        let damping_force = -self.damping * self.velocity;
        let acceleration = (spring_force + damping_force) / self.mass;

        self.velocity += acceleration * dt;
        self.value += self.velocity * dt;

        if self.is_settled() {
            self.value = self.target;
            self.velocity = Vec2::ZERO;
        }
    }
}

pub(crate) fn update_springs(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Spring, Has<Animating>)>,
) {
    let dt = time.delta_secs();
    for (entity, mut spring, was_animating) in query.iter_mut() {
        spring.step(dt);

        let is_settled = spring.is_settled();
        if was_animating && is_settled {
            commands.entity(entity).remove::<Animating>();
        } else if !was_animating && !is_settled {
            commands.entity(entity).insert(Animating);
        }
    }
}

/// Snapshot of the last-diffed [`Spring::is_settled`] value for a widget entity.
///
/// Mirrors [`PreviousTransitionPlaying`]/[`PreviousAnimationTimelinePlaying`]: `Spring`
/// deliberately never opts into [`crate::diffable_prop::DiffableProp`] since its `value`/
/// `velocity` fields change every frame while integrating, which would make it compare as
/// "changed" every frame instead of only when its settled-state actually flips.
#[derive(Component, Default)]
pub(crate) struct PreviousSpringSettled(bool);

/// Returns `true` (and updates the snapshot) if `entity` has a `Spring` component whose
/// settled-state changed since the last check. A widget with no `Spring` at all is simply
/// never affected by this check.
pub(crate) fn diff_spring(world: &mut World, entity: Entity) -> bool {
    let Some(spring) = world.get::<Spring>(entity) else {
        return false;
    };
    let is_settled = spring.is_settled();

    let changed = match world.get::<PreviousSpringSettled>(entity) {
        Some(previous) => previous.0 != is_settled,
        None => true,
    };

    if changed {
        world
            .entity_mut(entity)
            .insert(PreviousSpringSettled(is_settled));
    }

    changed
}

/// Lerps a widget's `WoodpeckerStyle` between two endpoints using a sibling [`Spring`]'s
/// `value.x` (clamped `0.0..=1.0`) as the interpolation factor, the same role [`Transition`]'s
/// wall-clock elapsed time plays for its own `style_a`/`style_b`. Driven every frame by
/// [`update_spring_styles`], independent of whether the widget's own `render` re-runs -- so
/// retargeting the driving `Spring` mid-flight (see [`Spring::set_target`]) continues smoothly
/// from wherever `value` currently is, rather than snapping to an endpoint first the way
/// `Transition::start_reverse` does.
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq)]
pub struct SpringStyle {
    /// The style when the driving `Spring`'s `value.x` is `0.0`.
    pub style_a: WoodpeckerStyle,
    /// The style when the driving `Spring`'s `value.x` is `1.0`.
    pub style_b: WoodpeckerStyle,
}

pub(crate) fn update_spring_styles(
    mut query: Query<(&Spring, &SpringStyle, &mut WoodpeckerStyle)>,
) {
    for (spring, spring_style, mut styles) in query.iter_mut() {
        *styles = spring_style
            .style_a
            .lerp(&spring_style.style_b, spring.value.x.clamp(0.0, 1.0));
    }
}

/// A stateless helper for orchestrating staggered starts across a group of animations.
///
/// Both [`Transition`] and [`AnimationTimeline`] measure elapsed time from their `start` field
/// via `Instant::elapsed`, which saturates to zero rather than panicking when `start` is in the
/// future. Giving each item in a group a `start` produced by [`Self::staggered_start`] therefore
/// delays it by `index * stagger_delay_ms`, with no other changes needed to either animation
/// primitive.
pub struct AnimationGroup;

impl AnimationGroup {
    /// Returns the `Instant` an item at `index` in a staggered group should start at, delayed by
    /// `index * stagger_delay_ms` milliseconds from now.
    pub fn staggered_start(index: usize, stagger_delay_ms: f32) -> Instant {
        let delay_ms = (index as f32 * stagger_delay_ms).max(0.0);
        Instant::now() + Duration::from_secs_f32(delay_ms / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style_with_width(width: f32) -> WoodpeckerStyle {
        WoodpeckerStyle {
            width: Units::Pixels(width),
            ..Default::default()
        }
    }

    fn width_track() -> AnimationTrack {
        AnimationTrack {
            easing: TransitionEasing::Linear,
            delay: 0.0,
            duration: 200.0,
            style_a: style_with_width(0.0),
            style_b: style_with_width(100.0),
        }
    }

    #[test]
    fn track_progress_is_zero_before_its_delay_elapses() {
        let track = AnimationTrack {
            delay: 100.0,
            ..width_track()
        };
        assert_eq!(track.progress(0.0), 0.0);
        assert_eq!(track.progress(50.0), 0.0);
    }

    #[test]
    fn track_progress_is_the_eased_lerp_mid_window() {
        let track = width_track();
        assert_eq!(track.progress(100.0), 0.5);
    }

    #[test]
    fn track_progress_is_one_after_its_duration_ends() {
        let track = width_track();
        assert_eq!(track.progress(500.0), 1.0);
    }

    #[test]
    fn merge_into_only_overwrites_fields_the_track_actually_animates() {
        let track = width_track();
        let base = WoodpeckerStyle {
            opacity: 0.5,
            ..Default::default()
        };

        let merged = track.merge_into(base, 100.0);

        assert_eq!(merged.opacity, 0.5);
        assert_eq!(merged.width, Units::Pixels(50.0));
    }

    #[test]
    fn timeline_settles_at_the_final_style_after_its_total_duration() {
        let mut timeline = AnimationTimeline {
            playing: true,
            looping: false,
            tracks: vec![AnimationTrack {
                duration: 10.0,
                ..width_track()
            }],
            start: Instant::now() - Duration::from_secs_f32(1.0),
        };

        let result = timeline.update(WoodpeckerStyle::default());

        assert_eq!(result.width, Units::Pixels(100.0));
        assert!(!timeline.is_playing());
    }

    #[test]
    fn looping_timeline_restarts_instead_of_settling() {
        let mut timeline = AnimationTimeline {
            playing: true,
            looping: true,
            tracks: vec![AnimationTrack {
                duration: 10.0,
                ..width_track()
            }],
            start: Instant::now() - Duration::from_secs_f32(1.0),
        };

        timeline.update(WoodpeckerStyle::default());

        assert!(timeline.is_playing());
        assert!(timeline.start.elapsed().as_secs_f32() < 1.0);
    }

    #[test]
    fn spring_converges_towards_its_target_within_epsilon() {
        let mut spring = Spring {
            target: Vec2::new(100.0, 0.0),
            ..Default::default()
        };

        for _ in 0..1000 {
            spring.step(1.0 / 60.0);
        }

        assert!(spring.is_settled());
        assert!((spring.value - spring.target).length() < 0.1);
    }

    #[test]
    fn spring_snap_to_is_immediately_settled() {
        let mut spring = Spring::default();
        spring.snap_to(Vec2::new(50.0, 50.0));
        assert!(spring.is_settled());
    }

    #[test]
    fn staggered_start_delays_later_indices_further_into_the_future() {
        let first = AnimationGroup::staggered_start(0, 100.0);
        let second = AnimationGroup::staggered_start(1, 100.0);
        assert!(second > first);
    }
}
