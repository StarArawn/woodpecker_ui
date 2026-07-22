use std::sync::Arc;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::prelude::WidgetChildren;

use super::tree::PanelId;

/// Builds one panel's content on demand. A `DockArea` only ever calls the factory for the
/// *active* tab of each tab group it renders -- panels sitting behind another tab, or in a
/// still-collapsed part of the tree, never pay to build their content at all.
pub type PanelFactory = Arc<dyn Fn(PanelId) -> WidgetChildren + Send + Sync>;

/// One registered panel: its stable identity, display title, whether it can be closed from its
/// tab header, and how to build its content.
#[derive(Clone)]
pub struct PanelDef {
    /// This panel's stable identity within its `DockArea`.
    pub id: PanelId,
    /// The text shown on this panel's tab header.
    pub title: String,
    /// Whether this panel's tab header shows a close button.
    pub closable: bool,
    /// Builds this panel's content, called only when it's the active tab of its group.
    pub factory: PanelFactory,
}

impl PanelDef {
    /// A closable panel with `factory` building its content.
    pub fn new(
        id: impl Into<PanelId>,
        title: impl Into<String>,
        factory: impl Fn(PanelId) -> WidgetChildren + Send + Sync + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            closable: true,
            factory: Arc::new(factory),
        }
    }

    /// Hides this panel's tab close button -- for panels a layout shouldn't lose entirely
    /// (e.g. a primary editor pane), as opposed to auxiliary panels the user can dismiss.
    pub fn not_closable(mut self) -> Self {
        self.closable = false;
        self
    }
}

/// The full set of panels a [`super::DockArea`] can show, keyed by [`PanelId`] once resolved
/// into a [`DockPanelRegistry`]. `Arc`-wrapped and compared by pointer identity (mirrors
/// `TableRows`' identical reasoning) since it holds un-`Reflect`-able closures and only ever
/// needs to be *replaced* wholesale, not diffed field by field.
#[derive(Clone)]
pub struct DockPanels(pub Arc<Vec<PanelDef>>);

impl DockPanels {
    /// Wraps `panels` for use as `DockArea::panels`.
    pub fn new(panels: Vec<PanelDef>) -> Self {
        Self(Arc::new(panels))
    }
}

impl Default for DockPanels {
    fn default() -> Self {
        Self(Arc::new(Vec::new()))
    }
}

impl PartialEq for DockPanels {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl From<Vec<PanelDef>> for DockPanels {
    fn from(panels: Vec<PanelDef>) -> Self {
        Self::new(panels)
    }
}

/// [`DockPanels`] resolved into a lookup table by [`PanelId`], built once when a `DockArea`
/// mounts. Not `Reflect`/`DiffableProp` -- it's read-only for the rest of the `DockArea`'s life
/// (redocking rearranges [`super::DockTree`], never this), so there's nothing to diff.
#[derive(Component, Clone, Default)]
pub struct DockPanelRegistry(HashMap<String, PanelDef>);

impl DockPanelRegistry {
    /// Resolves `panels` into a lookup table by `PanelId`.
    pub fn from_panels(panels: &DockPanels) -> Self {
        Self(
            panels
                .0
                .iter()
                .map(|panel| (panel.id.0.clone(), panel.clone()))
                .collect(),
        )
    }

    /// Looks up a registered panel by id.
    pub fn get(&self, id: &PanelId) -> Option<&PanelDef> {
        self.0.get(&id.0)
    }
}
