use crate::{layout::system::WidgetLayout, picking_backend::MouseWheelScroll, prelude::*};
use bevy::prelude::*;

/// One node in a [`TreeView`]'s tree, given as owned data -- the same
/// "declare-the-whole-shape-every-render" pattern [`Table::rows`] uses, rather than a widget
/// tree the caller builds by hand.
#[derive(Reflect, Clone, PartialEq, Debug, Default)]
pub struct TreeNode {
    /// A stable identifier, unique across the whole tree -- used for selection, expand/collapse
    /// state, and the `TreeNodeSelected`/`TreeNodeToggled` events.
    pub key: String,
    /// The row's display text.
    pub label: String,
    /// Nested nodes. An empty `Vec` means this row has no disclosure triangle.
    pub children: Vec<TreeNode>,
}

/// Fired when a row is selected (click, or Enter/Space with the row active via keyboard).
#[derive(Debug, Clone, Reflect)]
pub struct TreeNodeSelected {
    /// The selected node's key.
    pub key: String,
}

/// Fired when a row's expanded/collapsed state toggles (click on its disclosure triangle, or
/// Left/Right with the row active via keyboard).
#[derive(Debug, Clone, Reflect)]
pub struct TreeNodeToggled {
    /// The toggled node's key.
    pub key: String,
    /// Its new expanded state.
    pub expanded: bool,
}

/// A collection of styles for [`TreeView`].
#[derive(Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct TreeViewStyles {
    /// Height of a single row, in logical pixels.
    pub row_height: f32,
    /// Row label text color.
    pub label_color: Color,
    /// Disclosure triangle color.
    pub disclosure_color: Color,
    /// Background of the selected row.
    pub selected_background: Color,
    /// Background of the row under the pointer (when not the selected row).
    pub hovered_background: Color,
    /// Border drawn around the roving keyboard cursor's row (`TreeViewState::active_index`),
    /// when that row isn't also the selected one -- the only on-screen indication that
    /// Up/Down/Left/Right are doing anything, since moving the active row otherwise changes
    /// nothing visible until Enter/Space actually selects it.
    pub active_border_color: Color,
}

impl Default for TreeViewStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for TreeViewStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            row_height: theme.control_height,
            label_color: theme.text,
            disclosure_color: theme.text.with_alpha(0.6),
            selected_background: theme.primary.with_alpha(0.3),
            hovered_background: theme.background_light,
            active_border_color: theme.primary,
        }
    }
}

/// Self-managed state: which nodes are expanded, which is selected, and which row the roving
/// keyboard cursor currently sits on. Seeded from [`TreeView::selected_key`] on mount and then
/// owned internally from then on -- the same "initial prop, then state takes over" pattern
/// `RadioGroup`/`Dropdown` use.
#[derive(Component, Debug, Clone, PartialEq, Reflect, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
struct TreeViewState {
    expanded: Vec<String>,
    selected_key: Option<String>,
    active_index: usize,
    /// Keyed rather than indexed, deliberately: a row's *entity* is reused across renders (see
    /// `children.add_key(key)` below), so its hover observer is only ever attached once and
    /// keeps whatever it originally captured for the lifetime of that entity. An index would
    /// go stale the moment an earlier row's expand/collapse shifts this row to a different
    /// position in the flattened list -- a key never does, since it's the row's own stable
    /// identity, not its transient position.
    hovered_key: Option<String>,
    /// The last [`TreeView::force_expand_ancestors_of`] value already acted on -- a one-shot
    /// trigger, not a standing constraint: expanding a newly-picked node's ancestors once
    /// shouldn't re-force them open again if the user manually collapses one afterwards.
    last_forced_expand: Option<String>,
}

impl TreeViewState {
    fn is_expanded(&self, key: &str) -> bool {
        self.expanded.iter().any(|k| k == key)
    }

    fn set_expanded(&mut self, key: &str, expanded: bool) {
        let currently = self.is_expanded(key);
        if currently == expanded {
            return;
        }
        if expanded {
            self.expanded.push(key.to_string());
        } else {
            self.expanded.retain(|k| k != key);
        }
    }
}

/// A single flattened, currently-visible row -- a depth-first walk of [`TreeView::nodes`] that
/// skips collapsed subtrees entirely. Flattening up front is what makes keyboard navigation a
/// simple linear index instead of a tree walk on every keypress, and is also what lets
/// [`TreeView::virtualized`] plug straight into
/// [`compute_virtual_window`](crate::widgets::virtualization::compute_virtual_window).
struct FlatRow<'a> {
    node: &'a TreeNode,
    depth: usize,
}

fn flatten_visible<'a>(
    nodes: &'a [TreeNode],
    depth: usize,
    state: &TreeViewState,
    out: &mut Vec<FlatRow<'a>>,
) {
    for node in nodes {
        out.push(FlatRow { node, depth });
        if !node.children.is_empty() && state.is_expanded(&node.key) {
            flatten_visible(&node.children, depth + 1, state, out);
        }
    }
}

/// Finds `target_key` in `nodes` and returns the keys of every node on the path from a root
/// node down to (but not including) it, root-first -- the set of nodes that must be expanded
/// for `target_key`'s own row to be visible. `None` if `target_key` isn't in the tree at all.
fn find_ancestor_path(nodes: &[TreeNode], target_key: &str) -> Option<Vec<String>> {
    for node in nodes {
        if node.key == target_key {
            return Some(Vec::new());
        }
        if let Some(mut path) = find_ancestor_path(&node.children, target_key) {
            path.insert(0, node.key.clone());
            return Some(path);
        }
    }
    None
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        ..Default::default()
    }
}

/// A hierarchical, expand/collapse tree of labeled rows -- a file browser, a scene graph, a
/// settings outline. For a flat list (no nesting), prefer `Table`/`VirtualList` instead.
///
/// Only the `TreeView` root itself carries `Focusable` (one Tab stop regardless of how many
/// rows are visible -- "roving tabindex"): Up/Down move a roving active-row cursor,
/// Right expands (or descends into an already-expanded row's first child), Left collapses (or
/// ascends to the parent row), Enter/Space selects the active row.
///
/// Not built in this version, flagged rather than silently dropped: drag-to-reorder (needs
/// ancestor-cycle validation and a "drop *between* rows" interaction nothing in the crate does
/// today -- see the roadmap this widget shipped under).
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = default_style(),
    WidgetChildren,
    TreeViewStyles,
    Pickable,
    Focusable,
    TreeViewScrollState
)]
pub struct TreeView {
    /// The root-level nodes of the tree.
    pub nodes: Vec<TreeNode>,
    /// Which node's key is selected, initially -- see [`TreeViewState`]'s doc comment for how
    /// this interacts with later clicks/keyboard selection.
    pub selected_key: Option<String>,
    /// Indent per depth level, in logical pixels. `0.0` (the `Default` value) is treated as
    /// "use the built-in default" rather than literally no indentation, since an unindented
    /// tree reads as a flat list.
    pub indent: f32,
    /// When set to a key present in `nodes`, expands every ancestor of that node so its own
    /// row becomes visible -- a one-shot reaction to a *new* value (e.g. a programmatic
    /// selection from outside, like a "jump to this node" action), not a standing constraint:
    /// changing this to the same value again, or the user manually collapsing an ancestor
    /// afterwards, does not re-force anything open. Leave `None` for normal use; `selected_key`
    /// alone does not expand anything (a click can only select an already-visible row).
    pub force_expand_ancestors_of: Option<String>,
    /// When `true`, only mounts rows visible in this tree's own (bounded) viewport.
    pub virtualized: bool,
}

const DEFAULT_INDENT: f32 = 16.0;
const DISCLOSURE_WIDTH: f32 = 16.0;
const TREE_VIEW_OVERSCAN: usize = 4;
const TREE_VIEW_MAX_OVERSCAN: usize = 64;
const SCROLL_LINE: f32 = 64.0;
const SCROLLBAR_THICKNESS: f32 = 10.0;

#[derive(Component, Default, Clone, PartialEq)]
struct TreeViewScrollState {
    prev_scroll_offset: f32,
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    icon_font: Res<IconFont>,
    layout_query: Query<&WidgetLayout>,
    mut context_query: Query<&mut ScrollContext>,
    mut query: Query<(
        &TreeView,
        &TreeViewStyles,
        &mut WidgetChildren,
        &mut TreeViewScrollState,
    )>,
    mut state_query: Query<&mut TreeViewState>,
) {
    let Ok((tree_view, styles, mut children, mut scroll_state)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        TreeViewState {
            selected_key: tree_view.selected_key.clone(),
            ..Default::default()
        },
    );
    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    if tree_view.force_expand_ancestors_of != state.last_forced_expand {
        if let Some(target_key) = tree_view.force_expand_ancestors_of.as_deref() {
            if let Some(ancestors) = find_ancestor_path(&tree_view.nodes, target_key) {
                for key in ancestors {
                    state.set_expanded(&key, true);
                }
            }
        }
        state.last_forced_expand = tree_view.force_expand_ancestors_of.clone();
    }

    let mut flat = Vec::new();
    flatten_visible(&tree_view.nodes, 0, &state, &mut flat);
    if flat.is_empty() {
        state.active_index = 0;
    } else if state.active_index >= flat.len() {
        state.active_index = flat.len() - 1;
    }

    let indent = if tree_view.indent > 0.0 {
        tree_view.indent
    } else {
        DEFAULT_INDENT
    };
    let current_widget_val = *current_widget;

    *children = WidgetChildren::default();
    children.observe(
        current_widget_val,
        move |trigger: On<WidgetKeyboardButtonEvent>,
              mut commands: Commands,
              tree_query: Query<&TreeView>,
              mut state_query: Query<&mut TreeViewState>| {
            let Ok(tree_view) = tree_query.get(current_widget_val.0) else {
                return;
            };
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };

            let mut flat = Vec::new();
            flatten_visible(&tree_view.nodes, 0, &state, &mut flat);
            if flat.is_empty() {
                return;
            }
            let active_index = state.active_index.min(flat.len() - 1);

            match trigger.code {
                KeyCode::ArrowDown => {
                    state.active_index = (active_index + 1).min(flat.len() - 1);
                }
                KeyCode::ArrowUp => {
                    state.active_index = active_index.saturating_sub(1);
                }
                KeyCode::ArrowRight => {
                    let node = flat[active_index].node;
                    if !node.children.is_empty() {
                        if state.is_expanded(&node.key) {
                            if active_index + 1 < flat.len() {
                                state.active_index = active_index + 1;
                            }
                        } else {
                            let key = node.key.clone();
                            state.set_expanded(&key, true);
                            commands.trigger(Change {
                                target: current_widget_val.0,
                                data: TreeNodeToggled {
                                    key,
                                    expanded: true,
                                },
                            });
                        }
                    }
                }
                KeyCode::ArrowLeft => {
                    let depth = flat[active_index].depth;
                    let node = flat[active_index].node;
                    if !node.children.is_empty() && state.is_expanded(&node.key) {
                        let key = node.key.clone();
                        state.set_expanded(&key, false);
                        commands.trigger(Change {
                            target: current_widget_val.0,
                            data: TreeNodeToggled {
                                key,
                                expanded: false,
                            },
                        });
                    } else if depth > 0 {
                        // Ascend to the nearest preceding row one level shallower.
                        if let Some(parent_index) = (0..active_index)
                            .rev()
                            .find(|&i| flat[i].depth == depth - 1)
                        {
                            state.active_index = parent_index;
                        }
                    }
                }
                KeyCode::Enter | KeyCode::Space => {
                    let key = flat[active_index].node.key.clone();
                    state.selected_key = Some(key.clone());
                    commands.trigger(Change {
                        target: current_widget_val.0,
                        data: TreeNodeSelected { key },
                    });
                }
                _ => {}
            }
        },
    );

    if !tree_view.virtualized {
        let mut row_children = WidgetChildren::default();
        for (row_index, row) in flat.iter().enumerate() {
            push_tree_row(
                &mut row_children,
                current_widget_val,
                state_entity,
                &icon_font,
                styles,
                indent,
                &state,
                row_index,
                row,
            );
        }
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Column,
                ..Default::default()
            },
            row_children,
        ));
        children.add_key("rows");
        children.apply(current_widget_val.as_parent());
        return;
    }

    let context_entity =
        hooks.use_own_context(&mut commands, *current_widget, ScrollContext::default());
    let Ok(mut context) = context_query.get_mut(context_entity) else {
        return;
    };

    let own_layout = layout_query.get(**current_widget).ok();
    let viewport_width = own_layout.map(|l| l.width()).unwrap_or(0.0);
    let viewport_height = own_layout.map(|l| l.height()).unwrap_or(0.0);
    let content_extent = flat.len() as f32 * styles.row_height;
    context.scrollbox_width = viewport_width;
    context.scrollbox_height = viewport_height;
    context.content_width = viewport_width;
    context.content_height = content_extent;
    let current_scroll_y = context.scroll_y();
    context.set_scroll_y(current_scroll_y);
    let scroll_offset = (-context.scroll_y()).max(0.0);

    let overscan = dynamic_overscan(
        scroll_state.prev_scroll_offset,
        scroll_offset,
        styles.row_height,
        TREE_VIEW_OVERSCAN,
        TREE_VIEW_MAX_OVERSCAN,
    );
    scroll_state.prev_scroll_offset = scroll_offset;

    let window = compute_virtual_window(
        scroll_offset,
        viewport_height,
        styles.row_height,
        flat.len(),
        overscan,
    );

    let mut row_children = WidgetChildren::default();
    if window.lead_spacer > 0.0 {
        row_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: window.lead_spacer.into(),
                ..Default::default()
            },
        ));
        row_children.add_key("lead_spacer");
    }
    for row_index in window.start..window.end {
        push_tree_row(
            &mut row_children,
            current_widget_val,
            state_entity,
            &icon_font,
            styles,
            indent,
            &state,
            row_index,
            &flat[row_index],
        );
    }
    if window.trail_spacer > 0.0 {
        row_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: window.trail_spacer.into(),
                ..Default::default()
            },
        ));
        row_children.add_key("trail_spacer");
    }

    children.observe(
        current_widget_val,
        move |mut trigger: On<Pointer<MouseWheelScroll>>,
              mut context_query: Query<&mut ScrollContext>| {
            trigger.propagate(false);
            let Ok(mut context) = context_query.get_mut(context_entity) else {
                return;
            };
            let scroll_y = context.scroll_y();
            context.set_scroll_y(scroll_y + trigger.scroll.y * SCROLL_LINE);
        },
    );
    children.add::<Clip>((
        Clip,
        WoodpeckerStyle {
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
            row_children,
        )),
    ));
    children.add_key("content");

    children.add::<ScrollBar>(ScrollBar {
        thickness: SCROLLBAR_THICKNESS,
        ..Default::default()
    });
    children.add_key("scrollbar");

    children.apply(current_widget_val.as_parent());
}

#[allow(clippy::too_many_arguments)]
fn push_tree_row(
    target: &mut WidgetChildren,
    current_widget_val: CurrentWidget,
    state_entity: Entity,
    icon_font: &IconFont,
    styles: &TreeViewStyles,
    indent: f32,
    state: &TreeViewState,
    row_index: usize,
    row: &FlatRow,
) {
    let key = row.node.key.clone();
    let is_selected = state.selected_key.as_deref() == Some(key.as_str());
    let is_hovered = state.hovered_key.as_deref() == Some(key.as_str());
    let is_active = state.active_index == row_index;
    let background = if is_selected {
        styles.selected_background
    } else if is_hovered {
        styles.hovered_background
    } else {
        Color::NONE
    };

    let mut disclosure_children = WidgetChildren::default();
    if !row.node.children.is_empty() {
        let glyph = if state.is_expanded(&key) {
            icons::CARET_DOWN
        } else {
            icons::CARET_RIGHT
        };
        disclosure_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: DISCLOSURE_WIDTH.into(),
                font_size: styles.row_height * 0.5,
                color: styles.disclosure_color,
                font: Some(icon_font.0.id()),
                ..Default::default()
            },
            WidgetRender::Text {
                content: glyph.into(),
            },
            // `should_block_lower: false` -- otherwise this click-blocks the row
            // Element beneath it (and, transitively, `TreeView`'s own root) from also
            // registering the click, which `CurrentFocus::click_focus` needs to see to
            // focus the tree (see the same reasoning on the row's own `Pickable` below).
            // Without a `Pickable` at all here, the glyph wasn't clickable in the first
            // place -- clicks landed on the row's own hit region instead, which only
            // selects, never toggles.
            Pickable {
                should_block_lower: false,
                is_hoverable: true,
            },
        ));
        let toggle_key = key.clone();
        disclosure_children.observe(
            current_widget_val,
            move |mut trigger: On<Pointer<Click>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut TreeViewState>| {
                trigger.propagate(false);
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let expanded = !state.is_expanded(&toggle_key);
                state.set_expanded(&toggle_key, expanded);
                commands.trigger(Change {
                    target: current_widget_val.0,
                    data: TreeNodeToggled {
                        key: toggle_key.clone(),
                        expanded,
                    },
                });
            },
        );
    } else {
        disclosure_children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: DISCLOSURE_WIDTH.into(),
                ..Default::default()
            },
        ));
    }
    disclosure_children.add_key("disclosure");

    let select_key = key.clone();
    let hover_key = key.clone();
    target
        .add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: styles.row_height.into(),
                padding: Edge::all(0.0).left(row.depth as f32 * indent),
                align_items: Some(WidgetAlignItems::Center),
                background_color: background,
                border: if is_active {
                    Edge::all(1.0)
                } else {
                    Edge::all(0.0)
                },
                border_color: styles.active_border_color,
                ..Default::default()
            },
            WidgetRender::Quad,
            // `should_block_lower: false` -- `Pickable::default()`'s `true` would block
            // `TreeView`'s own root (a lower-depth ancestor whose bounding box this row
            // sits inside) from *also* registering the click. `CurrentFocus::click_focus`
            // only focuses a `Focusable` entity when *that* entity's own
            // `PickingInteraction` goes `Pressed` -- since every row visually covers part
            // of the root, blocking meant the root's own interaction never updated, so
            // clicking a row selected it but never actually focused the tree (keyboard nav
            // silently went nowhere).
            Pickable {
                should_block_lower: false,
                is_hoverable: true,
            },
            disclosure_children.with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: styles.row_height * 0.5,
                    color: styles.label_color,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: row.node.label.clone(),
                },
            )),
        ))
        .observe(
            current_widget_val,
            // Deliberately re-flattens and looks up `select_key`'s *current* position
            // rather than capturing `row_index` by value: this row's entity is reused
            // across renders (keyed reconciliation, see `target.add_key(key)` below),
            // so this observer is only ever attached once and would otherwise keep
            // whatever index the row happened to have at first mount, even after an
            // earlier row's expand/collapse shifts every row below it to a new position.
            move |_: On<Pointer<Click>>,
                  mut commands: Commands,
                  tree_query: Query<&TreeView>,
                  mut state_query: Query<&mut TreeViewState>| {
                let Ok(tree_view) = tree_query.get(current_widget_val.0) else {
                    return;
                };
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let mut flat = Vec::new();
                flatten_visible(&tree_view.nodes, 0, &state, &mut flat);
                if let Some(current_index) =
                    flat.iter().position(|row| row.node.key == select_key)
                {
                    state.active_index = current_index;
                }
                state.selected_key = Some(select_key.clone());
                commands.trigger(Change {
                    target: current_widget_val.0,
                    data: TreeNodeSelected {
                        key: select_key.clone(),
                    },
                });
            },
        )
        .hover_state(
            current_widget_val,
            state_entity,
            SystemCursorIcon::Pointer,
            move |state: &mut TreeViewState, hovering| {
                if hovering {
                    state.hovered_key = Some(hover_key.clone());
                } else if state.hovered_key.as_deref() == Some(hover_key.as_str()) {
                    state.hovered_key = None;
                }
            },
        );
    target.add_key(key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtualized_window_is_much_smaller_than_a_large_flattened_tree() {
        let tree: Vec<TreeNode> = (0..1_000)
            .map(|i| node(&format!("row{i}"), vec![]))
            .collect();
        let state = TreeViewState::default();
        let mut flat = Vec::new();
        flatten_visible(&tree, 0, &state, &mut flat);
        assert_eq!(flat.len(), 1_000);

        let row_height = TreeViewStyles::from_theme(&Theme::default()).row_height;
        let viewport_height = 400.0;
        let window = compute_virtual_window(
            0.0,
            viewport_height,
            row_height,
            flat.len(),
            TREE_VIEW_OVERSCAN,
        );

        assert!(window.end - window.start < 50);
    }

    fn node(key: &str, children: Vec<TreeNode>) -> TreeNode {
        TreeNode {
            key: key.into(),
            label: key.into(),
            children,
        }
    }

    fn sample_tree() -> Vec<TreeNode> {
        vec![
            node(
                "a",
                vec![
                    node("a.1", vec![]),
                    node("a.2", vec![node("a.2.1", vec![])]),
                ],
            ),
            node("b", vec![]),
        ]
    }

    fn flat_keys(nodes: &[TreeNode], state: &TreeViewState) -> Vec<(String, usize)> {
        let mut flat = Vec::new();
        flatten_visible(nodes, 0, state, &mut flat);
        flat.into_iter()
            .map(|row| (row.node.key.clone(), row.depth))
            .collect()
    }

    #[test]
    fn fully_collapsed_tree_only_shows_root_level_rows() {
        let tree = sample_tree();
        let state = TreeViewState::default();

        assert_eq!(
            flat_keys(&tree, &state),
            vec![("a".into(), 0), ("b".into(), 0)]
        );
    }

    #[test]
    fn expanding_a_node_reveals_only_its_direct_children() {
        let tree = sample_tree();
        let mut state = TreeViewState::default();
        state.set_expanded("a", true);

        assert_eq!(
            flat_keys(&tree, &state),
            vec![
                ("a".into(), 0),
                ("a.1".into(), 1),
                ("a.2".into(), 1),
                ("b".into(), 0),
            ]
        );
    }

    #[test]
    fn expanding_nested_nodes_reveals_the_full_visible_path() {
        let tree = sample_tree();
        let mut state = TreeViewState::default();
        state.set_expanded("a", true);
        state.set_expanded("a.2", true);

        assert_eq!(
            flat_keys(&tree, &state),
            vec![
                ("a".into(), 0),
                ("a.1".into(), 1),
                ("a.2".into(), 1),
                ("a.2.1".into(), 2),
                ("b".into(), 0),
            ]
        );
    }

    #[test]
    fn collapsing_a_node_hides_its_expanded_descendants_too() {
        let tree = sample_tree();
        let mut state = TreeViewState::default();
        state.set_expanded("a", true);
        state.set_expanded("a.2", true);
        state.set_expanded("a", false);

        // Collapsing "a" hides "a.2" (and therefore "a.2.1"), even though "a.2" itself is
        // still individually marked expanded -- collapsing a subtree doesn't need to walk
        // down and clear its descendants' expanded flags, since they're simply unreachable
        // from the flattened walk while an ancestor is collapsed.
        assert_eq!(
            flat_keys(&tree, &state),
            vec![("a".into(), 0), ("b".into(), 0)]
        );
    }

    #[test]
    fn set_expanded_is_idempotent() {
        let mut state = TreeViewState::default();
        state.set_expanded("a", true);
        state.set_expanded("a", true);

        assert_eq!(state.expanded, vec!["a".to_string()]);
    }

    #[test]
    fn find_ancestor_path_returns_root_to_leaf_order_excluding_the_target_itself() {
        let tree = sample_tree();
        assert_eq!(
            find_ancestor_path(&tree, "a.2.1"),
            Some(vec!["a".to_string(), "a.2".to_string()])
        );
    }

    #[test]
    fn find_ancestor_path_of_a_root_node_is_empty() {
        let tree = sample_tree();
        assert_eq!(find_ancestor_path(&tree, "b"), Some(vec![]));
    }

    #[test]
    fn find_ancestor_path_of_an_unknown_key_is_none() {
        let tree = sample_tree();
        assert_eq!(find_ancestor_path(&tree, "nope"), None);
    }

    #[test]
    fn force_expand_ancestors_of_is_a_one_shot_trigger() {
        let tree = sample_tree();
        let mut state = TreeViewState::default();

        let target = Some("a.2.1".to_string());
        if target != state.last_forced_expand {
            if let Some(ancestors) = find_ancestor_path(&tree, target.as_deref().unwrap()) {
                for key in ancestors {
                    state.set_expanded(&key, true);
                }
            }
            state.last_forced_expand = target.clone();
        }
        assert!(state.is_expanded("a"));
        assert!(state.is_expanded("a.2"));

        // Manually collapse "a" -- a second render with the *same* target must not re-force
        // it back open.
        state.set_expanded("a", false);
        if target != state.last_forced_expand {
            panic!("should not re-trigger for an unchanged target");
        }
        assert!(!state.is_expanded("a"));
    }
}
