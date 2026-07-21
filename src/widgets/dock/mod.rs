mod drop_zone;
mod floating;
mod panels;
mod tab_bar;
mod tree;
mod tree_render;

pub use floating::{DockFloating, FloatingDockWindow, FloatingWindowId};
pub use panels::{DockPanelRegistry, DockPanels, PanelDef, PanelFactory};
pub use tab_bar::DockTabBarStyles;
pub use tree::{DockAxis, DockEdge, DockNode, DockTree, NodePath, PanelId};

use bevy::prelude::*;

use crate::prelude::*;

use floating::render_floating_windows;
use tree_render::{render_dock_node, DockRenderCtx};

pub(crate) use drop_zone::{DockDropSurface, DockRootDropZone};
pub(crate) use tab_bar::{DockDragActive, DockTabBar, DockTabHeader};
pub(crate) use tree_render::DockSplitDragBase;

/// Styles for [`DockArea`] and the drop-zone highlight [`drop_zone::DockDropSurface`] draws.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct DockStyles {
    /// The translucent fill an edge/center drop zone shows while a drag hovers it.
    pub drop_highlight: Color,
    /// The border drawn around each [`DockNode::Tabs`] group's own content box -- gives
    /// adjacent panels a visible seam instead of blending straight into each other across a
    /// bare splitter.
    pub panel_border: Color,
    /// Corner radius of that same border.
    pub panel_border_radius: f32,
    /// The gap left between a [`DockNode::Split`]'s children (and between the outermost
    /// children and the `DockArea`'s own edge), in logical pixels -- breathing room around each
    /// panel rather than every pane touching its neighbors and the splitter directly.
    pub panel_gap: f32,
    /// Padding applied around a panel's own content, between it and its `Tabs` group's own
    /// border -- a baseline inset every panel gets for free, rather than relying on each
    /// panel's own factory to remember to pad itself.
    pub content_padding: f32,
}

impl Default for DockStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for DockStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            drop_highlight: theme.primary,
            panel_border: theme.border,
            panel_border_radius: 6.0,
            panel_gap: 6.0,
            content_padding: 12.0,
        }
    }
}

fn default_dock_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        ..Default::default()
    }
}

/// A resizable, tabbed, redockable panel layout -- drag a tab to a panel's edge to split, or to
/// its center to join; dropping it anywhere else (nothing valid under the cursor) reverts it
/// back to right where it started, rather than spawning a floating window. `DockFloating`/
/// `FloatingDockWindow` still exist as infrastructure (a panel placed there, by whatever means,
/// still renders and redocks correctly), just nothing in this crate creates one via a plain
/// drag anymore -- changed on explicit feedback that an imprecise drop landing just outside a
/// target read as more surprising than forgiving.
///
/// Unlike [`WoodpeckerWindow`], callers never hand-place [`crate::widgets::Splitter`]/tab nodes
/// themselves: `DockArea` owns its whole tree and renders it recursively (see
/// [`tree_render::render_dock_node`]). Register panels via [`PanelDef`]/[`DockPanels`] --
/// factories are lazy closures, so only the active tab of each group is ever actually built.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq, Clone)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_dock_style(), WidgetChildren, DockStyles)]
pub struct DockArea {
    /// The starting layout, seeded once when this `DockArea` first mounts. Later renders read
    /// back the *live* tree (mutated by drag/redock/close since then) via `hooks.use_own_context`
    /// -- re-declaring a `DockArea` with a different `initial_tree` after the first mount has no
    /// effect, the same "seeded once, then owned" contract every context in this crate follows.
    pub initial_tree: DockTree,
    /// The panels this dock can show. `DockPanels` holds un-`Reflect`-able closures (each
    /// `PanelDef::factory`), so this field is `#[reflect(ignore)]` -- meaning the generic
    /// reflect-based diff this crate's other `DiffableProp` fields get can't see changes to it
    /// at all, only `initial_tree`'s. Resolved into a lookup table fresh every time `DockArea`'s
    /// render *does* run (see `render`'s own body) -- covers the common case of a fixed panel
    /// set declared once at startup alongside a fixed `initial_tree`; swapping `panels` alone
    /// afterward, with `initial_tree` unchanged, won't be picked up until something else also
    /// triggers a re-render. A known, accepted limitation of this pass.
    #[reflect(ignore)]
    pub panels: DockPanels,
}

impl Default for DockArea {
    fn default() -> Self {
        Self {
            initial_tree: DockTree::default(),
            panels: DockPanels::default(),
        }
    }
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    icon_font: Res<IconFont>,
    mut query: Query<(&DockArea, &DockStyles, &mut WidgetChildren)>,
    tree_query: Query<&DockTree>,
    floating_query: Query<&DockFloating>,
    registry_query: Query<&DockPanelRegistry>,
) {
    let Ok((dock_area, styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let tree_entity = hooks.use_own_context(
        &mut commands,
        *current_widget,
        dock_area.initial_tree.clone(),
    );
    let floating_entity =
        hooks.use_own_context(&mut commands, *current_widget, DockFloating::default());
    let registry_entity =
        hooks.use_own_context(&mut commands, *current_widget, DockPanelRegistry::default());
    hooks.use_own_context(&mut commands, *current_widget, DockDragActive::default());
    commands
        .entity(registry_entity)
        .insert(DockPanelRegistry::from_panels(&dock_area.panels));

    let Ok(tree) = tree_query.get(tree_entity) else {
        return;
    };

    let ctx = DockRenderCtx {
        current_widget: *current_widget,
        tree_entity,
        styles: *styles,
    };

    *children = WidgetChildren::default();

    if let Some(root) = &tree.root {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                ..Default::default()
            },
            render_dock_node(root, NodePath::root(), ctx),
        ));
        children.add_key("tree");
    } else {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                align_items: Some(WidgetAlignItems::Center),
                justify_content: Some(WidgetJustifyContent::Center),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: styles.drop_highlight.with_alpha(0.6),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "No panels docked".into(),
                },
            )),
        ));
        children.add_key("empty");
    }

    children.add::<DockRootDropZone>((DockRootDropZone,));
    children.add_key("root_drop_zone");

    if let Ok(floating) = floating_query.get(floating_entity) {
        if !floating.windows.is_empty() {
            if let Ok(registry) = registry_query.get(registry_entity) {
                render_floating_windows(
                    &mut children,
                    floating,
                    registry,
                    floating_entity,
                    &icon_font,
                    ctx,
                );
            }
        }
    }

    children.apply(current_widget.as_parent());
}
