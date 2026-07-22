use bevy::prelude::Reflect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub(crate) enum GuideTopic {
    Overview,
    Architecture,
    BuildingAWidget,
    StateAndHooks,
    EventsAndObservers,
    StylingAndTheming,
    LayoutAndUnits,
    Composition,
    PortalsAndOverlays,
    Docking,
}

pub(crate) const GUIDE_TOPICS: &[(&str, GuideTopic)] = &[
    ("Overview", GuideTopic::Overview),
    ("Architecture", GuideTopic::Architecture),
    ("Building a Widget", GuideTopic::BuildingAWidget),
    ("State & Hooks", GuideTopic::StateAndHooks),
    ("Events & Observers", GuideTopic::EventsAndObservers),
    ("Styling & Theming", GuideTopic::StylingAndTheming),
    ("Layout & Units", GuideTopic::LayoutAndUnits),
    ("Composition", GuideTopic::Composition),
    ("Portals & Overlays", GuideTopic::PortalsAndOverlays),
    ("Docking", GuideTopic::Docking),
];

pub(crate) fn guide_title(topic: GuideTopic) -> &'static str {
    GUIDE_TOPICS
        .iter()
        .find(|(_, t)| *t == topic)
        .map(|(label, _)| *label)
        .unwrap()
}

pub(crate) fn markdown_source(topic: GuideTopic) -> &'static str {
    match topic {
        GuideTopic::Overview => OVERVIEW,
        GuideTopic::Architecture => ARCHITECTURE,
        GuideTopic::BuildingAWidget => BUILDING_A_WIDGET,
        GuideTopic::StateAndHooks => STATE_AND_HOOKS,
        GuideTopic::EventsAndObservers => EVENTS_AND_OBSERVERS,
        GuideTopic::StylingAndTheming => STYLING_AND_THEMING,
        GuideTopic::LayoutAndUnits => LAYOUT_AND_UNITS,
        GuideTopic::Composition => COMPOSITION,
        GuideTopic::PortalsAndOverlays => PORTALS_AND_OVERLAYS,
        GuideTopic::Docking => DOCKING,
    }
}

const OVERVIEW: &str = r#"
Woodpecker UI is a retained-mode, reactive UI library built directly on top of Bevy's ECS --
there's no separate scene graph or widget tree living outside the world. A widget is just an
entity with a marker component, a `WoodpeckerStyle`, and a `WidgetChildren` describing what it
should currently contain.

Instead of a `view()` function you call once, each widget type has a render system that Bevy
reruns automatically whenever something that system reads has changed -- a prop on the widget's
own component, a piece of state it owns, or a context value an ancestor provides. You describe
what the widget should look like right now; the library figures out when to call that
description again.

## What this guide covers

The pages in this section walk through the pieces you need to build your own widgets: the
`Widget` derive and render systems, per-widget state via hooks, the event/observer pattern,
styling and layout, composing widgets out of other widgets, and the portal/overlay system
floating UI (modals, drawers, popovers) is built on. Every component catalogued in the sidebar
below this section is built using exactly these pieces -- nothing about them is special-cased by
the framework.
"#;

const ARCHITECTURE: &str = r#"
Every widget is an entity. Its `WidgetChildren` component lists the child widgets it wants to
exist right now, each keyed by an identity (its position by default, or an explicit key you
supply). Every frame, the reconciler compares that list against what actually exists and spawns,
updates, or despawns entities to match -- the same idea as a virtual-DOM diff, except the "DOM"
is the ECS world itself.

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<MyWidget>()
        .add_systems(Startup, startup)
        .run();
}
```

`register_widget::<T>()` wires up the machinery that runs `T`'s render system whenever it needs
to. Inside that system, the `CurrentWidget` resource tells you which entity you're currently
rendering for -- almost every widget's render system takes it as its first parameter and uses it
to look up its own components and to parent its children.

> Keys matter: a child added without an explicit key is identified by its position in the list.
> If a list's item order can change (reordering, filtering, insertion in the middle) always call
> `.with_key(...)` / `.add_key(...)` with a stable identifier -- otherwise the reconciler will
> match the wrong old entity to the wrong new child and any per-child state (hooks, animation,
> scroll position) will jump to the wrong item.

## Optional: bsn!

Behind the `bevy_bsn` feature flag, Bevy's own `bsn!` macro can replace the `WidgetChildren`
builder chain for the one-shot tree you spawn in `startup()`:

```rust
commands.entity(*root_widget).apply_scene(bsn! {
    Children [
        MyWidget { some_prop: 0 },
    ]
});
```

`bsn!`'s scene system resolves and spawns once, with no concept of the per-frame diff/patch
cycle `WidgetChildren` reconciliation depends on -- a `Children [...]` block would spawn brand
new child entities every render if used inside a widget's own `render`, not reconciled ones. So
inside `render`, reach for `WidgetChildren::add_scene` instead of `add`/`with_child` to describe
one child's own components with `bsn!`, while still nesting further widgets and attaching
observers the normal way:

```rust
children.add_scene::<Element, _>(bsn! {
    WoodpeckerStyle { font_size: 24.0 }
    WidgetRender::Text { content: label_text }
});
children.add_key("label");
```

This still goes through the normal keyed reconciliation -- same entity reused across renders,
its components patched in place each render, exactly as if you'd called `add` with a tuple
bundle. `bsn!` here only replaces how *that one child's own components* are described, never
how it's found, reused, or nested.
"#;

const BUILDING_A_WIDGET: &str = r#"
A widget needs three things: a marker component deriving `Widget`, a render system registered
against it with `#[auto_update(...)]`, and a `#[require(...)]` for the components every instance
needs by default (almost always `WoodpeckerStyle` and `WidgetChildren`).

```rust
#[derive(Widget, Component, Reflect, PartialEq, Default, Debug, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct Counter {
    initial_count: u32,
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&Counter, &mut WidgetChildren)>,
) {
    let Ok((widget, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;

    *children = WidgetChildren::default().with_child::<Element>((
        Element,
        WidgetRender::Text {
            content: format!("Count: {}", widget.initial_count),
        },
    ));

    children.apply(current_widget.as_parent());
}
```

> Never forget the trailing `children.apply(current_widget.as_parent())` -- building a
> `WidgetChildren` value only describes the desired children; `apply` is what actually
> reconciles them against the world. Every widget's render system ends with this line.

The widget's own component (`Counter` above) is its public API -- the fields callers set when
they spawn it, like props in other UI frameworks. It must derive `Reflect` with
`#[reflect(Component, DiffableProp, PartialEq)]` so the framework can detect when a caller
changes those fields and knows to re-run `render`.
"#;

const STATE_AND_HOOKS: &str = r#"
A widget's own component is owned by its caller -- re-rendering the parent can overwrite it. For
state a widget owns and mutates itself (an expanded flag, a text cursor position, a counter), use
`HookHelper::use_state` to get a persistent "state entity" tied to the widget's own entity
instead.

```rust
#[derive(Component, PartialEq, Default, Debug, Clone, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct CounterState {
    count: u32,
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    state_query: Query<&CounterState>,
    mut query: Query<&mut WidgetChildren>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity = hooks.use_state(&mut commands, *current_widget, CounterState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let current_widget = *current_widget;

    *children = WidgetChildren::default().with_child::<Element>((
        Element,
        WidgetRender::Text { content: format!("Count: {}", state.count) },
    ));
    children.apply(current_widget.as_parent());
}
```

`use_state` finds-or-creates the state entity: the first render spawns it with the initial value
you pass, every later render finds the same entity again (the initial value is ignored once it
exists), so the state genuinely persists across renders. The state component needs the same
`Reflect`/`DiffableProp` derive as a widget's own component -- without it, mutating the state
entity elsewhere (from an observer, say) won't be noticed and the widget will silently stop
re-rendering when its state changes.

## Sharing state with descendants: use_context

`use_context` works like `use_state` but walks up the widget tree looking for an existing context
entity of that type on an ancestor before creating a new one -- it's how a provider widget higher
up (a theme override, a scroll container) shares state with descendants without threading it
through every widget in between. `use_own_context` is the same idea but never inherits from an
ancestor -- always finds-or-creates directly on the current widget -- for state that's private
bookkeeping rather than something meant to be shared.

## Reacting to one specific change: use_effect

A render can be triggered by all sorts of unrelated things -- a sibling's context changed, an
ancestor re-declared this widget's bundle. `use_effect` reports whether *one particular*
dependency changed since the last time it was called, so a render system can run a side effect
(fire an event, kick off an animation) only when that specific thing changed, not on every
unrelated render.

```rust
fn render(
    current_widget: Res<CurrentWidget>,
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    widget: Query<&MyWidget>,
    effect_query: Query<&EffectDep<bool>>,
) {
    let Ok(widget) = widget.get(**current_widget) else {
        return;
    };
    if hooks.use_effect(&mut commands, *current_widget, &effect_query, widget.open) {
        commands.trigger(Change { target: *current_widget, data: OpenChanged(widget.open) });
    }
}
```

Needs a `Query<&EffectDep<T>>` alongside your own state query -- `HookHelper` has no way to read
a component's value back on its own, the same reason `use_state` needs a `Query<&MyState>` too.

## Remembering the last value: use_previous

`use_previous` returns whatever value you passed in *last* render (`None` on the first call) --
useful for detecting a transition, like "just became visible", rather than only ever seeing the
current state.

```rust
let was_open = hooks.use_previous(&mut commands, *current_widget, &previous_query, widget.open);
let just_opened = was_open == Some(false) && widget.open;
```

## Caching an expensive computation: use_memo

`use_memo` recomputes a derived value only when its dependency changes, returning the cached
value otherwise -- for something too expensive to rebuild on every render the way a widget's
children normally are.

```rust
let sorted: Vec<String> = hooks.use_memo(
    &mut commands,
    *current_widget,
    &memo_query,
    widget.items.clone(),
    |items| {
        let mut sorted = items.clone();
        sorted.sort();
        sorted
    },
);
```

## Timers, intervals, and debouncing

Three more hooks cover the "do something after time passes" family, each keyed off a `Duration`
you supply (usually `Res<Time>::elapsed()`):

- `use_timer` fires `true` exactly once, on the first render at or past a given duration since
  it was first called -- a one-shot timeout (an auto-dismissing toast, say).
- `use_interval` fires `true` repeatedly, once per duration, resetting each time it fires -- a
  repeating tick.
- `use_debounce` returns `Some(value)` once `value` has stopped changing for a given duration,
  `None` while it's still settling -- useful for search-as-you-type or a resize in progress.

```rust
if hooks.use_timer(&mut commands, *current_widget, &timer_query, Duration::from_secs(4), time.elapsed()) {
    commands.trigger(Change { target: *current_widget, data: Dismissed });
}
```

All three keep the widget re-rendering every frame for as long as they're pending (a timer not
yet fired, an interval that never truly settles, a value still within its debounce window) --
you don't need to do anything extra to make that happen, unlike a plain `use_state` value that
only changes when someone else mutates it.
"#;

const EVENTS_AND_OBSERVERS: &str = r#"
Widgets react to input through Bevy observers attached while building a `WidgetChildren`.
`.with_observe(current_widget, system)` attaches to the child you just added with `.with_child`;
`.observe(current_widget, system)` does the same but as a separate call against the last child
added via `.add`. Both take an ordinary Bevy observer system, most often listening for
`Pointer<Click>`.

```rust
*children = WidgetChildren::default().with_child::<WButton>((
    WButton,
    WidgetChildren::default().with_child::<Element>((
        Element,
        WidgetRender::Text { content: "Click me".into() },
    )),
)).with_observe(
    current_widget,
    move |_: On<Pointer<Click>>, mut query: Query<&mut CounterState>| {
        if let Ok(mut state) = query.get_mut(state_entity) {
            state.count += 1;
        }
    },
);
```

Some widgets don't build their own children (their content arrives pre-populated via the
caller's spawn bundle) but still need an observer on themselves -- use
`.self_observe(current_widget, system)` on the `WidgetChildren` for that, since `.observe` would
otherwise misattach to whichever child happens to already be queued.

## Change<T>: a widget's own events

Widgets that need to tell their caller something changed (a checkbox's checked state, a list item
being clicked) fire a `Change<T>` event: `commands.trigger(Change { target: entity, data })`.
Callers listen the same way as any other event, matching on `On<Change<T>>` in an observer.

> Observers attached via `.observe`/`.self_observe` are only ever spawned once per (widget,
> target) pair -- re-declaring the same observer on a later render is a no-op, it does not
> replace the old closure. Never capture a per-render snapshot of a value inside one of these
> closures; read the current value fresh through a `Query` when the observer actually fires, the
> same way the examples above read `state_entity` (which is stable across renders) rather than a
> captured value from `state`.
"#;

const STYLING_AND_THEMING: &str = r#"
`WoodpeckerStyle` is a flexbox-style layout and paint description -- width, height,
flex_direction, padding, margin, border, gap, align_items, justify_content, colors, and more.
Its default is `Auto` width/height and `Row` flex direction, which is easy to forget: a widget
that requires `WoodpeckerStyle` but never explicitly sets its own `flex_direction`/`width`/
`height` will silently hug its content in a row, instead of filling its parent or stacking
children vertically as you probably intended.

```rust
*styles = WoodpeckerStyle {
    width: Units::Percentage(100.0),
    height: Units::Percentage(100.0),
    flex_direction: WidgetFlexDirection::Column,
    padding: Edge::all(theme.spacing.md),
    background_color: theme.background,
    border_color: theme.border,
    border: Edge::all(1.0),
    border_radius: Corner::all(theme.panel_radius),
    ..*styles
};
```

The `Theme` resource centralizes colors, spacing, and type scale so widgets share a consistent
look instead of hard-coding numbers: `theme.primary`/`theme.background`/`theme.border`/
`theme.text` for color, `theme.spacing.{xs,sm,md,lg,xl}` for a spacing scale,
`theme.typography.{h1,h2,h3,body,body_small,caption}` for font sizes, and
`theme.elevation.{sm,md,lg,xl}` for ready-made box-shadow presets. Reach for these before
hard-coding a color or pixel value.
"#;

const LAYOUT_AND_UNITS: &str = r#"
Every size-like field on `WoodpeckerStyle` (width, height, padding, margin, ...) takes a `Units`
value: `Units::Pixels(f32)`, `Units::Percentage(f32)`, `Units::Auto` (hug content), or
`Units::Calc { percent, pixels }` -- a CSS `calc()`-style mix of a percentage of the parent plus
a fixed pixel offset, resolved once the parent's own size is known.

```rust
WoodpeckerStyle {
    width: Units::Calc { percent: 100.0, pixels: -32.0 },
    ..Default::default()
}
```

Reach for `Calc` specifically when a size needs to track the parent's size AND include a fixed
offset at the same time -- a fixed-width sidebar plus a flexible remainder is better expressed
with `flex_grow`/a fixed sidebar width than `Calc`; `Calc` earns its keep for things like "100%
minus this button's own width" that flex layout alone can't express.

A text element's `max_width` is respected by its own text wrapping, not just the final layout
box -- a caption capped narrower than its parent wraps (and reserves height for) that narrower
width, not the parent's full width.
"#;

const COMPOSITION: &str = r#"
`WidgetChildren` is what a widget builds for *itself* -- the content its own render system
decides should exist. `PassedChildren` is different: it wraps content a *caller* handed to a
composing widget (a `Modal`, `ScrollBox`, `Card`) for that widget to place somewhere inside its
own output. A composing widget's render system takes both: its own `WidgetChildren` (chrome it
builds, like a modal's title bar) and a `PassedChildren` it re-embeds somewhere inside that
chrome.

```rust
WidgetChildren::default().with_child::<ScrollBox>((
    ScrollBox::default(),
    PassedChildren(
        WidgetChildren::default().with_child::<Element>((
            Element,
            WidgetRender::Text { content: "Scrollable content".into() },
        )),
    ),
))
```

This split is why a widget like `List` documents its child slot as accepting `ListItem` rather
than flattening `ListItem`'s own fields into `List`'s prop table -- the children a caller passes
in are a genuinely separate concept from the composing widget's own props.
"#;

const PORTALS_AND_OVERLAYS: &str = r#"
Floating UI -- `Modal`, `Drawer`, `Popover`, `WoodpeckerWindow` -- needs to paint above everything
else regardless of where it's declared in the widget tree. `.portal()` / `.portal_to(target)` on
a `WidgetChildren` entry reparents that child's *real* Bevy `ChildOf` to an overlay root (or an
explicit target entity) so it paints on top, while a separate `LogicalParent` still points at its
original declaring widget.

That `LogicalParent` link exists purely for things like theme/context inheritance -- a portaled
modal still sees the `Theme` overrides its logical ancestors provide, even though it isn't really
a Bevy child of them anymore. It is not a substitute for real parenting when it comes to
lifecycle: the framework's own reconciler is what notices a portaled widget's logical ancestor
disappearing and cleans it up, since native Bevy despawn-cascade only ever follows the real
(portaled) hierarchy, which by then points somewhere else entirely.

> You won't normally call `.portal()` yourself -- it's what `Modal`/`Drawer`/`Popover`/
> `WoodpeckerWindow` use internally to float above the page. It's worth understanding mainly so a
> portaled widget's behavior (painting above siblings, surviving scroll clipping, but still
> inheriting theme from where it was declared) makes sense.
"#;

const DOCKING: &str = r#"
`DockArea` is a resizable, tabbed, redockable panel layout -- the kind of shell an editor or
inspector-style tool UI needs, built entirely from primitives the rest of this guide already
covers: `use_context` for shared state, `Change<T>` events from `Splitter`, and `.portal()` for
floated panels (`DockArea` reuses `WoodpeckerWindow` directly for those, rather than building its
own floating chrome).

## Registering panels

A caller never builds panel content up front. `DockArea::panels` takes a `DockPanels` wrapping a
`Vec<PanelDef>`, each pairing a stable `PanelId` and title with a *factory* closure:

```rust
DockArea {
    initial_tree: DockTree::single(PanelId::new("inspector")),
    panels: DockPanels::new(vec![
        PanelDef::new("inspector", "Inspector", |_id| my_inspector_panel()),
    ]),
}
```

The factory only runs for the *active* tab of each group -- a panel sitting behind another tab,
or in a still-collapsed part of the tree, never pays to build its content at all.

## Why a panel's own state survives being redocked

Every panel's content is declared with `.with_key(panel_id)` at the point it's rendered. Dragging
a tab to a new split or tab group changes *where* in the tree that key is rendered, but the key
itself doesn't change -- so the framework's normal keyed-child reconciliation reuses the same
entity, and whatever `use_state` that panel's own content set up (scroll position, expanded
tree nodes, form input) survives the move untouched.

## Reading the tree from outside

`DockArea` owns its tree as a `DockTree` context, seeded once from `initial_tree` and mutated
from then on by drag/redock/close. A caller wanting to inspect or reset the live layout (e.g. a
"reset to default layout" button elsewhere in the app) reaches it the same way any descendant
does:

```rust
if let Some(tree_entity) = hooks.get_context::<DockTree>(current_widget) {
    if let Ok(mut tree) = tree_query.get_mut(tree_entity) {
        *tree = DockTree::single(PanelId::new("inspector"));
    }
}
```

> There's no save/load-to-disk story yet -- `DockTree`/`PanelId` are deliberately kept
> serialization-shaped (plain enums, string-keyed) even without a `Serialize` derive, so adding
> that later doesn't need a rework, but today a fresh `DockArea` always starts from whatever
> `initial_tree` the caller passes in.
"#;
