use std::sync::Arc;

use crate::{layout::system::WidgetLayout, picking_backend::MouseWheelScroll, prelude::*};
use bevy::prelude::*;

/// A single-column virtualized list of arbitrary per-item content -- inventories, chat logs,
/// asset browsers, anything with many uniform-height rows where each row's *content* varies
/// but its *height* doesn't. For plain-text rows, prefer `Table::virtualized` instead; this
/// widget exists for the case where each row needs custom widgets (an icon, a button, a
/// nested layout).
///
/// Deliberately not composed from `ScrollContextProvider`/`ScrollBox`: a context registered by
/// a *child* can't be read back in the *same* render pass that needs it to decide what to
/// spawn (context lookup only walks upward), so composing those widgets here would always lag
/// a frame behind the real scroll position. Instead, this widget registers its own
/// `ScrollContext` on itself (via `hooks.use_context`, read and written directly in the same
/// render), which is exactly what lets a real `ScrollBar` child find and drive it -- `ScrollBar`
/// looks up a `ScrollContext` by walking *its own* ancestors, and this widget's entity is
/// that ancestor.
///
/// v1 scope: no PageUp/PageDown/Home/End keyboard paging -- flagged as a known trim, not
/// silently dropped, since adding it later is additive (it would just drive the same
/// `ScrollContext` the wheel handler and `ScrollBar` already share).
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq, Clone)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = default_style(),
    WidgetChildren,
    WatchLayout,
    Pickable,
    VirtualListScrollState
)]
pub struct VirtualList {
    /// Total number of items in the list (not how many are currently mounted).
    pub item_count: usize,
    /// The fixed height of a single item, in logical pixels.
    pub item_extent: f32,
    /// Builds the content for the item at a given index, called only for items actually in
    /// (or just outside) the visible window.
    #[reflect(ignore)]
    pub item_content: VirtualListItemContent,
}

/// Wraps a `VirtualList`'s per-item content builder. A plain `Arc<dyn Fn(usize) ->
/// WidgetChildren + Send + Sync>` field can't derive `PartialEq` (function objects have no
/// meaningful equality), so this newtype supplies one that always reports equal -- the
/// closure is set once at construction and never itself a reason to re-render (changing
/// `item_count`/`item_extent`, or the data the closure closes over via `Res`/`Query` inside
/// its own render, is what actually drives new content).
#[derive(Clone)]
pub struct VirtualListItemContent(Arc<dyn Fn(usize) -> WidgetChildren + Send + Sync>);

impl VirtualListItemContent {
    /// Wraps a per-item content builder for use as [`VirtualList::item_content`].
    pub fn new(f: impl Fn(usize) -> WidgetChildren + Send + Sync + 'static) -> Self {
        Self(Arc::new(f))
    }
}

impl PartialEq for VirtualListItemContent {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Default for VirtualListItemContent {
    fn default() -> Self {
        Self(Arc::new(|_| WidgetChildren::default()))
    }
}

/// Extra items kept mounted just outside the viewport on either side, so a fast scroll
/// doesn't show a blank flash before the next frame's items spawn in. This is a *floor*, not
/// the actual overscan used every frame -- see `render`'s velocity-scaled overscan, which
/// widens this whenever a single frame's scroll delta would otherwise outrun it.
const VIRTUAL_LIST_OVERSCAN: usize = 4;
/// Upper bound on the velocity-scaled overscan (see `render`) -- caps the one-time cost of an
/// extreme single-frame jump (e.g. dragging the scrollbar thumb straight to the bottom) at
/// mounting this many items, rather than however many thousands a raw delta might imply.
const VIRTUAL_LIST_MAX_OVERSCAN: usize = 64;
/// Logical pixels scrolled per wheel "notch" -- matches `ScrollBox`'s own default.
const SCROLL_LINE: f32 = 64.0;
/// The scrollbar's own thickness, in logical pixels -- matches `ScrollBox`'s own default.
const SCROLLBAR_THICKNESS: f32 = 10.0;

/// Tracks the previous frame's scroll offset purely to size this frame's overscan -- see
/// `render`'s velocity-scaled overscan doc comment. Not meant to be read by anything else.
/// Deliberately not `DiffableProp` -- it's written unconditionally every render, which would
/// make it permanently "changed" and defeat the point of the generic diff system rather than
/// ever meaningfully gate a render.
#[derive(Component, Default, Clone, PartialEq)]
struct VirtualListScrollState {
    prev_scroll_offset: f32,
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        ..Default::default()
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    layout_query: Query<&WidgetLayout>,
    mut query: Query<(
        &VirtualList,
        &mut WidgetChildren,
        &mut VirtualListScrollState,
    )>,
    mut context_query: Query<&mut ScrollContext>,
) {
    let Ok((list, mut children, mut scroll_state)) = query.get_mut(**current_widget) else {
        return;
    };

    // `use_own_context`, deliberately not the ordinary ancestor-walking `use_context`: a
    // `VirtualList` nested inside an unrelated page's own `ScrollContextProvider`/`ScrollBox`
    // (an ordinary, expected thing to do -- e.g. one virtualized panel inside an otherwise
    // normally-scrolling settings page) would otherwise silently adopt *that* page-level
    // `ScrollContext` instead of registering its own, since `use_context` just returns the
    // nearest `ScrollContext` it finds walking up, with no way to tell "the list's own private
    // scroll bookkeeping" apart from "the enclosing page's". The two then fight over the same
    // entity every render -- each write overwriting the other's `content_height`/
    // `scrollbox_height`/etc with values meant for a completely different scroll region -- which
    // cascades into a permanent, never-settling resize loop across the whole page (confirmed
    // empirically: every card on the page kept oscillating in width, forever, until this fix).
    let context_entity =
        hooks.use_own_context(&mut commands, *current_widget, ScrollContext::default());
    let Ok(mut context) = context_query.get_mut(context_entity) else {
        return;
    };

    // Own viewport size, read from our own already-computed layout -- zero on the very first
    // render (before layout has run once), which yields an empty window that self-corrects
    // next frame via `WatchLayout` once a real size lands.
    let own_layout = layout_query.get(**current_widget).ok();
    let viewport_width = own_layout.map(|l| l.width()).unwrap_or(0.0);
    let viewport_height = own_layout.map(|l| l.height()).unwrap_or(0.0);

    let content_extent = list.item_count as f32 * list.item_extent;
    context.scrollbox_width = viewport_width;
    context.scrollbox_height = viewport_height;
    context.content_width = viewport_width;
    context.content_height = content_extent;
    // Re-clamps `scroll_y` against the sizes just written above (e.g. after `item_count`
    // shrinks, or on the very first render before any size was known) without duplicating
    // `ScrollContext::set_scroll_y`'s own clamping logic here.
    let current_scroll_y = context.scroll_y();
    context.set_scroll_y(current_scroll_y);
    let scroll_offset = (-context.scroll_y()).max(0.0);

    // See `dynamic_overscan`'s own doc comment for why a fixed overscan flickers under fast
    // scrolling, and why this widens it to cover however far the offset actually moved.
    let overscan = dynamic_overscan(
        scroll_state.prev_scroll_offset,
        scroll_offset,
        list.item_extent,
        VIRTUAL_LIST_OVERSCAN,
        VIRTUAL_LIST_MAX_OVERSCAN,
    );
    scroll_state.prev_scroll_offset = scroll_offset;

    let window = compute_virtual_window(
        scroll_offset,
        viewport_height,
        list.item_extent,
        list.item_count,
        overscan,
    );

    let mut item_children = WidgetChildren::default();
    if window.lead_spacer > 0.0 {
        item_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: window.lead_spacer.into(),
                ..Default::default()
            },
        ));
        item_children.add_key("lead_spacer");
    }
    for index in window.start..window.end {
        item_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: list.item_extent.into(),
                ..Default::default()
            },
            (list.item_content.0)(index),
        ));
        item_children.add_key(format!("item{index}"));
    }
    if window.trail_spacer > 0.0 {
        item_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: window.trail_spacer.into(),
                ..Default::default()
            },
        ));
        item_children.add_key("trail_spacer");
    }

    *children = WidgetChildren::default();
    let current_widget_val = *current_widget;
    // Attached while `children`'s queue is still empty, so this observer scopes to
    // `VirtualList`'s own entity (its required `Pickable`) rather than the `Clip` child added
    // below -- see `WidgetChildren::observe`'s doc comment on "last added child" scoping.
    children.observe(
        current_widget_val,
        move |mut trigger: On<Pointer<MouseWheelScroll>>,
              mut context_query: Query<&mut ScrollContext>| {
            trigger.propagate(false);
            let Ok(mut context) = context_query.get_mut(context_entity) else {
                return;
            };
            let scroll_y = context.scroll_y();
            context.set_scroll_y(scroll_y + trigger.pixel_delta(SCROLL_LINE).y);
        },
    );
    children.add::<Clip>((
        Clip,
        WoodpeckerStyle {
            // `height` and `overflow` must stay pinned to `Clip`'s own `#[require]` defaults
            // (100% / `WidgetOverflow::Clip`) -- this bundle would otherwise silently reset
            // both back to `WoodpeckerStyle::default()`'s `Auto` / `Visible` via
            // `..Default::default()`. Without `height: 100%`, `Clip` hugs its absolutely-
            // positioned child (the mounted item window) instead of staying bounded to
            // `VirtualList`'s own fixed height; without `overflow: Clip`, that same
            // (unclipped, under `Visible`) child content still leaks into an ancestor
            // `ScrollContent`'s measured height even once `Clip`'s own outer size is bounded
            // -- see `Clip`'s own doc comment for why both matter together.
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            overflow: WidgetOverflow::Clip,
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                width: Units::Percentage(100.0),
                top: (-scroll_offset).into(),
                flex_direction: WidgetFlexDirection::Column,
                ..Default::default()
            },
            item_children,
        )),
    ));
    children.add_key("content");

    // A real `ScrollBar`, driven by the same `ScrollContext` this widget just updated above --
    // it finds that context by walking its own ancestors, landing on this entity. Gives fast
    // drag-to-scroll and a visible "where am I" thumb, on top of the wheel handler above.
    children.add::<ScrollBar>(ScrollBar {
        thickness: SCROLLBAR_THICKNESS,
        ..Default::default()
    });
    children.add_key("scrollbar");

    children.apply(current_widget_val.as_parent());
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use bevy::ecs::system::RunSystemOnce;

    use super::*;
    use crate::layout::system::ReflectedLayout;

    fn render_list(world: &mut World, entity: Entity) {
        world.insert_resource(CurrentWidget(entity));
        world.run_system_once(render).unwrap();
        world.remove_resource::<CurrentWidget>();
    }

    fn counting_item_content() -> (VirtualListItemContent, Arc<AtomicUsize>) {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_for_closure = call_count.clone();
        (
            VirtualListItemContent::new(move |_index| {
                call_count_for_closure.fetch_add(1, Ordering::SeqCst);
                WidgetChildren::default()
            }),
            call_count,
        )
    }

    #[test]
    fn viewport_sized_layout_only_builds_the_visible_window() {
        let mut world = World::new();
        world.init_resource::<HookHelper>();

        let (item_content, call_count) = counting_item_content();
        let entity = world
            .spawn((
                VirtualList {
                    item_count: 1_000,
                    item_extent: 20.0,
                    item_content,
                },
                WidgetChildren::default(),
            ))
            .id();
        world
            .entity_mut(entity)
            .insert(WidgetLayout(ReflectedLayout {
                size: Vec2::new(200.0, 100.0),
                ..Default::default()
            }));

        // The very first render only creates the scroll context entity via `hooks.use_context`
        // (whose freshly-`Commands`-spawned component isn't visible to this same system's own
        // queries yet) and bails out before reaching the item-building loop -- the same
        // one-frame-late pattern every `use_context`-based widget in this crate has. The
        // second render is the first one that actually builds a window.
        render_list(&mut world, entity);
        render_list(&mut world, entity);

        let count = call_count.load(Ordering::SeqCst);
        assert!(
            count > 0 && count < 50,
            "expected only the visible window's items to be built, got {count}"
        );
    }

    #[test]
    fn no_layout_yet_only_builds_the_overscan_without_panicking() {
        let mut world = World::new();
        world.init_resource::<HookHelper>();

        let (item_content, call_count) = counting_item_content();
        let entity = world
            .spawn((
                VirtualList {
                    item_count: 1_000,
                    item_extent: 20.0,
                    item_content,
                },
                WidgetChildren::default(),
            ))
            .id();
        // No `WidgetLayout` inserted -- simulates layout never having run yet, so the
        // viewport reads as zero-height.

        // First render only creates scroll context and bails (see the comment in
        // `viewport_sized_layout_only_builds_the_visible_window`); the second is the first to
        // actually reach the windowing logic with a missing `WidgetLayout`.
        render_list(&mut world, entity);
        render_list(&mut world, entity);

        // A zero-height viewport still yields a small, valid (not empty, not panicking)
        // window -- just the leading overscan -- rather than nothing at all. See
        // `compute_virtual_window`'s own `zero_viewport_extent_yields_a_thin_but_valid_window`
        // test for the underlying math.
        assert_eq!(call_count.load(Ordering::SeqCst), VIRTUAL_LIST_OVERSCAN);
    }

    #[test]
    fn scroll_offset_clamps_to_the_valid_scroll_range() {
        let mut world = World::new();
        world.init_resource::<HookHelper>();

        let (item_content, _call_count) = counting_item_content();
        let entity = world
            .spawn((
                VirtualList {
                    item_count: 10,
                    item_extent: 20.0, // content_extent = 200
                    item_content,
                },
                WidgetChildren::default(),
            ))
            .id();
        world
            .entity_mut(entity)
            .insert(WidgetLayout(ReflectedLayout {
                size: Vec2::new(200.0, 100.0), // viewport height 100 -> max valid offset = 100
                ..Default::default()
            }));

        // First render creates the scroll context entity (scroll_y starts at 0.0).
        render_list(&mut world, entity);
        let context_entity = world
            .resource::<HookHelper>()
            .get_context::<ScrollContext>(CurrentWidget(entity))
            .expect("render should have created a scroll context");

        // Force an out-of-range value, as if a huge wheel delta had been applied directly
        // (bypassing `set_scroll_y`'s own clamp, the way a stale/asserted value might).
        world
            .get_mut::<ScrollContext>(context_entity)
            .unwrap()
            .scroll_y = -10_000.0;

        render_list(&mut world, entity);

        let clamped = world.get::<ScrollContext>(context_entity).unwrap().scroll_y;
        assert_eq!(clamped, -100.0);
    }
}
