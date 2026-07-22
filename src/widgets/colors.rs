use bevy::prelude::*;

/// Default dark background color -- the darkest surface (text input fills, chrome bars).
pub const DARK_BACKGROUND: Color = Color::Srgba(Srgba::new(0.075, 0.082, 0.102, 1.0));
/// Default background color -- the base surface for controls (buttons, panels, dropdowns).
///
/// Deliberately a big jump up from [`DARK_BACKGROUND`] so a control reads as clearly
/// "raised" even against a host app's own dark background.
pub const BACKGROUND: Color = Color::Srgba(Srgba::new(0.165, 0.180, 0.216, 1.0));
/// Default mid background color -- one step lighter, used for raised controls and hover.
pub const BACKGROUND_MID: Color = Color::srgba(0.208, 0.224, 0.267, 1.0);
/// Default light background color -- hover/active surfaces.
pub const BACKGROUND_LIGHT: Color = Color::Srgba(Srgba::new(0.259, 0.278, 0.329, 1.0));
/// Default subtle border/divider color, one step lighter than [`BACKGROUND_LIGHT`] so it
/// reads as a hairline outline rather than another fill.
#[allow(
    clippy::approx_constant,
    reason = "0.318 is a color channel, not pi-related"
)]
pub const BORDER: Color = Color::Srgba(Srgba::new(0.318, 0.341, 0.400, 1.0));
/// Default primary accent color -- a clean blue used for focus rings, selected/checked
/// states, and hover highlights.
pub const PRIMARY: Color = Color::Srgba(Srgba::new(0.373, 0.51, 0.965, 1.0));
/// Default primary light color -- lighter accent variant for hover states.
pub const PRIMARY_LIGHT: Color = Color::Srgba(Srgba::new(0.475, 0.592, 0.976, 1.0));
/// Default body text color -- a soft off-white, easier on the eyes than pure white on a
/// dark surface.
pub const TEXT: Color = Color::Srgba(Srgba::new(0.94, 0.945, 0.96, 1.0));
/// Default success/positive status color (badges, toasts, progress).
pub const SUCCESS: Color = Color::Srgba(Srgba::new(0.35, 0.73, 0.5, 1.0));
/// Default danger/negative status color (badges, toasts, progress).
pub const DANGER: Color = Color::Srgba(Srgba::new(0.92, 0.4, 0.4, 1.0));
/// Default warning/caution status color (badges, toasts, progress).
pub const WARNING: Color = Color::Srgba(Srgba::new(0.92, 0.65, 0.25, 1.0));

/// Default font size shared by standard form controls and titles (buttons, text inputs,
/// dropdowns, tabs, modal/window titles) so they read as one consistent scale.
pub const FONT_SIZE: f32 = 14.0;
/// Default height for single-line form controls (buttons, text inputs, dropdown header).
pub const CONTROL_HEIGHT: f32 = 28.0;
/// Default corner radius for form controls (buttons, inputs, checkboxes, dropdown items).
pub const CONTROL_RADIUS: f32 = 8.0;
/// Default corner radius for larger panel-level surfaces (modals, windows, color picker).
pub const PANEL_RADIUS: f32 = 10.0;
