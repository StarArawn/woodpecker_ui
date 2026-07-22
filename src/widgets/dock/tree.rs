use crate::prelude::*;
use bevy::prelude::*;

/// A stable key identifying one dockable panel -- deliberately a `String`, not an `Entity`:
/// entities aren't stable across app restarts, and this whole module is kept serialization-
/// shaped (plain enums/structs, string-keyed) even though nothing derives `Serialize` yet, so a
/// future `dock_persistence` feature can save/load a [`DockTree`] without reworking this type.
#[derive(Reflect, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct PanelId(pub String);

impl PanelId {
    /// Wraps `id` as a `PanelId`.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&str> for PanelId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl From<String> for PanelId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// The direction a [`DockNode::Split`]'s children lay out along.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DockAxis {
    /// Children lay out left to right.
    Row,
    /// Children lay out top to bottom.
    Column,
}

/// Where a panel was dropped relative to an existing dock node -- `Center` joins the target's
/// tab group; the other four wrap the target in a new [`DockNode::Split`] on that side.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DockEdge {
    /// Split with the new panel on the left.
    Left,
    /// Split with the new panel on the right.
    Right,
    /// Split with the new panel on top.
    Top,
    /// Split with the new panel on the bottom.
    Bottom,
    /// Join the target's own tab group instead of splitting.
    Center,
}

/// One node of a [`DockTree`]: either a resizable split (each child paired with its own share
/// of the available space, kept normalized to sum to `1.0`) or a tab group of panels with one
/// active at a time.
#[derive(Reflect, Clone, PartialEq, Debug)]
pub enum DockNode {
    /// A resizable split.
    Split {
        /// The direction the children lay out along.
        axis: DockAxis,
        /// Each child paired with its own share (summing to `1.0`) of the split's space.
        children: Vec<(DockNode, f32)>,
    },
    /// A tab group of panels, one active at a time.
    Tabs {
        /// The panels in this group, in tab order.
        panels: Vec<PanelId>,
        /// The index into `panels` currently shown.
        active: usize,
    },
}

impl DockNode {
    /// A tab group containing `panels`, the first one active.
    pub fn tabs(panels: impl IntoIterator<Item = PanelId>) -> Self {
        Self::Tabs {
            panels: panels.into_iter().collect(),
            active: 0,
        }
    }
}

/// An index chain from a [`DockTree`]'s root, through zero or more [`DockNode::Split`]s, to the
/// node it identifies. An empty path is the root itself.
#[derive(Reflect, Clone, PartialEq, Eq, Default, Debug)]
pub struct NodePath(pub Vec<usize>);

impl NodePath {
    /// The path to the tree's own root.
    pub fn root() -> Self {
        Self(Vec::new())
    }

    /// The path to this path's `index`-th child.
    pub fn child(&self, index: usize) -> Self {
        let mut path = self.0.clone();
        path.push(index);
        Self(path)
    }
}

/// The full split/tab layout tree for one [`super::DockArea`]. Structural mutations (redock,
/// close, resize) all go through this type's own methods rather than ad hoc tree surgery at
/// call sites, so [`Self::simplify`] (collapsing empty/single-child nodes) always runs after
/// anything that could produce one.
#[derive(Component, Reflect, Clone, PartialEq, Default, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct DockTree {
    /// The tree's contents, or `None` when every panel has been removed.
    pub root: Option<DockNode>,
    /// The last [`DockNode::Tabs`] a panel was made active in, via [`Self::set_active`] --
    /// [`super::floating`]'s redock button targets this when the dragged-out panel didn't come
    /// from a recorded location (e.g. it's the first thing ever floated).
    pub last_focused: Option<NodePath>,
}

impl DockTree {
    /// A tree containing a single panel and nothing else.
    pub fn single(panel: PanelId) -> Self {
        Self {
            root: Some(DockNode::tabs([panel])),
            last_focused: Some(NodePath::root()),
        }
    }

    /// The *current* path to the [`DockNode::Tabs`] group containing `panel`, if it's present
    /// anywhere in the tree.
    ///
    /// The tree-wide answer to the exact problem [`Self::last_focused`]'s own staleness fix
    /// addresses for one specific field: a [`NodePath`] is a chain of *indices*, not a stable
    /// id, so any path captured before a [`Self::remove_panel`] call elsewhere in the tree can
    /// point at the wrong node (or nothing) by the time it's used, once that removal's
    /// [`Self::simplify`] finishes renumbering everything downstream of the collapse. A drop
    /// handler that captures "the target tab group's path" at render time and later does
    /// `remove_panel(dragged)` (to detach the dragged panel from its old spot) *before*
    /// `insert_center`/`insert_edge` (to place it at the target) must re-resolve the target via
    /// this method, keyed on a panel it's known to still contain, rather than reuse that
    /// stale-by-construction captured path -- confirmed concretely: dropping a panel onto a
    /// *different* tab group whose removal collapsed an ancestor `Split` silently discarded the
    /// dragged panel entirely, since the captured target path no longer resolved to anything.
    pub fn find_panel_group(&self, panel: &PanelId) -> Option<NodePath> {
        fn search(node: &DockNode, path: NodePath, panel: &PanelId) -> Option<NodePath> {
            match node {
                DockNode::Tabs { panels, .. } => panels.contains(panel).then_some(path),
                DockNode::Split { children, .. } => children
                    .iter()
                    .enumerate()
                    .find_map(|(index, (child, _))| search(child, path.child(index), panel)),
            }
        }
        search(self.root.as_ref()?, NodePath::root(), panel)
    }

    /// The node at `path`, or `None` if `path` doesn't resolve (a stale path, or one indexing
    /// past a `Tabs` leaf, which has no indexable children of its own).
    pub fn node_at(&self, path: &NodePath) -> Option<&DockNode> {
        let mut current = self.root.as_ref()?;
        for &index in &path.0 {
            match current {
                DockNode::Split { children, .. } => current = &children.get(index)?.0,
                DockNode::Tabs { .. } => return None,
            }
        }
        Some(current)
    }

    /// Mutable version of [`Self::node_at`].
    pub fn node_at_mut(&mut self, path: &NodePath) -> Option<&mut DockNode> {
        let mut current = self.root.as_mut()?;
        for &index in &path.0 {
            match current {
                DockNode::Split { children, .. } => current = &mut children.get_mut(index)?.0,
                DockNode::Tabs { .. } => return None,
            }
        }
        Some(current)
    }

    /// Removes `panel` from whichever [`DockNode::Tabs`] contains it, then [`Self::simplify`]s
    /// the tree. Returns whether it was found at all.
    pub fn remove_panel(&mut self, panel: &PanelId) -> bool {
        fn remove_from(node: &mut DockNode, panel: &PanelId) -> bool {
            match node {
                DockNode::Tabs { panels, active } => {
                    let Some(pos) = panels.iter().position(|p| p == panel) else {
                        return false;
                    };
                    panels.remove(pos);
                    if !panels.is_empty() && *active >= panels.len() {
                        *active = panels.len() - 1;
                    }
                    true
                }
                DockNode::Split { children, .. } => children
                    .iter_mut()
                    .any(|(child, _)| remove_from(child, panel)),
            }
        }

        let Some(root) = self.root.as_mut() else {
            return false;
        };
        let removed = remove_from(root, panel);
        if removed {
            self.simplify();
        }
        removed
    }

    /// Joins `panel` into the tab group at `target` (a no-op if `target` isn't a
    /// [`DockNode::Tabs`], e.g. a stale path from before a concurrent structural change), making
    /// it the active tab. If the tree is entirely empty (`root` is `None`) *and* `target` is the
    /// root path, seeds a fresh single-panel tree instead -- `node_at_mut` has nothing to find a
    /// `Tabs` node at otherwise, so without this a drop onto an empty [`super::DockArea`]'s own
    /// always-available root edges/center would silently do nothing.
    pub fn insert_center(&mut self, target: &NodePath, panel: PanelId) {
        if self.root.is_none() && target.0.is_empty() {
            self.root = Some(DockNode::tabs([panel]));
            return;
        }
        if let Some(DockNode::Tabs { panels, active }) = self.node_at_mut(target) {
            if !panels.contains(&panel) {
                panels.push(panel);
                *active = panels.len() - 1;
            }
        }
    }

    /// Wraps the node at `target` in a new [`DockNode::Split`], with a fresh single-panel tab
    /// group for `panel` on the side `edge` names and `ratio` of the new split's space. `edge`
    /// of [`DockEdge::Center`] delegates to [`Self::insert_center`] instead of wrapping anything.
    /// See [`Self::insert_center`]'s own doc comment for the empty-tree/root-target case, which
    /// this shares (there's nothing to put on the *other* side of a split when the tree started
    /// out empty, so it seeds a single-panel tree the same way).
    pub fn insert_edge(&mut self, target: &NodePath, panel: PanelId, edge: DockEdge, ratio: f32) {
        if edge == DockEdge::Center || (self.root.is_none() && target.0.is_empty()) {
            self.insert_center(target, panel);
            return;
        }
        let Some(slot) = self.node_at_mut(target) else {
            return;
        };
        let existing = std::mem::replace(slot, DockNode::tabs([]));
        let new_tabs = DockNode::tabs([panel]);
        let ratio = ratio.clamp(0.05, 0.95);
        let axis = match edge {
            DockEdge::Left | DockEdge::Right => DockAxis::Row,
            DockEdge::Top | DockEdge::Bottom => DockAxis::Column,
            DockEdge::Center => unreachable!("handled above"),
        };
        let children = match edge {
            DockEdge::Left | DockEdge::Top => vec![(new_tabs, ratio), (existing, 1.0 - ratio)],
            DockEdge::Right | DockEdge::Bottom => vec![(existing, 1.0 - ratio), (new_tabs, ratio)],
            DockEdge::Center => unreachable!("handled above"),
        };
        *slot = DockNode::Split { axis, children };
        self.revalidate_last_focused();
    }

    /// Makes `panel` the active tab of the [`DockNode::Tabs`] at `target`, and records `target`
    /// as [`Self::last_focused`].
    pub fn set_active(&mut self, target: &NodePath, panel: &PanelId) {
        if let Some(DockNode::Tabs { panels, active }) = self.node_at_mut(target) {
            if let Some(pos) = panels.iter().position(|p| p == panel) {
                *active = pos;
            }
        }
        self.last_focused = Some(target.clone());
    }

    /// Sets the split ratio between `children[child_index]` and `children[child_index + 1]` of
    /// the [`DockNode::Split`] at `split_path`, given `(base_a, base_b)` -- their ratio pair at
    /// the *start* of the current drag gesture -- shifted by `delta` logical pixels along the
    /// split's own axis, out of a total `axis_extent_px`.
    ///
    /// Takes the base explicitly rather than reading `children[child_index]`'s *current* value,
    /// because a `Splitter`'s own `Change<SplitterChanged>::delta` is always the *total*
    /// displacement since the drag started (re-measured from `SplitterState::drag_start_world`
    /// every `Pointer<Drag>` tick, not accumulated frame to frame -- see that type's own doc
    /// comment), not a per-event increment. Adding it to the tree's already-updated ratio on
    /// every tick compounds the same total delta repeatedly, producing runaway, jumpy resizing;
    /// the fix mirrors `examples/gallery/demos/part4.rs`'s own `SplitterDemo`, which captures
    /// `left_width_at_drag_start` once on `Pointer<DragStart>` and recomputes fresh from that
    /// fixed base on every `Change<SplitterChanged>` rather than incrementing in place.
    pub fn set_split_ratio_from_base(
        &mut self,
        split_path: &NodePath,
        child_index: usize,
        base_a: f32,
        base_b: f32,
        delta: f32,
        axis_extent_px: f32,
    ) {
        if axis_extent_px <= 0.0 {
            return;
        }
        let Some(DockNode::Split { children, .. }) = self.node_at_mut(split_path) else {
            return;
        };
        if child_index + 1 >= children.len() {
            return;
        }

        const MIN_RATIO: f32 = 0.05;
        let delta_fraction = delta / axis_extent_px;
        let pair_total = base_a + base_b;
        let new_a = (base_a + delta_fraction).clamp(MIN_RATIO, pair_total - MIN_RATIO);
        children[child_index].1 = new_a;
        children[child_index + 1].1 = pair_total - new_a;
    }

    /// Collapses [`DockNode::Split`]s down to their one remaining child and drops emptied
    /// [`DockNode::Tabs`] entirely, recursively -- called automatically by [`Self::remove_panel`]
    /// (structural mutation is otherwise the caller's job, but every mutation this crate performs
    /// on its own goes through a method here, so this is never left for a caller to remember).
    ///
    /// Also re-validates [`Self::last_focused`]: collapsing a `Split` down to its one remaining
    /// child shifts every path *through* that point in the tree (an index chain, not a stable
    /// id -- see [`NodePath`]'s own doc comment), which can silently turn an old `last_focused`
    /// into a path that either doesn't resolve at all or, worse, now resolves to some other,
    /// unrelated node after the renumbering. Confirmed concretely: float a panel out of a
    /// two-child root `Split` whose *other* child was the last-focused one -- removing the
    /// floated panel collapses the root down to that other child directly, so the old
    /// (now one level too deep) path stops resolving, and the floating window's own redock
    /// button -- which targets exactly this field -- silently did nothing.
    pub fn simplify(&mut self) {
        self.root = self.root.take().and_then(simplify_node);
        self.revalidate_last_focused();
    }

    /// Clears [`Self::last_focused`] if it no longer resolves to a [`DockNode::Tabs`] -- called
    /// after every structural mutation that can shift path indices (not just [`Self::simplify`];
    /// [`Self::insert_edge`] wrapping an ancestor of `last_focused` in a new `Split` renumbers it
    /// too), so a consumer reading `last_focused` never has to guess whether it's stale -- `Some`
    /// always means a currently-valid path.
    fn revalidate_last_focused(&mut self) {
        if let Some(path) = &self.last_focused {
            if !matches!(self.node_at(path), Some(DockNode::Tabs { .. })) {
                self.last_focused = None;
            }
        }
    }
}

fn simplify_node(node: DockNode) -> Option<DockNode> {
    match node {
        DockNode::Tabs { panels, active } => {
            if panels.is_empty() {
                None
            } else {
                Some(DockNode::Tabs {
                    active: active.min(panels.len() - 1),
                    panels,
                })
            }
        }
        DockNode::Split { axis, children } => {
            let mut simplified: Vec<(DockNode, f32)> = children
                .into_iter()
                .filter_map(|(child, ratio)| simplify_node(child).map(|child| (child, ratio)))
                .collect();

            if simplified.is_empty() {
                None
            } else if simplified.len() == 1 {
                Some(simplified.pop().expect("checked len == 1").0)
            } else {
                let total: f32 = simplified.iter().map(|(_, ratio)| *ratio).sum();
                if total > 0.0 {
                    for (_, ratio) in simplified.iter_mut() {
                        *ratio /= total;
                    }
                }
                Some(DockNode::Split {
                    axis,
                    children: simplified,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel(id: &str) -> PanelId {
        PanelId::from(id)
    }

    #[test]
    fn node_at_walks_a_hand_built_tree() {
        let tree = DockTree {
            root: Some(DockNode::Split {
                axis: DockAxis::Row,
                children: vec![
                    (DockNode::tabs([panel("a")]), 0.5),
                    (
                        DockNode::Split {
                            axis: DockAxis::Column,
                            children: vec![
                                (DockNode::tabs([panel("b")]), 0.5),
                                (DockNode::tabs([panel("c"), panel("d")]), 0.5),
                            ],
                        },
                        0.5,
                    ),
                ],
            }),
            last_focused: None,
        };

        assert_eq!(
            tree.node_at(&NodePath::root().child(0)),
            Some(&DockNode::tabs([panel("a")]))
        );
        assert_eq!(
            tree.node_at(&NodePath::root().child(1).child(1)),
            Some(&DockNode::tabs([panel("c"), panel("d")]))
        );
        assert_eq!(tree.node_at(&NodePath::root().child(5)), None);
        assert_eq!(tree.node_at(&NodePath::root().child(0).child(0)), None);
    }

    #[test]
    fn insert_edge_wraps_target_with_new_panel_on_the_named_side() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_edge(&NodePath::root(), panel("b"), DockEdge::Right, 0.3);

        let Some(DockNode::Split { axis, children }) = &tree.root else {
            panic!("expected a Split after insert_edge");
        };
        assert_eq!(*axis, DockAxis::Row);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].0, DockNode::tabs([panel("a")]));
        assert!((children[0].1 - 0.7).abs() < f32::EPSILON);
        assert_eq!(children[1].0, DockNode::tabs([panel("b")]));
        assert!((children[1].1 - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn insert_edge_left_puts_new_panel_first() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_edge(&NodePath::root(), panel("b"), DockEdge::Top, 0.4);

        let Some(DockNode::Split { axis, children }) = &tree.root else {
            panic!("expected a Split after insert_edge");
        };
        assert_eq!(*axis, DockAxis::Column);
        assert_eq!(children[0].0, DockNode::tabs([panel("b")]));
        assert_eq!(children[1].0, DockNode::tabs([panel("a")]));
    }

    #[test]
    fn insert_center_joins_the_target_tab_group_and_activates_it() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_center(&NodePath::root(), panel("b"));

        assert_eq!(
            tree.root,
            Some(DockNode::Tabs {
                panels: vec![panel("a"), panel("b")],
                active: 1,
            })
        );
    }

    #[test]
    fn insert_center_on_an_empty_tree_seeds_a_single_panel_root() {
        let mut tree = DockTree::default();
        assert_eq!(tree.root, None);

        tree.insert_center(&NodePath::root(), panel("a"));
        assert_eq!(tree.root, Some(DockNode::tabs([panel("a")])));
    }

    #[test]
    fn insert_edge_on_an_empty_tree_seeds_a_single_panel_root_regardless_of_edge() {
        let mut tree = DockTree::default();

        tree.insert_edge(&NodePath::root(), panel("a"), DockEdge::Left, 0.4);
        assert_eq!(
            tree.root,
            Some(DockNode::tabs([panel("a")])),
            "nothing to split against yet -- an empty tree just seeds a single-panel root, same \
             as insert_center"
        );
    }

    #[test]
    fn remove_panel_from_a_multi_tab_group_keeps_the_group() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_center(&NodePath::root(), panel("b"));

        assert!(tree.remove_panel(&panel("a")));
        assert_eq!(
            tree.root,
            Some(DockNode::Tabs {
                panels: vec![panel("b")],
                active: 0,
            })
        );
        assert!(!tree.remove_panel(&panel("a")), "already removed");
    }

    #[test]
    fn remove_panel_collapses_a_split_down_to_its_one_remaining_child() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_edge(&NodePath::root(), panel("b"), DockEdge::Right, 0.5);
        assert!(matches!(tree.root, Some(DockNode::Split { .. })));

        assert!(tree.remove_panel(&panel("b")));
        assert_eq!(tree.root, Some(DockNode::tabs([panel("a")])));
    }

    #[test]
    fn remove_last_panel_empties_the_tree() {
        let mut tree = DockTree::single(panel("a"));
        assert!(tree.remove_panel(&panel("a")));
        assert_eq!(tree.root, None);
    }

    #[test]
    fn remove_panel_collapses_a_deeply_nested_split_chain() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_edge(&NodePath::root(), panel("b"), DockEdge::Right, 0.5);
        tree.insert_edge(
            &NodePath::root().child(1),
            panel("c"),
            DockEdge::Bottom,
            0.5,
        );

        assert!(tree.remove_panel(&panel("c")));
        assert_eq!(
            tree.root,
            Some(DockNode::Split {
                axis: DockAxis::Row,
                children: vec![
                    (DockNode::tabs([panel("a")]), 0.5),
                    (DockNode::tabs([panel("b")]), 0.5),
                ],
            })
        );

        assert!(tree.remove_panel(&panel("b")));
        assert_eq!(tree.root, Some(DockNode::tabs([panel("a")])));
    }

    #[test]
    fn set_split_ratio_from_base_shifts_ratio_between_the_two_named_children_and_clamps() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_edge(&NodePath::root(), panel("b"), DockEdge::Right, 0.5);

        tree.set_split_ratio_from_base(&NodePath::root(), 0, 0.5, 0.5, 100.0, 400.0);
        let Some(DockNode::Split { children, .. }) = &tree.root else {
            panic!("expected a Split");
        };
        assert!((children[0].1 - 0.75).abs() < 1e-5);
        assert!((children[1].1 - 0.25).abs() < 1e-5);

        tree.set_split_ratio_from_base(&NodePath::root(), 0, 0.5, 0.5, 10_000.0, 400.0);
        let Some(DockNode::Split { children, .. }) = &tree.root else {
            panic!("expected a Split");
        };
        assert!(children[0].1 >= 0.05 && children[0].1 <= 0.95);
        assert!((children[0].1 + children[1].1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn set_active_updates_active_index_and_last_focused() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_center(&NodePath::root(), panel("b"));

        tree.set_active(&NodePath::root(), &panel("a"));
        assert_eq!(
            tree.root,
            Some(DockNode::Tabs {
                panels: vec![panel("a"), panel("b")],
                active: 0,
            })
        );
        assert_eq!(tree.last_focused, Some(NodePath::root()));
    }

    #[test]
    fn removing_a_panel_that_collapses_the_tree_invalidates_a_stale_last_focused() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_edge(&NodePath::root(), panel("b"), DockEdge::Right, 0.5);
        tree.set_active(&NodePath::root().child(1), &panel("b"));
        assert_eq!(tree.last_focused, Some(NodePath::root().child(1)));

        assert!(tree.remove_panel(&panel("a")));
        assert_eq!(tree.root, Some(DockNode::tabs([panel("b")])));
        assert_eq!(tree.last_focused, None);
    }

    #[test]
    fn last_focused_survives_a_mutation_that_does_not_affect_its_own_path() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_edge(&NodePath::root(), panel("b"), DockEdge::Right, 0.5);
        tree.set_active(&NodePath::root().child(0), &panel("a"));
        assert_eq!(tree.last_focused, Some(NodePath::root().child(0)));

        tree.insert_edge(
            &NodePath::root().child(1),
            panel("c"),
            DockEdge::Bottom,
            0.5,
        );

        assert_eq!(tree.last_focused, Some(NodePath::root().child(0)));
        assert_eq!(
            tree.node_at(&NodePath::root().child(0)),
            Some(&DockNode::tabs([panel("a")]))
        );
    }

    #[test]
    fn find_panel_group_locates_the_tabs_node_containing_a_panel() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_edge(&NodePath::root(), panel("b"), DockEdge::Right, 0.5);
        tree.insert_edge(
            &NodePath::root().child(1),
            panel("c"),
            DockEdge::Bottom,
            0.5,
        );

        assert_eq!(
            tree.find_panel_group(&panel("a")),
            Some(NodePath::root().child(0))
        );
        assert_eq!(
            tree.find_panel_group(&panel("b")),
            Some(NodePath::root().child(1).child(0))
        );
        assert_eq!(tree.find_panel_group(&panel("nonexistent")), None);
    }

    #[test]
    fn find_panel_group_re_resolves_correctly_after_a_sibling_removal_collapses_the_tree() {
        let mut tree = DockTree::single(panel("a"));
        tree.insert_edge(&NodePath::root(), panel("b"), DockEdge::Right, 0.5);
        let stale_path_before_removal = tree.find_panel_group(&panel("b"));
        assert_eq!(stale_path_before_removal, Some(NodePath::root().child(1)));

        assert!(tree.remove_panel(&panel("a")));
        assert_eq!(tree.root, Some(DockNode::tabs([panel("b")])));

        assert_eq!(tree.node_at(&stale_path_before_removal.unwrap()), None);
        assert_eq!(tree.find_panel_group(&panel("b")), Some(NodePath::root()));
    }
}
