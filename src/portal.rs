use bevy::prelude::*;

/// Opt-in marker on a `WidgetChildren`-declared child: instead of physically parenting
/// (`ChildOf`) the spawned entity to the widget that declared it, it's parented elsewhere --
/// `None` defaults to the crate's shared [`OverlayRoot`], `Some(entity)` targets a specific
/// `StackingContext`-bearing entity instead (e.g. a nested overlay-within-overlay scenario).
/// This is the React-portal equivalent: it decouples *where a widget physically renders/picks*
/// from *where it's logically declared* (preserved via [`LogicalParent`]).
///
/// Set via [`crate::children::WidgetChildren::portal`]/`.portal_to(target)`, called
/// immediately after `.add::<T>()`, the same convention as `.add_key(...)`. Only takes effect
/// the first time the entity is spawned -- re-renders that reuse the same keyed entity never
/// touch its `ChildOf`/`Portal`/`LogicalParent` again.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Portal(pub Option<Entity>);

/// The entity a portaled child was *declared* under, preserved even though its real `ChildOf`
/// points somewhere else entirely. `HookHelper`'s context/theme ancestry walk prefers this
/// over `ChildOf` when present, so a portaled widget's descendants still resolve context as if
/// the widget had never moved. Set once at spawn (alongside `ChildOf` and `Portal`, in the same
/// `Commands`/`World` mutation), never touched again -- a widget's logical position in the
/// tree doesn't change across re-renders even if its physical position could in principle.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct LogicalParent(pub Entity);

/// The entity every `.portal()` call (with no explicit `.portal_to(target)`) resolves to by
/// default. Not auto-provisioned by this module -- see the doc comment on the resource itself
/// for who's responsible for spawning and inserting it.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub struct OverlayRoot(pub Entity);

/// Opts an entity out of `layout::system::traverse_layout_update`'s dev-build lint, which
/// otherwise flags any non-portaled entity using a built-in overlay `WidgetZ::Global` tier as
/// the fingerprint of a pre-portal-migration escape-hatch that needs `.portal()`. A handful of
/// legitimate exceptions exist -- e.g. `devtools::panel::DevtoolsRoot`, which is *documented* to
/// require being declared as a direct root-level sibling of the app, so it never needs to escape
/// an ancestor's ambient content the way a deeply-nested `Modal`/`Toast` does; portaling it would
/// only match the portaled built-ins without fixing anything (confirmed empirically). Insert
/// this directly on such an entity's own bundle to silence the false positive.
#[derive(Component, Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct SkipPortalLint;
