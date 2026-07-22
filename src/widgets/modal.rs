use bevy::prelude::*;

use crate::prelude::*;

#[derive(Component, Reflect, PartialEq, Clone, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ModalState {
    previous_visibility: bool,
}

/// Styles for the modal
#[derive(Component, Reflect, PartialEq, Clone, Debug)]
pub struct ModalStyles {
    /// Window Styles
    pub window: WoodpeckerStyle,
    /// Titlebar Styles
    pub title_bar: WoodpeckerStyle,
}

impl Default for ModalStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ModalStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            window: WoodpeckerStyle {
                background_color: theme.background,
                border_color: theme.primary,
                border: Edge::all(1.0),
                border_radius: Corner::all(theme.panel_radius),
                flex_direction: WidgetFlexDirection::Column,
                box_shadow: Some(theme.elevation.xl),
                ..Default::default()
            },
            title_bar: WoodpeckerStyle {
                height: Units::Pixels(40.0),
                width: Units::Percentage(100.0),
                padding: Edge::new(0.0, 0.0, 0.0, 12.0),
                align_items: Some(WidgetAlignItems::Center),
                background_color: theme.dark_background,
                border_radius: Corner::all(0.0)
                    .top_left(theme.panel_radius)
                    .top_right(theme.panel_radius),
                border_color: theme.primary,
                border: Edge::all(0.0).bottom(1.0),
                ..Default::default()
            },
        }
    }
}

/// Replace title children for modals and windows.
#[derive(Component, Default, PartialEq, Clone)]
pub struct TitleChildren(pub WidgetChildren);

/// A widget that displays a modal
#[derive(Component, Widget, Reflect, PartialEq, Clone, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, PassedChildren, WidgetChildren, Transition = get_transition(), ModalStyles)]
pub struct Modal {
    /// The text to display in the modal's title bar
    pub title: String,
    /// A set of styles to apply to the children element wrapper.
    pub children_styles: WoodpeckerStyle,
    /// Is the modal open?
    pub visible: bool,
    /// Animation timeout in milliseconds.
    pub timeout: f32,
    /// The overlay background alpha value
    pub overlay_color: Color,
    /// State for animation play
    pub transition_play: bool,
    /// The min size of the modal,
    pub min_size: Vec2,
}

/// The wrapper's own style -- `Fixed`/centered/tiered, unaffected by the opacity fade (see
/// `get_transition`). Modal's own entity can't portal itself (only whoever *declares* a child
/// can mark it `.portal()`-ed), so this is applied to a synthetic child that portals to
/// `OverlayRoot` instead of to Modal's own `WoodpeckerStyle` -- see `render`'s `portal_wrapper`
/// construction.
fn get_styles() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        justify_content: Some(WidgetAlignContent::Center),
        align_items: Some(WidgetAlignItems::Center),
        position: WidgetPosition::Fixed,
        // Without an explicit z_index, a focused `WoodpeckerWindow` could render/pick above
        // this modal. See `StackingTier` for the full reserved-tier scale this crate's
        // built-in overlay widgets share.
        z_index: Some(WidgetZ::Global(StackingTier::Modal as u32)),
        ..Default::default()
    }
}

/// Only interpolates `opacity` -- kept on Modal's own (non-portaled) entity purely so
/// `diff_transition` can keep gating a re-render at the moment the animation starts/stops (see
/// `render`'s `should_render`). This is deliberately NOT what actually drives the fade seen on
/// screen -- see `render`'s `wrapper_transition` for that -- since Modal's own entity's
/// `WoodpeckerStyle` isn't rendered anywhere; giving it `get_styles()`'s `Fixed`/`Global` z_index
/// here would false-positive the dev-build portal lint on a non-portaled entity for no benefit.
fn get_transition() -> Transition {
    Transition {
        easing: TransitionEasing::Linear,
        looping: false,
        playing: false,
        style_a: WoodpeckerStyle {
            opacity: 0.0,
            ..Default::default()
        },
        style_b: WoodpeckerStyle {
            opacity: 1.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for Modal {
    fn default() -> Self {
        Self {
            title: Default::default(),
            children_styles: Default::default(),
            visible: false,
            timeout: 250.0,
            overlay_color: Srgba::new(0.0, 0.0, 0.0, 0.6).into(),
            transition_play: false,
            min_size: Vec2::new(400.0, 250.0),
        }
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    theme: Res<Theme>,
    // `Option`, not a hard `Res`: a sibling of `OverlayRootWidget` in the same initial tree can
    // in principle render before `sync_overlay_root` has observed it -- see that system's doc
    // comment. Skipping a frame beats panicking on a legitimate startup-order race.
    overlay_root: Option<Res<OverlayRoot>>,
    mut query: Query<(
        &Modal,
        &mut WidgetChildren,
        &PassedChildren,
        &ModalStyles,
        &mut Transition,
        Option<&TitleChildren>,
    )>,
    mut modal_state: Query<&mut ModalState>,
) {
    let Ok((
        modal,
        mut internal_children,
        passed_children,
        modal_styles,
        mut transition,
        title_children,
    )) = query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        ModalState {
            previous_visibility: modal.visible,
        },
    );

    let Ok(mut state) = modal_state.get_mut(state_entity) else {
        return;
    };

    *transition = Transition {
        reversing: !modal.visible,
        timeout: modal.timeout,
        ..*transition
    };

    if state.previous_visibility != modal.visible {
        if transition.reversing {
            transition.start_reverse()
        } else {
            transition.start();
        }
        state.previous_visibility = modal.visible;
    }

    // Always reset and re-`apply()`, even when not rendering anything, so a closed modal
    // declares zero children and the normal keyed reconciliation despawns its overlay/window
    // immediately.
    *internal_children = WidgetChildren::default();

    let should_render = (transition.is_playing() || modal.visible) && overlay_root.is_some();
    if should_render {
        let overlay_root = overlay_root.unwrap();
        let mut portal_children = WidgetChildren::default();
        portal_children
            // Overlay
            .add::<Element>((
                Element,
                WoodpeckerStyle {
                    background_color: modal.overlay_color,
                    width: Units::Percentage(100.0),
                    height: Units::Percentage(100.0),
                    position: WidgetPosition::Absolute,
                    ..Default::default()
                },
                Pickable::default(),
                WidgetRender::Quad,
            ));
        portal_children.add_key("overlay");
        portal_children
            .observe(*current_widget, move |mut trigger: On<Pointer<Over>>| {
                trigger.propagate(false);
            })
            .observe(*current_widget, move |mut trigger: On<Pointer<Out>>| {
                trigger.propagate(false);
            })
            .observe(*current_widget, move |mut trigger: On<Pointer<Click>>| {
                trigger.propagate(false);
            })
            // Window
            .add::<Element>((
                Element,
                WoodpeckerStyle {
                    min_width: modal.min_size.x.into(),
                    min_height: modal.min_size.y.into(),
                    ..modal_styles.window
                },
                // Confines Tab/Shift+Tab navigation to the modal's own content while it's on
                // screen (including mid-close-transition) -- see `tab_focus::tab_navigate`.
                FocusTrapRoot,
                WidgetChildren::default()
                    // Title Bar
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            ..modal_styles.title_bar
                        },
                        WidgetRender::Quad,
                        // Title text
                        if let Some(children) = title_children.as_ref() {
                            children.0.clone()
                        } else {
                            WidgetChildren::default().with_child::<Element>((
                                Element,
                                WoodpeckerStyle {
                                    font_size: theme.font_size,
                                    color: theme.text,
                                    text_wrap: TextWrap::None,
                                    ..Default::default()
                                },
                                WidgetRender::Text {
                                    content: modal.title.clone(),
                                },
                            ))
                        },
                    ))
                    .with_key("title_bar")
                    // Content
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            width: Units::Percentage(100.0),
                            height: Units::Percentage(100.0),
                            ..Default::default()
                        },
                        passed_children.0.clone(),
                    ))
                    .with_key("content"),
                WidgetRender::Quad,
            ));
        portal_children.add_key("window");

        // The modal's own entity can't portal itself (only whoever *declares* a child can
        // mark it `.portal()`-ed) -- so it wraps its whole visual output (overlay + window)
        // in this single child instead, and portals *that*.
        //
        // The wrapper gets its OWN real `Transition` component here (not a value copied from
        // `*transition` each render), sharing `*transition`'s timing fields (`start`,
        // `reversing`, `playing`, `timeout`) via `..*transition` so both tick in perfect
        // lockstep, but with `get_styles()`-based (`Fixed`/centered/tiered) style_a/b instead
        // of `*transition`'s own minimal opacity-only ones. This matters because `render` here
        // only runs occasionally while the fade is playing (`diff_transition` only forces a
        // render on the frame `is_playing()` *changes*, not continuously -- see its own doc
        // comment) -- once inserted, `transition::update_transitions` (a generic system that
        // runs unconditionally every frame, independent of whether *this* `render` runs) keeps
        // ticking the wrapper's `WoodpeckerStyle` on its own from here on, exactly like it
        // always did for Modal's own entity before this widget portaled its content. Reading
        // the fade back from `*transition`/`styles` here instead (recomputing a merged style
        // once per `render` call) is what caused the fade to freeze after only a couple of
        // frames: `render` doesn't run often enough on its own to drive it.
        let wrapper_transition = Transition {
            style_a: WoodpeckerStyle {
                opacity: 0.0,
                ..get_styles()
            },
            style_b: WoodpeckerStyle {
                opacity: 1.0,
                ..get_styles()
            },
            ..*transition
        };
        internal_children.add::<Element>((
            Element,
            get_styles(),
            wrapper_transition,
            WidgetRender::Layer,
            StackingContext,
            portal_children,
        ));
        internal_children.add_key("portal_wrapper");
        internal_children.portal_to(overlay_root.0);
    }

    internal_children.apply(current_widget.as_parent());
}
