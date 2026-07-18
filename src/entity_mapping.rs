use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    portal::{LogicalParent, OverlayRoot, Portal},
    runner::get_all_children,
    ObserverCache, ParentWidget,
};

/// Maps parent widgets to child widgets.
///
/// Reconciliation is keyed (like React's list-diffing) rather than positional: a child's
/// entity identity survives being reordered, inserted around, or removed elsewhere in its
/// parent's child list. Only a child whose key genuinely disappears from the new list gets
/// despawned.
#[derive(Resource, Default)]
pub struct WidgetMapper {
    parent_entity_to_child: HashMap<ParentWidget, ParentChildMap>,
}

#[derive(Default)]
struct ParentChildMap {
    /// The authoritative (key -> entity) mapping as of the end of the last completed
    /// reconciliation pass for this parent.
    by_key: HashMap<String, Entity>,
    /// Present only while a reconciliation pass is in progress for this parent this frame.
    pending: Option<PendingReconciliation>,
}

#[derive(Default)]
struct PendingReconciliation {
    /// Entries not yet claimed by a key seen so far in this frame's pass. Whatever remains
    /// here when the pass finishes is genuinely removed and gets despawned.
    unclaimed: HashMap<String, Entity>,
    /// The new (key -> entity) map being built up as this frame's children are processed.
    resolved: HashMap<String, Entity>,
    /// Resolved keys in render order. Used for the final `Children` reorder.
    order: Vec<String>,
    /// How many times each base key has been seen so far this pass, for duplicate-key
    /// disambiguation.
    occurrences: HashMap<String, usize>,
}

impl WidgetMapper {
    /// Create a new widget mapper.
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a reconciliation pass for `parent`, snapshotting its current (key -> entity)
    /// mapping so [`Self::get_or_insert_entity_world`] can reuse keys regardless of position.
    ///
    /// Must be called exactly once per parent per frame, before any
    /// [`Self::get_or_insert_entity_world`] calls for that parent -- even with zero children,
    /// so stale children from a previous frame still get cleaned up.
    pub(crate) fn start_reconciliation(&mut self, parent: ParentWidget) {
        let map = self.parent_entity_to_child.entry(parent).or_default();
        map.pending = Some(PendingReconciliation {
            unclaimed: std::mem::take(&mut map.by_key),
            ..Default::default()
        });
    }

    /// Looks up the entity a keyed child currently resolves to, as of the end of `parent`'s
    /// last completed reconciliation pass -- i.e. one frame behind whatever this frame's own
    /// `get_or_insert_entity_world` calls are about to (re)declare. Exists for widgets that
    /// need to read a just-declared child's own `WidgetLayout` (position/size only settle
    /// after layout runs, which is after this frame's render), the same one-frame-lag
    /// tradeoff `devtools/highlight.rs` already accepts elsewhere in this crate. Returns
    /// `None` before the key has ever been resolved (e.g. the very first render).
    ///
    /// `key` must be the exact *resolved* key `get_or_insert_entity_world` stored -- for a
    /// child added via `.add::<T>(...).add_key("foo")`, that's `"{T::get_name()}-foo"`, not
    /// the bare `"foo"` passed to `.add_key`. [`Self::get_keyed_child`] builds this composite
    /// key for you and should be preferred over calling this directly with a hand-built string.
    pub(crate) fn get_child(&self, parent: ParentWidget, key: &str) -> Option<Entity> {
        self.parent_entity_to_child
            .get(&parent)?
            .by_key
            .get(key)
            .copied()
    }

    /// Same as [`Self::get_child`], but reconstructs the composite key
    /// `get_or_insert_entity_world` stores for an explicitly-keyed child of widget type `T` --
    /// i.e. what `.add::<T>(...).add_key(key)` actually resolves to internally
    /// (`"{T::get_name()}-{key}"`, not the bare `key` string). Prefer this over calling
    /// [`Self::get_child`] directly: hand-reconstructing the composite key at each call site
    /// is easy to get subtly wrong (silently returning `None` forever, since the mismatched
    /// key never resolves to anything) -- see the regression test below.
    pub(crate) fn get_keyed_child<T: crate::context::Widget>(
        &self,
        parent: ParentWidget,
        key: &str,
    ) -> Option<Entity> {
        self.get_child(parent, &format!("{}-{key}", T::get_name()))
    }

    /// Resolves (spawning if necessary, reusing if a matching key already existed) the
    /// entity for the next child under `parent`, in render order.
    ///
    /// `portal` mirrors [`crate::children::WidgetChildren::portal`]/`.portal_to(target)`:
    /// `None` means "physically parent to `parent`, like every non-portaled child always
    /// has"; `Some(Portal(None))` physically parents to [`OverlayRoot`] instead;
    /// `Some(Portal(Some(target)))` to `target`. Only consulted the first time this entity is
    /// spawned -- a reused (already-existing) entity's `ChildOf`/`Portal`/`LogicalParent` are
    /// never touched again, matching `LogicalParent`'s own "set once" contract.
    ///
    /// [`Self::start_reconciliation`] must have already been called for `parent` this
    /// frame, and [`Self::finish_reconciliation`] must be called once after the last child
    /// has been processed.
    pub(crate) fn get_or_insert_entity_world(
        &mut self,
        world: &mut World,
        widget_name: String,
        parent: ParentWidget,
        child_key: Option<String>,
        portal: Option<Portal>,
    ) -> Entity {
        let base_key = if let Some(child_key) = child_key {
            format!("{widget_name}-{child_key}")
        } else {
            widget_name
        };

        let map = self.parent_entity_to_child.entry(parent).or_default();
        let pending = map.pending.as_mut().expect(
            "WidgetMapper::start_reconciliation must be called for a parent before \
             get_or_insert_entity_world is called for any of its children",
        );

        let occurrence = *pending.occurrences.get(&base_key).unwrap_or(&0);
        pending.occurrences.insert(base_key.clone(), occurrence + 1);
        let resolved_key = if occurrence == 0 {
            base_key
        } else {
            warn!(
                "Woodpecker UI: duplicate widget key \"{base_key}\" used more than once under \
                 the same parent. Give each item a unique `.with_key(...)` to preserve entity \
                 identity across re-renders (e.g. when reordering/removing items from a list)."
            );
            format!("{base_key}#{occurrence}")
        };

        // Computed as an owned value so no borrow of `self` is held across `world.spawn`.
        let existing = pending.unclaimed.remove(&resolved_key);
        let entity = existing.unwrap_or_else(|| {
            let Some(portal) = portal else {
                return world.spawn(ChildOf(*parent)).id();
            };
            let physical_parent = portal
                .0
                .unwrap_or_else(|| world.resource::<OverlayRoot>().0);
            world
                .spawn((ChildOf(physical_parent), portal, LogicalParent(*parent)))
                .id()
        });

        let pending = self
            .parent_entity_to_child
            .get_mut(&parent)
            .expect("parent's map entry can't have disappeared mid-reconciliation")
            .pending
            .as_mut()
            .expect("pending reconciliation can't have disappeared mid-reconciliation");
        pending.resolved.insert(resolved_key.clone(), entity);
        pending.order.push(resolved_key);

        entity
    }

    /// Must be called once, after all of a parent's children have been processed this frame
    /// via [`Self::get_or_insert_entity_world`]. Despawns any child whose key wasn't claimed
    /// by the new child list, and reorders the parent's `Children` to match render order.
    pub(crate) fn finish_reconciliation(
        &mut self,
        world: &mut World,
        observer_cache: &mut ObserverCache,
        parent: ParentWidget,
    ) {
        let Some(map) = self.parent_entity_to_child.get_mut(&parent) else {
            return;
        };
        let Some(pending) = map.pending.take() else {
            return;
        };

        // Anything left unclaimed genuinely disappeared from the new child list --
        // recursively clean up bookkeeping and despawn it (Bevy's hierarchy relationship
        // cascades the despawn to descendants automatically).
        for entity_to_remove in pending.unclaimed.into_values() {
            if world.get_entity(entity_to_remove).is_err() {
                continue;
            }

            observer_cache.despawn_for_target(world, entity_to_remove);
            self.despawn_portaled_descendants(world, observer_cache, entity_to_remove);

            for child in get_all_children(world, entity_to_remove) {
                if world.get_entity(child).is_err() {
                    continue;
                }
                let child_parent = world
                    .entity(child)
                    .get::<ChildOf>()
                    .expect(
                        "Unknown dangling child! This is an error with woodpecker UI source \
                         please file a bug report.",
                    )
                    .parent();
                self.remove_by_entity_id(child_parent, child);
                self.parent_entity_to_child.remove(&ParentWidget(child));
                observer_cache.despawn_for_target(world, child);
                self.despawn_portaled_descendants(world, observer_cache, child);
            }

            self.parent_entity_to_child
                .remove(&ParentWidget(entity_to_remove));
            world.entity_mut(entity_to_remove).despawn();
        }

        // Portaled entities are physically parented (`ChildOf`) elsewhere entirely (see
        // `get_or_insert_entity_world`) -- they're still tracked here for identity-reuse and
        // despawn-on-disappear (the loop above doesn't care where `ChildOf` points), just
        // excluded from this parent's own `Children` order.
        let mut ordered_entities = pending
            .order
            .iter()
            .map(|key| pending.resolved[key])
            .filter(|entity| !world.entity(*entity).contains::<Portal>())
            .collect::<Vec<_>>();

        // Preserve any existing child this pass's declared-children bookkeeping doesn't know
        // about at all, rather than letting `replace_related` below sever it. Three cases land
        // here: `parent` is itself a portal *target* (e.g. `OverlayRoot`) with children
        // physically attached by a completely different widget's reconciliation pass; `parent`
        // has its own self-observers (`WidgetChildren::self_observe`/`observe`'s self-target
        // branch spawn them as `ChildOf(parent)` directly, bypassing
        // `get_or_insert_entity_world` entirely, so they're invisible to `pending`/`by_key`
        // above); or `parent` has `hooks.use_state`/`use_context` entities (same story --
        // `HookHelper` spawns and tracks those itself, keyed off `parent`'s own entity ID, not
        // through this reconciliation at all). In every case the entity is invisible to *this*
        // pass's `pending`/`by_key` bookkeeping, not because it disappeared -- anything that
        // really was declared but vanished this frame is already despawned by this point (via
        // `pending.unclaimed` above), so anything still here and unaccounted for is legitimate.
        if let Some(existing_children) = world.get::<Children>(parent.entity()) {
            for child in existing_children.iter() {
                if !ordered_entities.contains(&child) {
                    ordered_entities.push(child);
                }
            }
        }

        if world.get_entity(parent.entity()).is_ok() {
            world
                .entity_mut(parent.entity())
                .replace_related::<ChildOf>(&ordered_entities);
        }

        let map = self
            .parent_entity_to_child
            .get_mut(&parent)
            .expect("parent's map entry can't have disappeared during its own reconciliation");
        map.by_key = pending.resolved;
    }

    pub(crate) fn remove_by_entity_id(&mut self, parent: Entity, entity: Entity) {
        if let Some(map) = self.parent_entity_to_child.get_mut(&ParentWidget(parent)) {
            map.by_key.retain(|_, e| *e != entity);
            if let Some(pending) = map.pending.as_mut() {
                pending.resolved.retain(|_, e| *e != entity);
                pending.unclaimed.retain(|_, e| *e != entity);
            }
        }
    }

    /// Recursively despawns any entity `entity` (or a normal descendant of it) declared as a
    /// portal target. Bevy's own `despawn()` cascade only follows real `ChildOf` relationships,
    /// but a portaled entity is physically parented to `OverlayRoot` (or another
    /// `StackingContext`), not to whatever logically declared it -- so it's invisible to that
    /// cascade and to `get_all_children`'s own physical-hierarchy walk. Without this, a widget
    /// with a portaled child (`Modal`/`Drawer`/`WoodpeckerWindow`/etc.) whose *ancestor* gets
    /// removed wholesale (rather than the widget itself individually re-rendering and dropping
    /// the key) never gets a chance to notice and clean up its own portaled content through its
    /// own reconciliation pass -- the portaled entity keeps rendering forever, orphaned.
    fn despawn_portaled_descendants(
        &mut self,
        world: &mut World,
        observer_cache: &mut ObserverCache,
        entity: Entity,
    ) {
        let Some(map) = self.parent_entity_to_child.remove(&ParentWidget(entity)) else {
            return;
        };
        for child in map.by_key.into_values() {
            if world.get_entity(child).is_err() {
                continue;
            }
            self.despawn_portaled_descendants(world, observer_cache, child);
            if world.entity(child).contains::<Portal>() {
                observer_cache.despawn_for_target(world, child);
                world.entity_mut(child).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reconcile(
        world: &mut World,
        mapper: &mut WidgetMapper,
        parent: ParentWidget,
        keys: &[&str],
    ) -> Vec<Entity> {
        reconcile_with_portals(
            world,
            mapper,
            parent,
            &keys.iter().map(|&key| (key, None)).collect::<Vec<_>>(),
        )
    }

    fn reconcile_with_portals(
        world: &mut World,
        mapper: &mut WidgetMapper,
        parent: ParentWidget,
        keys: &[(&str, Option<Portal>)],
    ) -> Vec<Entity> {
        mapper.start_reconciliation(parent);
        let entities = keys
            .iter()
            .map(|(key, portal)| {
                mapper.get_or_insert_entity_world(
                    world,
                    "Element".to_string(),
                    parent,
                    Some(key.to_string()),
                    *portal,
                )
            })
            .collect::<Vec<_>>();
        let mut observer_cache = ObserverCache::default();
        mapper.finish_reconciliation(world, &mut observer_cache, parent);
        entities
    }

    /// Removing an item from the middle of a keyed list must not disturb the entity
    /// identity of items that come after it, and the parent's `Children` order must match
    /// the new declared order.
    #[test]
    fn remove_from_middle_preserves_entity_identity_and_reorders_children() {
        let mut world = World::new();
        let parent_entity = world.spawn_empty().id();
        let parent = ParentWidget(parent_entity);
        let mut mapper = WidgetMapper::new();

        let keys = ["item0", "item1", "item2", "item3", "item4"];
        let first_pass = reconcile(&mut world, &mut mapper, parent, &keys);

        let children = world
            .entity(parent_entity)
            .get::<Children>()
            .expect("parent should have children after first reconciliation")
            .iter()
            .collect::<Vec<_>>();
        assert_eq!(
            children, first_pass,
            "initial Children order must match declared order"
        );

        // Remove "item1" from the middle.
        let remaining_keys = ["item0", "item2", "item3", "item4"];
        let second_pass = reconcile(&mut world, &mut mapper, parent, &remaining_keys);

        // item0, item2, item3, item4 (originally at indices 0, 2, 3, 4) must keep their
        // original entity identity.
        assert_eq!(second_pass[0], first_pass[0], "item0 entity must be reused");
        assert_eq!(second_pass[1], first_pass[2], "item2 entity must be reused");
        assert_eq!(second_pass[2], first_pass[3], "item3 entity must be reused");
        assert_eq!(second_pass[3], first_pass[4], "item4 entity must be reused");

        // item1's entity must have been despawned.
        assert!(
            world.get_entity(first_pass[1]).is_err(),
            "item1's entity must be despawned once its key disappears"
        );

        // Children order must reflect the new declared order, not the old positions.
        let children = world
            .entity(parent_entity)
            .get::<Children>()
            .expect("parent should have children after second reconciliation")
            .iter()
            .collect::<Vec<_>>();
        assert_eq!(
            children, second_pass,
            "Children order must match the new declared order"
        );
    }

    /// A parent's children being reordered (no additions/removals) must reuse every
    /// existing entity, matching identity purely by key rather than position.
    #[test]
    fn reordering_keys_preserves_entity_identity() {
        let mut world = World::new();
        let parent_entity = world.spawn_empty().id();
        let parent = ParentWidget(parent_entity);
        let mut mapper = WidgetMapper::new();

        let keys = ["a", "b", "c"];
        let first_pass = reconcile(&mut world, &mut mapper, parent, &keys);

        let reordered_keys = ["c", "a", "b"];
        let second_pass = reconcile(&mut world, &mut mapper, parent, &reordered_keys);

        assert_eq!(second_pass[0], first_pass[2], "\"c\" must keep its entity");
        assert_eq!(second_pass[1], first_pass[0], "\"a\" must keep its entity");
        assert_eq!(second_pass[2], first_pass[1], "\"b\" must keep its entity");

        let children = world
            .entity(parent_entity)
            .get::<Children>()
            .expect("parent should have children")
            .iter()
            .collect::<Vec<_>>();
        assert_eq!(
            children, second_pass,
            "Children order must match the reordered declaration"
        );
    }

    /// Going from N children to zero children in one pass must despawn every previous
    /// child, even though `get_or_insert_entity_world` is never called that frame.
    #[test]
    fn dropping_to_zero_children_despawns_everything() {
        let mut world = World::new();
        let parent_entity = world.spawn_empty().id();
        let parent = ParentWidget(parent_entity);
        let mut mapper = WidgetMapper::new();

        let keys = ["a", "b"];
        let first_pass = reconcile(&mut world, &mut mapper, parent, &keys);

        let empty: [&str; 0] = [];
        let second_pass = reconcile(&mut world, &mut mapper, parent, &empty);
        assert!(second_pass.is_empty());

        for entity in first_pass {
            assert!(
                world.get_entity(entity).is_err(),
                "all previous children must be despawned when the new list is empty"
            );
        }
        assert!(world.entity(parent_entity).get::<Children>().is_none());
    }

    /// Two children resolving to the same key under one parent (e.g. an unkeyed loop)
    /// must not silently alias onto the same entity -- they should be disambiguated
    /// rather than corrupting each other's data.
    #[test]
    fn duplicate_keys_are_disambiguated_not_aliased() {
        let mut world = World::new();
        let parent_entity = world.spawn_empty().id();
        let parent = ParentWidget(parent_entity);
        let mut mapper = WidgetMapper::new();

        let keys = ["dup", "dup", "dup"];
        let entities = reconcile(&mut world, &mut mapper, parent, &keys);

        assert_eq!(entities.len(), 3);
        assert_ne!(entities[0], entities[1]);
        assert_ne!(entities[1], entities[2]);
        assert_ne!(entities[0], entities[2]);
    }

    /// A portaled child's real `ChildOf` must point at its portal target, not the widget that
    /// declared it -- and it must be excluded from the declaring parent's own `Children`
    /// (`replace_related`), while its non-portaled siblings are unaffected.
    #[test]
    fn portaled_child_is_physically_parented_to_its_target_not_the_declaring_parent() {
        let mut world = World::new();
        let parent_entity = world.spawn_empty().id();
        let overlay_entity = world.spawn_empty().id();
        let parent = ParentWidget(parent_entity);
        let mut mapper = WidgetMapper::new();

        let entities = reconcile_with_portals(
            &mut world,
            &mut mapper,
            parent,
            &[
                ("sibling_a", None),
                ("portaled", Some(Portal(Some(overlay_entity)))),
                ("sibling_b", None),
            ],
        );
        let portaled = entities[1];

        assert_eq!(
            world.entity(portaled).get::<ChildOf>().unwrap().parent(),
            overlay_entity,
            "a portaled child's real ChildOf must point at its portal target"
        );
        assert_eq!(
            world.entity(portaled).get::<LogicalParent>().unwrap().0,
            parent_entity,
            "a portaled child's LogicalParent must be the widget that declared it"
        );

        let siblings = world
            .entity(parent_entity)
            .get::<Children>()
            .expect("declaring parent should still have its non-portaled children")
            .iter()
            .collect::<Vec<_>>();
        assert_eq!(
            siblings,
            vec![entities[0], entities[2]],
            "the declaring parent's own Children must contain only the non-portaled siblings, \
             in declared order, excluding the portaled entity entirely"
        );
    }

    /// A `.portal()` call with no explicit target (`Portal(None)`) must default to whatever
    /// entity the `OverlayRoot` resource currently points at.
    #[test]
    fn portal_with_no_explicit_target_defaults_to_overlay_root() {
        let mut world = World::new();
        let parent_entity = world.spawn_empty().id();
        let overlay_root_entity = world.spawn_empty().id();
        world.insert_resource(OverlayRoot(overlay_root_entity));
        let parent = ParentWidget(parent_entity);
        let mut mapper = WidgetMapper::new();

        let entities = reconcile_with_portals(
            &mut world,
            &mut mapper,
            parent,
            &[("portaled", Some(Portal(None)))],
        );

        assert_eq!(
            world.entity(entities[0]).get::<ChildOf>().unwrap().parent(),
            overlay_root_entity,
            "Portal(None) must default to the OverlayRoot resource's entity"
        );
    }

    /// A portaled entity's identity, `ChildOf` target, and `LogicalParent` must all survive
    /// unchanged across re-renders that reorder/add/remove its *siblings* -- reconciliation by
    /// key must reuse the same entity rather than despawning and respawning it, and its
    /// `ChildOf`/`LogicalParent` (set once at spawn) must never be touched again.
    #[test]
    fn portal_target_and_identity_survive_sibling_reordering_across_renders() {
        let mut world = World::new();
        let parent_entity = world.spawn_empty().id();
        let overlay_entity = world.spawn_empty().id();
        let parent = ParentWidget(parent_entity);
        let mut mapper = WidgetMapper::new();

        let first_pass = reconcile_with_portals(
            &mut world,
            &mut mapper,
            parent,
            &[
                ("sibling_a", None),
                ("portaled", Some(Portal(Some(overlay_entity)))),
            ],
        );

        // Second render: a new sibling is inserted *before* the portaled entity, and the
        // portaled entity is declared again with the exact same key.
        let second_pass = reconcile_with_portals(
            &mut world,
            &mut mapper,
            parent,
            &[
                ("sibling_new", None),
                ("sibling_a", None),
                ("portaled", Some(Portal(Some(overlay_entity)))),
            ],
        );

        assert_eq!(
            second_pass[2], first_pass[1],
            "the portaled entity's identity must be reused across renders, keyed reconciliation \
             is unaffected by portaling"
        );
        assert_eq!(
            world
                .entity(second_pass[2])
                .get::<ChildOf>()
                .unwrap()
                .parent(),
            overlay_entity,
            "the portaled entity's ChildOf must still point at its target after a re-render \
             that changed its siblings"
        );
        assert_eq!(
            world
                .entity(second_pass[2])
                .get::<LogicalParent>()
                .unwrap()
                .0,
            parent_entity,
            "LogicalParent must remain the declaring parent after a re-render"
        );
    }

    /// A portaled entity must still be despawned once its key genuinely disappears from the
    /// declared list -- being physically parented elsewhere must not exempt it from normal
    /// disappear-and-despawn cleanup.
    #[test]
    fn portaled_child_is_despawned_once_its_key_disappears() {
        let mut world = World::new();
        let parent_entity = world.spawn_empty().id();
        let overlay_entity = world.spawn_empty().id();
        let parent = ParentWidget(parent_entity);
        let mut mapper = WidgetMapper::new();

        let first_pass = reconcile_with_portals(
            &mut world,
            &mut mapper,
            parent,
            &[("portaled", Some(Portal(Some(overlay_entity))))],
        );

        reconcile_with_portals(&mut world, &mut mapper, parent, &[]);

        assert!(
            world.get_entity(first_pass[0]).is_err(),
            "a portaled entity must still be despawned once its key disappears, same as any \
             non-portaled entity"
        );
    }

    /// Regression test: a portaled entity must still be despawned when its logical *ancestor*
    /// (not the widget that directly declared it) is removed wholesale -- e.g. navigating away
    /// in a UI whose currently-shown page happens to contain a widget with a portaled child
    /// (`Modal`/`Drawer`/`WoodpeckerWindow`). The declaring widget itself never gets a chance to
    /// re-render and notice its own portaled child's key disappeared (it's being despawned,
    /// not re-rendered), so `finish_reconciliation`'s recursive cleanup must reach into it via
    /// `WidgetMapper`'s own bookkeeping rather than relying solely on Bevy's `ChildOf` cascade,
    /// which never reaches a portaled entity (physically parented elsewhere) at all.
    #[test]
    fn portaled_grandchild_is_despawned_when_its_ancestor_is_removed_wholesale() {
        let mut world = World::new();
        let overlay_entity = world.spawn_empty().id();
        let grandparent_entity = world.spawn_empty().id();
        let grandparent = ParentWidget(grandparent_entity);
        let mut mapper = WidgetMapper::new();

        let middle_entity = reconcile(&mut world, &mut mapper, grandparent, &["middle"])[0];

        let portaled_entity = reconcile_with_portals(
            &mut world,
            &mut mapper,
            ParentWidget(middle_entity),
            &[("portaled", Some(Portal(Some(overlay_entity))))],
        )[0];

        reconcile(&mut world, &mut mapper, grandparent, &[]);

        assert!(
            world.get_entity(middle_entity).is_err(),
            "the removed ancestor itself must be despawned"
        );
        assert!(
            world.get_entity(portaled_entity).is_err(),
            "a portaled entity must still be despawned when its logical ancestor is removed \
             wholesale, not just when the widget that directly declared it drops its key"
        );
    }

    /// Regression test: a portal *target* (e.g. `OverlayRoot`) is itself just another widget
    /// with its own `WidgetChildren` and its own, entirely unrelated reconciliation passes.
    /// Its own `finish_reconciliation` call must not sever a child that was physically
    /// attached to it by a *different* widget's portal -- that child is invisible to the
    /// target's own `pending`/`by_key` bookkeeping, so a naive `replace_related` using only
    /// that bookkeeping would silently un-parent it the moment the target reconciles its own
    /// (unrelated, possibly zero) declared children.
    #[test]
    fn portal_target_reconciling_its_own_unrelated_children_does_not_sever_a_portaled_child() {
        let mut world = World::new();
        let overlay_root_entity = world.spawn_empty().id();
        world.insert_resource(OverlayRoot(overlay_root_entity));
        let overlay_root = ParentWidget(overlay_root_entity);

        let declaring_parent_entity = world.spawn_empty().id();
        let declaring_parent = ParentWidget(declaring_parent_entity);
        let mut mapper = WidgetMapper::new();

        // A completely different widget portals a child into `overlay_root_entity`.
        let portaled = reconcile_with_portals(
            &mut world,
            &mut mapper,
            declaring_parent,
            &[("portaled", Some(Portal(None)))],
        )[0];
        assert_eq!(
            world.entity(portaled).get::<ChildOf>().unwrap().parent(),
            overlay_root_entity
        );

        // `overlay_root_entity`'s own widget now reconciles *its own* declared children --
        // zero of them, in this case (mirrors a portal target whose own render never calls
        // `.add()` on its own `WidgetChildren`).
        reconcile(&mut world, &mut mapper, overlay_root, &[]);

        assert_eq!(
            world.entity(portaled).get::<ChildOf>().unwrap().parent(),
            overlay_root_entity,
            "the portal target's own (unrelated) reconciliation pass must not sever a child \
             that was portaled into it by a different widget"
        );
        assert!(
            world
                .entity(overlay_root_entity)
                .get::<Children>()
                .unwrap()
                .iter()
                .any(|c| c == portaled),
            "the portaled child must still appear in the target's real bevy Children"
        );
    }

    use crate::context::Widget;

    #[derive(Component, Default)]
    struct TestKeyedWidget;
    impl Widget for TestKeyedWidget {}

    /// Regression test for a real bug: `get_or_insert_entity_world`'s stored key is
    /// `"{widget_name}-{child_key}"`, where `widget_name` is `T::get_name()` -- the full
    /// `std::any::type_name`, not a bare type name. A caller that calls
    /// [`WidgetMapper::get_child`] directly with just the bare `.add_key(...)` string (as
    /// `src/widgets/window.rs`, `tooltip.rs`, and `popover.rs` all originally did) silently
    /// never finds the entity -- `get_child` just returns `None` forever, since the key it's
    /// searching for never matches anything stored. [`WidgetMapper::get_keyed_child`] exists
    /// specifically to make this mismatch impossible by construction.
    #[test]
    fn get_keyed_child_finds_what_get_or_insert_entity_world_actually_stored() {
        let mut world = World::new();
        let parent_entity = world.spawn_empty().id();
        let parent = ParentWidget(parent_entity);
        let mut mapper = WidgetMapper::new();

        mapper.start_reconciliation(parent);
        let entity = mapper.get_or_insert_entity_world(
            &mut world,
            TestKeyedWidget::get_name(),
            parent,
            Some("foo".to_string()),
            None,
        );
        let mut observer_cache = ObserverCache::default();
        mapper.finish_reconciliation(&mut world, &mut observer_cache, parent);

        assert_eq!(
            mapper.get_keyed_child::<TestKeyedWidget>(parent, "foo"),
            Some(entity),
            "get_keyed_child must reconstruct the same composite key \
             get_or_insert_entity_world stored and find the entity"
        );
        assert_eq!(
            mapper.get_child(parent, "foo"),
            None,
            "the bare key alone (without the widget-type prefix) must NOT resolve -- this \
             pins down the exact footgun get_keyed_child exists to avoid"
        );
    }
}
