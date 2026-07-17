use crate::{
    prelude::*,
    widgets::badge::{BadgeStyles, BadgeVariant},
};
use bevy::{platform::time::Instant, prelude::*};

/// How long a card's enter/exit slide-and-fade takes, in milliseconds. `EXIT_GRACE_SECS`
/// (how long `tick_toasts` keeps a `closing` entry in the queue before actually removing it)
/// is derived from this so removal always lines up with the fade finishing, not before.
const CARD_ANIMATION_MS: f32 = 200.0;
const EXIT_GRACE_SECS: f32 = CARD_ANIMATION_MS / 1000.0;

/// [`ToastViewport`]'s themed card chrome. The left accent border color is deliberately not
/// here -- it varies per [`ToastEntry::variant`], so it's read from [`BadgeStyles`] (already
/// reused for `ToastEntry::variant`'s type) instead of duplicating a second variant→color map.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ToastStyles {
    /// Card background color.
    pub background_color: Color,
    /// Message text color.
    pub text_color: Color,
    /// Dismiss glyph color -- a de-emphasized variant of `text_color`.
    pub text_muted_color: Color,
    /// Card corner radius.
    pub corner_radius: f32,
    /// Message/dismiss glyph font size.
    pub font_size: f32,
    /// Card drop shadow.
    pub box_shadow: WidgetBoxShadow,
}

impl Default for ToastStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ToastStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_color: theme.background,
            text_color: theme.text,
            text_muted_color: theme.text.with_alpha(0.6),
            corner_radius: theme.control_radius,
            font_size: theme.font_size,
            box_shadow: theme.elevation.md,
        }
    }
}

/// One queued toast notification.
#[derive(Reflect, Clone, PartialEq, Debug)]
pub struct ToastEntry {
    id: u64,
    /// The message shown in the toast.
    pub message: String,
    /// The semantic color variant (reuses [`BadgeVariant`] rather than inventing a second,
    /// near-identical enum).
    pub variant: BadgeVariant,
    seconds_remaining: f32,
    /// Set once (by [`ToastQueue::dismiss`] or its own `seconds_remaining` running out) and
    /// never unset -- `ToastViewport::render` keeps rendering (and fading out) an entry for
    /// as long as it stays in the queue after this flips, rather than the card just vanishing
    /// the instant it's no longer wanted. See `tick_toasts` for how it's actually removed.
    closing: bool,
    /// Counts down from [`EXIT_GRACE_SECS`] once `closing` is set -- `tick_toasts` removes the
    /// entry from the queue when this reaches zero, timed to line up with the exit fade
    /// finishing on screen.
    closing_seconds_remaining: f32,
}

impl ToastEntry {
    /// A stable identifier for this toast, usable with [`ToastQueue::dismiss`].
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// The queue of currently-visible toast notifications. Insert with [`ToastQueue::push`] from
/// anywhere (any system with `ResMut<ToastQueue>`); render it by placing one
/// [`ToastViewport`] widget near the root of your app. Entries auto-dismiss after their
/// duration via [`tick_toasts`] (registered automatically by the plugin), or can be dismissed
/// early with [`ToastQueue::dismiss`].
#[derive(Resource, Reflect, Clone, PartialEq, Default)]
pub struct ToastQueue {
    entries: Vec<ToastEntry>,
    next_id: u64,
}

impl ToastQueue {
    /// Queues a new toast, visible for `duration_secs` seconds (auto-dismissed after).
    pub fn push_for(
        &mut self,
        message: impl Into<String>,
        variant: BadgeVariant,
        duration_secs: f32,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(ToastEntry {
            id,
            message: message.into(),
            variant,
            seconds_remaining: duration_secs,
            closing: false,
            closing_seconds_remaining: 0.0,
        });
        id
    }

    /// Queues a new toast with the default 4 second duration.
    pub fn push(&mut self, message: impl Into<String>, variant: BadgeVariant) -> u64 {
        self.push_for(message, variant, 4.0)
    }

    /// Starts dismissing a toast, e.g. from its own click/dismiss-glyph handler -- marks it
    /// `closing` rather than removing it immediately, so `ToastViewport` has time to play its
    /// exit fade first (see [`ToastEntry::closing`]). A no-op if it's already closing, so
    /// clicking an already-dismissing card doesn't restart its exit animation.
    pub fn dismiss(&mut self, id: u64) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            if !entry.closing {
                entry.closing = true;
                entry.closing_seconds_remaining = EXIT_GRACE_SECS;
            }
        }
    }
}

/// Ticks down every queued toast's remaining time, starts closing any that have expired, and
/// removes any that have finished their closing grace period. Added to `Update` automatically
/// by `WoodpeckerUIWidgetPlugin`.
///
/// Note: this mutates `ToastQueue` every frame a toast is visible, so `ToastViewport`
/// re-renders every frame while any toast is up -- fine for a handful of short-lived toasts,
/// not a pattern to copy for large/long-lived watched lists.
pub(crate) fn tick_toasts(time: Res<Time>, mut queue: ResMut<ToastQueue>) {
    if queue.entries.is_empty() {
        return;
    }
    let dt = time.delta_secs();
    for entry in queue.entries.iter_mut() {
        if entry.closing {
            entry.closing_seconds_remaining -= dt;
        } else {
            entry.seconds_remaining -= dt;
            if entry.seconds_remaining <= 0.0 {
                entry.closing = true;
                entry.closing_seconds_remaining = EXIT_GRACE_SECS;
            }
        }
    }
    queue
        .entries
        .retain(|e| !(e.closing && e.closing_seconds_remaining <= 0.0));
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        position: WidgetPosition::Fixed,
        flex_direction: WidgetFlexDirection::Column,
        justify_content: Some(WidgetAlignContent::End),
        align_items: Some(WidgetAlignItems::End),
        padding: Edge::all(20.0),
        gap: (0.0.into(), 10.0.into()),
        z_index: Some(WidgetZ::Global(StackingTier::Toast as u32)),
        ..Default::default()
    }
}

/// One card's enter/exit animation timing, tracked in [`ToastViewportState`] across
/// `ToastViewport`'s renders -- which happen every frame while any toast is visible, see
/// `tick_toasts`'s doc comment -- so a card's fade plays exactly once from a fixed start time
/// instead of restarting from `Instant::now()` every render.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
struct ToastCardTiming {
    id: u64,
    started_at: Instant,
    /// `false` while entering (or once settled/visible), `true` once its exit fade has begun.
    exiting: bool,
}

/// Per-card animation timing for every toast `ToastViewport` currently knows about. A plain
/// `Vec`, not a `HashMap`: a toast stack is always small (a handful of entries at most).
#[derive(Component, Debug, Clone, PartialEq, Reflect, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ToastViewportState {
    timings: Vec<ToastCardTiming>,
}

/// Renders the current [`ToastQueue`] as a stack of dismissible cards, pinned to the bottom-
/// right of the screen. Place exactly one of these near your app's root, alongside a
/// [`crate::portal::OverlayRoot`]-tagged widget (see [`OverlayRootWidget`](super::OverlayRootWidget)) --
/// its actual visible content is `.portal_to()`-ed there, since a widget can't portal itself.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WatchedResource<ToastQueue>, ToastStyles, BadgeStyles)]
pub struct ToastViewport;

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    // `Option`, not a hard `Res`: a sibling of `OverlayRootWidget` in the same initial tree can
    // in principle render before `sync_overlay_root` has observed it -- see that system's doc
    // comment. Skipping a frame beats panicking on a legitimate startup-order race.
    overlay_root: Option<Res<OverlayRoot>>,
    icon_font: Res<IconFont>,
    mut query: Query<(
        &WatchedResource<ToastQueue>,
        &ToastStyles,
        &BadgeStyles,
        &mut WidgetChildren,
    )>,
    mut state_query: Query<&mut ToastViewportState>,
) {
    let Ok((watched, toast_styles, badge_styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };
    let entries = watched.0.entries.clone();

    *children = WidgetChildren::default();
    let current_widget = *current_widget;

    let Some(overlay_root) = overlay_root else {
        children.apply(current_widget.as_parent());
        return;
    };

    if entries.is_empty() {
        children.apply(current_widget.as_parent());
        return;
    }

    let state_entity = hooks.use_state(&mut commands, current_widget, ToastViewportState::default());
    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    let mut card_children = WidgetChildren::default();
    for entry in &entries {
        let (accent, _) = badge_styles.colors(entry.variant);
        let id = entry.id;

        let steady = WoodpeckerStyle {
            background_color: toast_styles.background_color,
            border_color: accent,
            border: Edge::all(0.0).left(3.0),
            border_radius: Corner::all(toast_styles.corner_radius),
            padding: Edge::all(12.0),
            min_width: 240.0.into(),
            max_width: 360.0.into(),
            align_items: Some(WidgetAlignItems::Center),
            box_shadow: Some(toast_styles.box_shadow),
            left: Units::Pixels(0.0),
            opacity: 1.0,
            ..Default::default()
        };
        const SLIDE_OFFSET: f32 = 24.0;
        let mut offscreen = steady;
        offscreen.left = Units::Pixels(SLIDE_OFFSET);
        offscreen.opacity = 0.0;

        match state.timings.iter().position(|t| t.id == id) {
            None => state.timings.push(ToastCardTiming {
                id,
                started_at: Instant::now(),
                exiting: false,
            }),
            Some(i) if entry.closing && !state.timings[i].exiting => {
                state.timings[i].exiting = true;
                state.timings[i].started_at = Instant::now();
            }
            Some(_) => {}
        }
        let timing = state
            .timings
            .iter()
            .find(|t| t.id == id)
            .copied()
            .expect("just inserted or already present above");

        let (style_a, style_b) = if timing.exiting {
            (steady, offscreen)
        } else {
            (offscreen, steady)
        };
        let elapsed_ms = timing.started_at.elapsed().as_secs_f32() * 1000.0;
        let card_style = Transition {
            easing: TransitionEasing::CubicOut,
            timeout: CARD_ANIMATION_MS,
            looping: false,
            playing: elapsed_ms < CARD_ANIMATION_MS,
            reversing: false,
            start: timing.started_at,
            style_a,
            style_b,
        }
        .update();

        card_children.add::<Element>((
            Element,
            card_style,
            WidgetRender::Quad,
            Pickable::default(),
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: toast_styles.font_size,
                        color: toast_styles.text_color,
                        flex_grow: 1.0,
                        text_wrap: TextWrap::WordOrGlyph,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: entry.message.clone(),
                    },
                ))
                .with_key("message")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: toast_styles.font_size,
                        color: toast_styles.text_muted_color,
                        margin: Edge::all(0.0).left(10.0),
                        font: Some(icon_font.0.id()),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: icons::X.into(),
                    },
                ))
                .with_key("dismiss"),
        ));
        card_children.observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut queue: ResMut<ToastQueue>| {
                queue.dismiss(id);
            },
        );
        card_children.hover_cursor(current_widget, SystemCursorIcon::Pointer);
        card_children.add_key(id.to_string());
    }
    state
        .timings
        .retain(|t| entries.iter().any(|e| e.id == t.id));

    // `ToastViewport` can't portal itself (only whoever *declares* a child can mark it
    // `.portal()`-ed), so it wraps its whole visible output in this one child instead and
    // portals *that* -- see the matching comment in `modal.rs`'s `render`.
    children.add::<Element>((Element, default_style(), card_children));
    children.add_key("portal_wrapper");
    children.portal_to(overlay_root.0);

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = ToastStyles::from_theme(&theme);
        assert_eq!(styles.background_color, theme.background);
        assert_eq!(styles.text_color, theme.text);
        assert_eq!(styles.text_muted_color, theme.text.with_alpha(0.6));
        assert_eq!(styles.corner_radius, theme.control_radius);
        assert_eq!(styles.font_size, theme.font_size);
        assert_eq!(styles.box_shadow, theme.elevation.md);

        let dark = ToastStyles::from_theme(&Theme::dark());
        let light = ToastStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.background_color, light.background_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn dismiss_marks_the_entry_closing_instead_of_removing_it_immediately() {
        let mut queue = ToastQueue::default();
        let id = queue.push("hello", BadgeVariant::Info);

        queue.dismiss(id);

        assert_eq!(
            queue.entries.len(),
            1,
            "dismiss should leave the entry in the queue so its exit fade can play"
        );
        assert!(queue.entries[0].closing);
        assert_eq!(queue.entries[0].closing_seconds_remaining, EXIT_GRACE_SECS);
    }

    #[test]
    fn dismissing_an_already_closing_entry_does_not_restart_its_exit_timer() {
        let mut queue = ToastQueue::default();
        let id = queue.push("hello", BadgeVariant::Info);
        queue.dismiss(id);
        queue.entries[0].closing_seconds_remaining = 0.01;

        queue.dismiss(id);

        assert_eq!(
            queue.entries[0].closing_seconds_remaining, 0.01,
            "a second dismiss on an already-closing entry must not reset its exit fade"
        );
    }

    #[test]
    fn dismissing_an_unknown_id_does_not_panic() {
        let mut queue = ToastQueue::default();
        queue.push("hello", BadgeVariant::Info);

        queue.dismiss(999);

        assert_eq!(queue.entries.len(), 1);
    }
}
