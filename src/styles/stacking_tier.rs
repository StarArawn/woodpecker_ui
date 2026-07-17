/// The reserved z tiers for every built-in "floats above the rest of the app" widget, used
/// with [`crate::styles::WidgetZ::Global`] (e.g. `WidgetZ::Global(StackingTier::Modal as u32)`).
///
/// Replaces what used to be a scattered set of hand-picked magic numbers (`50`, `1000`,
/// `2000`, `3000`, `5000`, ...) across each widget's own file -- one widget author picking a
/// number without knowing what every other widget already uses is exactly how these tiers
/// drifted close enough together to collide (see [`StackingTier::Window`]'s doc comment).
/// Centralizing them here means a new tier is chosen with the full picture in view, the same
/// way a real CSS codebase manages z-index via a shared scale/design-token file rather than
/// every component picking its own number.
///
/// Spaced 100,000 apart, not sequentially (`0, 1, 2, ...`) -- deliberately far more headroom
/// than any tier will ever need on its own, so a widget that adds a small *offset* on top of
/// its base tier (see [`StackingTier::Window`]) can never numerically reach the next tier up,
/// however large that offset gets in practice.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StackingTier {
    /// Base tier for [`crate::widgets::WoodpeckerWindow`] -- its actual z is
    /// `StackingTier::Window as u32 + window_index`, where `window_index` is the small,
    /// per-`WindowingContextProvider` ascending ordinal `WindowingContext::get_or_add`/
    /// `shift_to_top` already assign. Previously the raw ordinal was used directly as the
    /// window's `WidgetZ::Global` value with no base offset at all, meaning it shared the
    /// exact same numeric axis as `Modal`'s `z=50` -- with 50+ windows open simultaneously
    /// under one provider, a window's z would reach and exceed `Modal`'s tier and start
    /// rendering/picking above an open modal. The 100,000-unit gap here means that can no
    /// longer happen at any window count realistic for this crate's use cases.
    Window = 0,
    /// [`crate::widgets::Drawer`]'s temporary (non-`Persistent`) variant -- above ambient app
    /// content and any open window, but deliberately below `Modal` so a confirm-before-closing
    /// dialog (or any other modal) can still appear on top of an open drawer. Not a multiple
    /// of 100,000 like the other tiers -- it sits in the gap between `Window` and `Modal`
    /// specifically, since no realistic `WoodpeckerWindow` z-offset (bounded well under this
    /// value -- see `WindowingContext`) can reach it.
    Drawer = 50_000,
    /// Modal dialogs -- above ambient app content and any open window, below every
    /// "attached to a trigger" floating panel.
    Modal = 100_000,
    /// Shared by [`crate::widgets::Dropdown`], [`crate::widgets::ComboBox`], and
    /// [`crate::widgets::Menu`]'s open panels -- these are mutually-exclusive, "one open near
    /// its own trigger at a time" widgets with no existing requirement to be ordered relative
    /// to each other, so sharing one tier (falling back to insertion order as the tiebreak,
    /// same as before this change) is intentional, not an oversight.
    Dropdown = 200_000,
    /// [`crate::widgets::Popover`]'s floating content -- also the tier
    /// [`crate::widgets::DatePicker`]'s calendar panel resolves to, since it's built on top
    /// of `Popover` and never overrides this itself.
    Popover = 300_000,
    /// [`crate::widgets::Tooltip`]'s floating label.
    Tooltip = 400_000,
    /// [`crate::widgets::ToastViewport`]'s stacked notification cards -- the highest
    /// app-content tier; nothing in a normal app is meant to render above a toast.
    Toast = 500_000,
    /// The runtime devtools inspector panel (`devtools` Cargo feature). Above every
    /// app-content tier including `Toast`, so the panel stays usable regardless of what the
    /// inspected app itself has open.
    Devtools = 600_000,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pins the tier scale's relative ordering to what the old, scattered magic numbers
    /// (`Modal=50 < Dropdown/ComboBox/Menu=1000 < Popover=2000 < Tooltip=3000 < Toast=5000`)
    /// already established -- the rename in this phase is meant to be a pure numeric
    /// substitution, not a re-ranking, so this must hold even though the absolute values
    /// changed completely.
    #[test]
    fn tiers_preserve_the_pre_rename_relative_ordering() {
        assert!(StackingTier::Window < StackingTier::Drawer);
        assert!(StackingTier::Drawer < StackingTier::Modal);
        assert!(StackingTier::Modal < StackingTier::Dropdown);
        assert!(StackingTier::Dropdown < StackingTier::Popover);
        assert!(StackingTier::Popover < StackingTier::Tooltip);
        assert!(StackingTier::Tooltip < StackingTier::Toast);
        assert!(StackingTier::Toast < StackingTier::Devtools);
    }

    /// `WoodpeckerWindow`'s z is `StackingTier::Window as u32 + window_index` -- this must
    /// never reach `StackingTier::Modal`'s tier, however many windows are open under one
    /// `WindowingContextProvider`. 100,000 windows is far beyond any realistic app, chosen
    /// specifically to prove the 100,000-unit gap holds with room to spare, not just for a
    /// plausible window count.
    #[test]
    fn window_offset_cannot_reach_the_modal_tier_even_with_100_000_windows() {
        let max_window_index = 100_000u32 - 1;
        assert!(StackingTier::Window as u32 + max_window_index < StackingTier::Modal as u32);
    }
}
