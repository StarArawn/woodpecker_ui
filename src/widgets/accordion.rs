use crate::prelude::*;
use bevy::prelude::*;

/// [`Accordion`]'s two expand/collapse behaviors.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AccordionMode {
    /// Any number of [`AccordionItem`]s can be expanded at once (MUI's own default).
    #[default]
    Multiple,
    /// Expanding one item collapses whichever other item was previously expanded.
    Single,
}

/// Shared expand/collapse state for every [`AccordionItem`] under one [`Accordion`] --
/// modeled directly on `TreeView`'s own `expanded: Vec<String>` (see `src/widgets/tree_view.rs`),
/// with `Single` mode's "only one at a time" behavior layered on top in [`Self::toggle`].
#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct AccordionContext {
    mode: AccordionMode,
    expanded: Vec<String>,
}

impl AccordionContext {
    fn is_expanded(&self, key: &str) -> bool {
        self.expanded.iter().any(|k| k == key)
    }

    /// Flips `key`'s expanded state, returning the new value. In `Single` mode, expanding
    /// `key` also collapses every other currently-expanded key.
    fn toggle(&mut self, key: &str) -> bool {
        let now_expanded = !self.is_expanded(key);
        match self.mode {
            AccordionMode::Multiple => {
                if now_expanded {
                    self.expanded.push(key.to_string());
                } else {
                    self.expanded.retain(|k| k != key);
                }
            }
            AccordionMode::Single => {
                self.expanded = if now_expanded {
                    vec![key.to_string()]
                } else {
                    Vec::new()
                };
            }
        }
        now_expanded
    }
}

/// Fired when an [`AccordionItem`] is expanded or collapsed -- mirrors
/// [`crate::widgets::TreeNodeToggled`]'s shape.
#[derive(Reflect, Clone, PartialEq, Debug)]
pub struct AccordionChanged {
    /// The toggled item's key.
    pub key: String,
    /// Its new expanded state.
    pub expanded: bool,
}

/// A context provider grouping any number of [`AccordionItem`] children -- declare items as
/// its normal `WidgetChildren`, the same way [`crate::widgets::TabContextProvider`] groups
/// [`crate::widgets::TabButton`]s. Purely a state/layout wrapper; renders no visible surface
/// of its own (each item draws its own bordered panel).
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, PassedChildren)]
pub struct Accordion {
    /// `Multiple` (default) or `Single` -- see [`AccordionMode`].
    pub mode: AccordionMode,
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &Accordion,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        &PassedChildren,
    )>,
) {
    let Ok((accordion, mut styles, mut children, passed_children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    // Only used the first time this entity provides the context -- later renders reuse
    // whatever's already there, same "initial prop, then state takes over" pattern
    // `TreeView`/`Dropdown` use for their own seeded-then-owned state.
    hooks.use_context(
        &mut commands,
        *current_widget,
        AccordionContext {
            mode: accordion.mode,
            expanded: Vec::new(),
        },
    );

    styles.flex_direction = WidgetFlexDirection::Column;
    if styles.width == Units::Auto {
        styles.width = Units::Percentage(100.0);
    }

    *children = passed_children.0.clone();
    children.apply(current_widget.as_parent());
}

/// [`AccordionItem`]'s themed colors -- a separate sibling component so it live-resyncs on a
/// [`Theme`] swap.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct AccordionItemStyles {
    /// Panel background color.
    pub background_color: Color,
    /// Panel border color.
    pub border_color: Color,
    /// Panel corner radius.
    pub corner_radius: f32,
    /// Header label text color.
    pub label_color: Color,
    /// Disclosure glyph color.
    pub disclosure_color: Color,
    /// Header background while hovered.
    pub header_hover_background: Color,
}

impl Default for AccordionItemStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for AccordionItemStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_color: theme.background,
            border_color: theme.border,
            corner_radius: theme.control_radius,
            label_color: theme.text,
            disclosure_color: theme.text.with_alpha(0.6),
            header_hover_background: theme.background_light,
        }
    }
}

#[derive(Component, Default, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq, Clone)]
struct AccordionItemState {
    hovering: bool,
    /// The last expanded value this item actually animated to -- compared against the current
    /// value (derived from `AccordionContext` each render) to detect the toggle edge, the same
    /// shadow-field pattern `ModalState::previous_visibility` uses.
    previous_expanded: bool,
    /// Everything else `item_render` last actually used to build `children`, checked against
    /// the live values each render -- lets a render that doesn't affect this item's own output
    /// skip the full rebuild entirely instead of tearing down and respawning the body's text
    /// for no visible-effect reason. `None` until the first real render.
    ///
    /// This matters because *every* item under one [`Accordion`] shares one [`AccordionContext`]
    /// entity: toggling "shipping" bumps a component that "returns" and "warranty" both read
    /// too, so all three re-render, even though only one of them actually changed anything.
    /// Without this check, `item_render` would unconditionally rebuild all three items' body
    /// `Clip`+content subtrees (tearing down and respawning their text) on every single toggle,
    /// not just the toggled item's own.
    #[reflect(ignore)]
    rendered: Option<AccordionItemRenderInputs>,
}

/// See [`AccordionItemState::rendered`]. Deliberately excludes `natural_height`/the body
/// `Transition`'s own state -- those only matter on a genuine toggle, already covered by
/// `previous_expanded`.
#[derive(Clone, PartialEq)]
struct AccordionItemRenderInputs {
    label: String,
    styles: AccordionItemStyles,
    hovering: bool,
    is_expanded: bool,
}

/// Milliseconds for the body's expand/collapse slide.
const BODY_TRANSITION_TIMEOUT: f32 = 200.0;

/// The body wrapper's animated-height `Transition` -- `height` is the only field that
/// actually differs between `style_a`/`style_b`; every other field must still match exactly
/// on both, since a *settled* (non-playing) `Transition` resolves to a full copy of whichever
/// end it settled on, not a merge with whatever was there before (see `Transition::update`).
///
/// `reversing: true` here (not the struct's own default of `false`) is deliberate: a
/// `Transition` that has never been `.start()`-ed still gets ticked by `update_transitions`
/// every frame like any other, and its own "at rest" formula resolves to `style_a` only when
/// `reversing` is `true` (`style_b` otherwise) -- so a fresh, never-toggled item (this
/// function's own return value, used verbatim until the first real toggle) needs `reversing:
/// true` to settle at `style_a` (closed) instead of `style_b` (open). An actual toggle
/// immediately overwrites `reversing` via `.start()`/`.start_reverse()` regardless, so this
/// only ever matters for the untouched, freshly-constructed case.
fn body_transition(natural_height: f32) -> Transition {
    let constant = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    };
    Transition {
        easing: TransitionEasing::CubicInOut,
        timeout: BODY_TRANSITION_TIMEOUT,
        looping: false,
        playing: false,
        reversing: true,
        style_a: WoodpeckerStyle {
            height: 0.0.into(),
            ..constant
        },
        style_b: WoodpeckerStyle {
            height: natural_height.into(),
            ..constant
        },
        ..Default::default()
    }
}

/// One section within an [`Accordion`] -- a clickable header (label + disclosure glyph) that
/// toggles a conditionally-shown body. Fires [`Change<AccordionChanged>`] on toggle. Generalizes
/// what `examples/game_ui/quests_panel.rs`'s `QuestLog` visually resembles but doesn't actually
/// implement (that file has no expand/collapse state at all) -- this is a fresh design modeled
/// on `TreeView`'s expand/collapse state instead, not a promotion of that example.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(item_render)]
#[require(WoodpeckerStyle, PassedChildren, WidgetChildren, WidgetRender = WidgetRender::Quad, AccordionItemStyles)]
pub struct AccordionItem {
    /// A stable identifier, unique within the enclosing [`Accordion`].
    pub key: String,
    /// The header's label text.
    pub label: String,
}

fn item_render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    // Nested in one tuple param (rather than three top-level ones) to stay within the
    // `#[auto_update]` macro's 9-parameter ceiling (the `hotreload` feature's `HotFunction`
    // trait is only implemented up to 9 arguments) -- a tuple of `SystemParam`s is itself one
    // `SystemParam`, so this doesn't change what's actually queried, just how it's grouped.
    (theme, widget_mapper, icon_font): (Res<Theme>, Res<WidgetMapper>, Res<IconFont>),
    // Shared by both the natural-height lookup (on the "content" entity, which has a
    // `WidgetLayout` but no `Transition`) and the existing-transition lookup (on the "body"
    // entity, which has the reverse) -- `Option<&_>` on both sides lets one query type answer
    // either lookup depending on which entity it's called with, keeping this at the
    // `#[auto_update]` macro's 9-parameter ceiling.
    body_query: Query<(Option<&WidgetLayout>, Option<&Transition>)>,
    mut query: Query<(
        &AccordionItem,
        &AccordionItemStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        &PassedChildren,
    )>,
    context_query: Query<&AccordionContext>,
    mut state_query: Query<&mut AccordionItemState>,
) {
    let Ok((item, item_styles, mut styles, mut children, passed_children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let context_entity =
        hooks.use_context(&mut commands, *current_widget, AccordionContext::default());
    let Ok(context) = context_query.get(context_entity) else {
        return;
    };
    let is_expanded = context.is_expanded(&item.key);

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        AccordionItemState::default(),
    );
    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    let live_inputs = AccordionItemRenderInputs {
        label: item.label.clone(),
        styles: *item_styles,
        hovering: state.hovering,
        is_expanded,
    };
    if state.rendered.as_ref() == Some(&live_inputs) {
        // Nothing this item actually renders has changed -- most commonly this render was
        // triggered by a *sibling* item's toggle bumping the shared `AccordionContext` (every
        // item re-checks it, since `Single` mode may need to collapse any of them). Bail out
        // before touching `children` at all -- see `AccordionItemState::rendered`.
        return;
    }
    state.rendered = Some(live_inputs);

    let current_widget = *current_widget;

    // The body's own natural (unconstrained) height comes from its INNER "content" child --
    // the outer "body" wrapper's own height is what's *being animated*, so reading that back
    // would just report whatever height the animation last settled on, not the true target.
    // One-frame-lagged (this frame's freshly (re)declared children haven't gone through
    // layout yet) -- same trick `diff_window_wrapper_layout` uses for `WoodpeckerWindow`'s
    // portaled wrapper. Harmless here: a toggle is a discrete user action, not a per-frame
    // continuous read, so by the time a *real* toggle happens the content has long since had
    // a chance to settle into a real, queryable layout.
    let body_entity = widget_mapper.get_keyed_child::<Clip>(current_widget.as_parent(), "body");
    let content_entity = body_entity
        .and_then(|e| widget_mapper.get_keyed_child::<Element>(ParentWidget(e), "content"));
    let natural_height = content_entity
        .and_then(|e| body_query.get(e).ok())
        .and_then(|(layout, _)| layout)
        .map(|layout| layout.size.y)
        .unwrap_or(0.0);

    // `WidgetChildren::apply` unconditionally re-`insert`s every bundle component on a reused
    // entity, every time this renders (not just on first spawn) -- so the "body" Clip's
    // `Transition` can't be mutated in place via a separate query and left at that; whatever
    // value is (re)declared in its bundle below wins. On an actual toggle this computes a
    // freshly-started transition; otherwise it clones whatever's already there completely
    // unchanged, so re-declaring it is a no-op that doesn't disturb `update_transitions`'ing
    // ticking it independently every frame (e.g. a hover-driven re-render mid-animation must
    // not reset the animation's start time).
    //
    // `body_baseline` is the explicit `WoodpeckerStyle` given alongside the `Transition` in
    // the bundle below (see its own comment) -- computed here, next to whichever branch
    // determined *why* the value is what it is, since the two branches need genuinely
    // different formulas: freshly starting a transition means "current progress is ~0" (close
    // to `style_a` normally, or `style_b` when reversing -- `start_reverse` begins playback
    // *at* `style_b`, see `Transition::update`), whereas preserving an unchanged transition
    // (the common case, e.g. a hover-driven re-render with no toggle) means "whatever it last
    // settled at" (the opposite formula). Using the *settled* formula for a *freshly started*
    // transition -- an earlier version of this code did exactly that -- shows the wrong
    // extreme for one frame before `update_transitions` corrects it, a visible flash on every
    // single toggle (`update_transitions` and the runner are registered as separate
    // `add_systems` calls with no ordering constraint between them, so that one wrong frame is
    // genuinely displayed, not just theoretical).
    let toggled = state.previous_expanded != is_expanded;
    let (body_transition_value, body_baseline) = if toggled {
        let mut transition = body_transition(natural_height);
        if is_expanded {
            transition.start();
        } else {
            transition.start_reverse();
        }
        let baseline = if transition.reversing {
            transition.style_b
        } else {
            transition.style_a
        };
        (transition, baseline)
    } else if let Some(existing) = body_entity
        .and_then(|e| body_query.get(e).ok())
        .and_then(|(_, transition)| transition)
    {
        let baseline = if existing.reversing {
            existing.style_a
        } else {
            existing.style_b
        };
        (*existing, baseline)
    } else {
        let transition = body_transition(natural_height);
        let baseline = transition.style_a;
        (transition, baseline)
    };
    if toggled {
        state.previous_expanded = is_expanded;
    }

    styles.width = Units::Percentage(100.0);
    styles.flex_direction = WidgetFlexDirection::Column;
    styles.background_color = item_styles.background_color;
    styles.border_color = item_styles.border_color;
    styles.border = Edge::all(1.0);
    styles.border_radius = Corner::all(item_styles.corner_radius);
    styles.margin = Edge::all(0.0).bottom(theme.spacing.sm);

    let key = item.key.clone();
    let glyph = if is_expanded {
        icons::CARET_DOWN
    } else {
        icons::CARET_RIGHT
    };

    *children = WidgetChildren::default();
    children
        .add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: theme.control_height.into(),
                padding: Edge::all(0.0)
                    .left(theme.spacing.md)
                    .right(theme.spacing.md),
                align_items: Some(WidgetAlignItems::Center),
                justify_content: Some(WidgetAlignContent::SpaceBetween),
                background_color: if state.hovering {
                    item_styles.header_hover_background
                } else {
                    Color::NONE
                },
                ..Default::default()
            },
            WidgetRender::Quad,
            Pickable::default(),
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: theme.typography.body,
                        color: item_styles.label_color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: item.label.clone(),
                    },
                ))
                .with_key("label")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: theme.typography.body,
                        color: item_styles.disclosure_color,
                        font: Some(icon_font.0.id()),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: glyph.into(),
                    },
                ))
                .with_key("disclosure"),
        ))
        .self_hover_state(
            current_widget,
            state_entity,
            SystemCursorIcon::Pointer,
            |state: &mut AccordionItemState, hovering| state.hovering = hovering,
        )
        .observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut context_query: Query<&mut AccordionContext>| {
                let Ok(mut context) = context_query.get_mut(context_entity) else {
                    return;
                };
                let expanded = context.toggle(&key);
                commands.trigger(Change {
                    target: current_widget.entity(),
                    data: AccordionChanged {
                        key: key.clone(),
                        expanded,
                    },
                });
            },
        );
    children.add_key("header");

    // Always declared, regardless of `is_expanded` -- both so the collapse animation has
    // something to play (removing it outright the instant `is_expanded` flips would skip the
    // animation entirely) and so "content"'s own `WidgetLayout` stays available to measure
    // (see `natural_height` above). `Clip` masks the overflow while `height` is mid-animation
    // between 0 and `content`'s natural height.
    //
    // `body_baseline` (computed above, alongside `body_transition_value`) is an explicit
    // `WoodpeckerStyle` given alongside the `Transition` itself -- same belt-and-suspenders
    // reasoning as `Drawer`'s sliding panel: `update_transitions` normally keeps this current
    // from here on, but a freshly-spawned entity has no `WoodpeckerStyle` of its own to
    // interpolate from until its first tick.
    children.add::<Clip>((
        Clip,
        body_baseline,
        body_transition_value,
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    flex_direction: WidgetFlexDirection::Column,
                    padding: Edge::all(theme.spacing.md).top(0.0),
                    ..Default::default()
                },
                passed_children.0.clone(),
            ))
            .with_key("content"),
    ));
    children.add_key("body");

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_mode_allows_several_items_expanded_at_once() {
        let mut ctx = AccordionContext::default();
        assert!(ctx.toggle("a"));
        assert!(ctx.toggle("b"));
        assert!(ctx.is_expanded("a"));
        assert!(ctx.is_expanded("b"));
    }

    #[test]
    fn single_mode_collapses_the_previous_item_when_a_new_one_expands() {
        let mut ctx = AccordionContext {
            mode: AccordionMode::Single,
            expanded: Vec::new(),
        };
        assert!(ctx.toggle("a"));
        assert!(ctx.is_expanded("a"));

        assert!(ctx.toggle("b"));
        assert!(ctx.is_expanded("b"));
        assert!(
            !ctx.is_expanded("a"),
            "expanding b must collapse a in Single mode"
        );
    }

    #[test]
    fn toggle_collapses_an_already_expanded_item() {
        let mut ctx = AccordionContext::default();
        ctx.toggle("a");
        assert!(ctx.is_expanded("a"));
        assert!(!ctx.toggle("a"));
        assert!(!ctx.is_expanded("a"));
    }

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = AccordionItemStyles::from_theme(&theme);
        assert_eq!(styles.background_color, theme.background);
        assert_eq!(styles.label_color, theme.text);

        let dark = AccordionItemStyles::from_theme(&Theme::dark());
        let light = AccordionItemStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.background_color, light.background_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn a_never_started_body_transition_settles_closed_not_open() {
        // `Transition::update`'s "at rest" formula is `if reversing { style_a } else {
        // style_b }` -- a freshly constructed, never-`.start()`-ed transition must have
        // `reversing: true` so it settles at `style_a` (height 0, closed), not `style_b`
        // (height `natural_height`, open). Getting this backwards means every accordion item
        // renders fully expanded before its first real toggle.
        let mut transition = body_transition(123.0);
        assert!(!transition.is_playing());
        let settled = transition.update();
        assert_eq!(settled.height, Units::Pixels(0.0));
    }

    #[test]
    fn starting_the_body_transition_animates_toward_the_natural_height() {
        let mut transition = body_transition(123.0);
        transition.start();
        assert!(transition.is_playing());
        // Immediately after starting, progress is ~0 -- the interpolated height should be
        // much closer to 0 (closed) than to the 123px target.
        let just_started = transition.update();
        let Units::Pixels(height) = just_started.height else {
            panic!("expected a pixel height");
        };
        assert!(
            height < 10.0,
            "expected height near 0 right after start(), got {height}"
        );
    }
}
