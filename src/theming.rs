use bevy::{ecs::component::Mutable, prelude::*};

use crate::{styles::WidgetBoxShadow, widgets::colors};

/// Design tokens every built-in widget's default style derives from. Swapping the resource
/// (`commands.insert_resource(Theme::light())`) live-updates every registered widget style
/// in place -- see [`ThemeRegisterExt::register_themed_style`]. Derives `Reflect` so apps can
/// also `register_watched_resource::<Theme>()` and react to a swap themselves -- e.g. to pick
/// a legible custom text color, since a caller-supplied label (like `WButton::text()`'s) isn't
/// itself a registered themed style and won't recolor automatically.
#[derive(Resource, Reflect, Clone, Copy, PartialEq)]
pub struct Theme {
    /// The darkest surface (text input fills, chrome bars).
    pub dark_background: Color,
    /// The base surface for controls (buttons, panels, dropdowns).
    pub background: Color,
    /// One step lighter than `background` -- raised controls, hover.
    pub background_mid: Color,
    /// Hover/active surfaces.
    pub background_light: Color,
    /// Subtle border/divider color.
    pub border: Color,
    /// Primary accent -- focus rings, selected/checked states, hover highlights.
    pub primary: Color,
    /// Lighter accent variant for hover states.
    pub primary_light: Color,
    /// Body text color.
    pub text: Color,
    /// Success/positive status color (badges, toasts, progress).
    pub success: Color,
    /// Danger/negative status color (badges, toasts, progress).
    pub danger: Color,
    /// Warning/caution status color (badges, toasts, progress).
    pub warning: Color,
    /// Shared font size for standard form controls and titles.
    pub font_size: f32,
    /// Shared height for single-line form controls.
    pub control_height: f32,
    /// Corner radius for form controls.
    pub control_radius: f32,
    /// Corner radius for larger panel-level surfaces.
    pub panel_radius: f32,
    /// Shared spacing scale for gaps/padding/margins.
    pub spacing: Spacing,
    /// Shared font-size scale for headings and body text.
    pub typography: Typography,
    /// Drop-shadow presets for widgets that float above the content behind them.
    pub elevation: Elevation,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

/// A shared spacing scale, matching the `SPACE_XS`/`SPACE_SM`/`SPACE_MD`/`SPACE_LG` constants
/// every one of this crate's own examples (`examples/dashboard/theme.rs`,
/// `examples/game_ui/theme.rs`) independently hand-rolled to the same values before this
/// existed -- `xl` is the one new step, for `Paper`/`Drawer`/`AppBar`-scale surfaces.
#[derive(Reflect, Clone, Copy, PartialEq)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl Spacing {
    pub const fn default_scale() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
        }
    }
}

/// A shared font-size scale. Deliberately 6 roles, not MUI's 13 -- `WoodpeckerStyle` has no
/// `font_weight` field, so weight/letter-spacing-differentiated roles (`button`, `overline`,
/// `subtitle1`/`subtitle2`) would be indistinguishable from `body`/`caption` in practice.
#[derive(Reflect, Clone, Copy, PartialEq)]
pub struct Typography {
    /// Large page/section heading.
    pub h1: f32,
    /// Sub-heading / card title.
    pub h2: f32,
    /// Small heading / emphasized label.
    pub h3: f32,
    /// Default body text. Deliberately duplicates `Theme::font_size` rather than aliasing it,
    /// so `Typography` stays a complete, self-contained scale on its own.
    pub body: f32,
    /// Secondary/de-emphasized body text (helper text, a list item's secondary line).
    pub body_small: f32,
    /// Smallest legible text (badge labels, timestamps, captions).
    pub caption: f32,
}

impl Typography {
    pub const fn default_scale() -> Self {
        Self {
            h1: 24.0,
            h2: 18.0,
            h3: 16.0,
            body: 14.0,
            body_small: 13.0,
            caption: 12.0,
        }
    }
}

/// A shared drop-shadow scale for widgets that float above the content behind them (dropdowns,
/// popovers, modals, a dragged window...). Four tiers, not a continuous elevation number --
/// matches [`Spacing`]/[`Typography`]'s own "a handful of named roles" shape. Shadows are
/// deliberately the same dark, semi-transparent black in both [`Theme::dark`] and
/// [`Theme::light`]: a drop shadow reads as "shadow" regardless of the surface color it falls
/// on, the same way every OS/design system keeps window/panel shadows dark in light mode too.
#[derive(Reflect, Clone, Copy, PartialEq)]
pub struct Elevation {
    /// A light lift -- `Paper` elevation 1, a tooltip label.
    pub sm: WidgetBoxShadow,
    /// A dropdown/menu/popover panel, `Paper` elevation 2, a toast card.
    pub md: WidgetBoxShadow,
    /// A temporary drawer, `Paper` elevation 3.
    pub lg: WidgetBoxShadow,
    /// The most prominent floating surface -- a modal dialog, a dragged window, `Paper`
    /// elevation 4.
    pub xl: WidgetBoxShadow,
}

impl Elevation {
    pub const fn default_scale() -> Self {
        const fn shadow(y_offset: f32, blur_radius: f32, alpha: f32) -> WidgetBoxShadow {
            WidgetBoxShadow {
                x_offset: 0.0,
                y_offset,
                spread: 0.0,
                blur_radius,
                color: Color::Srgba(Srgba {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha,
                }),
            }
        }
        Self {
            sm: shadow(1.0, 3.0, 0.3),
            md: shadow(2.0, 8.0, 0.35),
            lg: shadow(4.0, 16.0, 0.4),
            xl: shadow(8.0, 28.0, 0.45),
        }
    }
}

/// Selects which [`SurfacePalette`] and accent brightness [`resolve`] uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

/// The mode-specific surface/text colors [`resolve`] pulls from [`ThemeTokens`].
#[derive(Clone, Copy, PartialEq)]
pub struct SurfacePalette {
    pub dark_background: Color,
    pub background: Color,
    pub background_mid: Color,
    pub background_light: Color,
    pub border: Color,
    pub text: Color,
}

/// Single source of truth [`resolve`] builds both [`Theme::dark`] and [`Theme::light`] from --
/// pass a custom instance to [`resolve`] for a fully custom brand theme with both modes derived
/// together instead of hand-writing two complete [`Theme`] structs.
#[derive(Clone, Copy, PartialEq)]
pub struct ThemeTokens {
    pub dark_surfaces: SurfacePalette,
    pub light_surfaces: SurfacePalette,
    pub primary: Color,
    pub primary_light: Color,
    pub success: Color,
    pub danger: Color,
    pub warning: Color,
    pub font_size: f32,
    pub control_height: f32,
    pub control_radius: f32,
    pub panel_radius: f32,
    pub spacing: Spacing,
    pub typography: Typography,
    pub elevation: Elevation,
}

impl ThemeTokens {
    /// Matches this crate's original hardcoded palette.
    pub fn default_scale() -> Self {
        Self {
            dark_surfaces: SurfacePalette {
                dark_background: colors::DARK_BACKGROUND,
                background: colors::BACKGROUND,
                background_mid: colors::BACKGROUND_MID,
                background_light: colors::BACKGROUND_LIGHT,
                border: colors::BORDER,
                text: colors::TEXT,
            },
            light_surfaces: SurfacePalette {
                dark_background: Color::Srgba(Srgba::new(0.996, 0.996, 1.0, 1.0)),
                background: Color::Srgba(Srgba::new(0.973, 0.976, 0.984, 1.0)),
                background_mid: Color::srgba(0.933, 0.937, 0.949, 1.0),
                background_light: Color::Srgba(Srgba::new(0.878, 0.886, 0.906, 1.0)),
                border: Color::Srgba(Srgba::new(0.788, 0.796, 0.827, 1.0)),
                text: Color::Srgba(Srgba::new(0.114, 0.129, 0.169, 1.0)),
            },
            primary: colors::PRIMARY,
            primary_light: colors::PRIMARY_LIGHT,
            success: colors::SUCCESS,
            danger: colors::DANGER,
            warning: colors::WARNING,
            font_size: colors::FONT_SIZE,
            control_height: colors::CONTROL_HEIGHT,
            control_radius: colors::CONTROL_RADIUS,
            panel_radius: colors::PANEL_RADIUS,
            spacing: Spacing::default_scale(),
            typography: Typography::default_scale(),
            elevation: Elevation::default_scale(),
        }
    }
}

/// Accent hues darken to this fraction of their dark-mode brightness in light mode, to keep
/// contrast against near-white surfaces (matches Material Design's own light-mode tradeoff).
const LIGHT_ACCENT_SCALE: f32 = 0.78;

fn round3(x: f32) -> f32 {
    (x * 1000.0).round() / 1000.0
}

fn scale_brightness(color: Color, factor: f32) -> Color {
    let srgba = color.to_srgba();
    Color::Srgba(Srgba::new(
        round3(srgba.red * factor),
        round3(srgba.green * factor),
        round3(srgba.blue * factor),
        srgba.alpha,
    ))
}

/// Builds a full [`Theme`] for `mode` from `tokens`.
pub fn resolve(tokens: &ThemeTokens, mode: ThemeMode) -> Theme {
    let surfaces = match mode {
        ThemeMode::Dark => tokens.dark_surfaces,
        ThemeMode::Light => tokens.light_surfaces,
    };
    let accent_scale = match mode {
        ThemeMode::Dark => 1.0,
        ThemeMode::Light => LIGHT_ACCENT_SCALE,
    };
    Theme {
        dark_background: surfaces.dark_background,
        background: surfaces.background,
        background_mid: surfaces.background_mid,
        background_light: surfaces.background_light,
        border: surfaces.border,
        text: surfaces.text,
        primary: scale_brightness(tokens.primary, accent_scale),
        primary_light: scale_brightness(tokens.primary_light, accent_scale),
        success: scale_brightness(tokens.success, accent_scale),
        danger: scale_brightness(tokens.danger, accent_scale),
        warning: scale_brightness(tokens.warning, accent_scale),
        font_size: tokens.font_size,
        control_height: tokens.control_height,
        control_radius: tokens.control_radius,
        panel_radius: tokens.panel_radius,
        spacing: tokens.spacing,
        typography: tokens.typography,
        elevation: tokens.elevation,
    }
}

impl Theme {
    /// Identical to this crate's original hardcoded `colors` palette -- a zero-config app
    /// sees no visual change from before theming existed.
    pub fn dark() -> Self {
        resolve(&ThemeTokens::default_scale(), ThemeMode::Dark)
    }

    /// A light counterpart to [`Self::dark`] -- same accent hues, surfaces inverted to light
    /// neutrals, body text darkened for contrast.
    pub fn light() -> Self {
        resolve(&ThemeTokens::default_scale(), ThemeMode::Light)
    }
}

/// Implemented by any `*Styles` component that should track the active [`Theme`]. Register
/// with [`ThemeRegisterExt::register_themed_style`] so a `Theme` swap resyncs every instance
/// automatically.
pub trait ThemedStyle: Sized {
    /// Computes this style from the given theme's tokens.
    fn from_theme(theme: &Theme) -> Self;
}

/// Per-field theme overrides for one widget instance -- non-`None` fields stay pinned to their
/// override value across a [`Theme`] swap; `None` fields keep tracking the live theme.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct TokenPatch {
    pub dark_background: Option<Color>,
    pub background: Option<Color>,
    pub background_mid: Option<Color>,
    pub background_light: Option<Color>,
    pub border: Option<Color>,
    pub primary: Option<Color>,
    pub primary_light: Option<Color>,
    pub text: Option<Color>,
    pub success: Option<Color>,
    pub danger: Option<Color>,
    pub warning: Option<Color>,
    pub font_size: Option<f32>,
    pub control_height: Option<f32>,
    pub control_radius: Option<f32>,
    pub panel_radius: Option<f32>,
}

impl TokenPatch {
    fn apply(&self, theme: &Theme) -> Theme {
        Theme {
            dark_background: self.dark_background.unwrap_or(theme.dark_background),
            background: self.background.unwrap_or(theme.background),
            background_mid: self.background_mid.unwrap_or(theme.background_mid),
            background_light: self.background_light.unwrap_or(theme.background_light),
            border: self.border.unwrap_or(theme.border),
            primary: self.primary.unwrap_or(theme.primary),
            primary_light: self.primary_light.unwrap_or(theme.primary_light),
            text: self.text.unwrap_or(theme.text),
            success: self.success.unwrap_or(theme.success),
            danger: self.danger.unwrap_or(theme.danger),
            warning: self.warning.unwrap_or(theme.warning),
            font_size: self.font_size.unwrap_or(theme.font_size),
            control_height: self.control_height.unwrap_or(theme.control_height),
            control_radius: self.control_radius.unwrap_or(theme.control_radius),
            panel_radius: self.panel_radius.unwrap_or(theme.panel_radius),
            ..*theme
        }
    }
}

/// Opt one widget instance out of live theme resync -- e.g. a hand-styled one-off that
/// shouldn't change when the app swaps themes. Attach alongside the widget's `*Styles`
/// component. A bare `ThemeOverride`/`ThemeOverride::default()` freezes the instance entirely;
/// `ThemeOverride(TokenPatch { primary: Some(my_color), ..default() })` instead keeps it live
/// but pins just the given fields.
#[derive(Component, Default, Clone, Copy, PartialEq)]
pub struct ThemeOverride(pub TokenPatch);

/// Setup-time extension for registering a `*Styles` component as theme-aware.
pub trait ThemeRegisterExt {
    /// Registers `S` so every instance without a [`ThemeOverride`] sibling is recomputed via
    /// `S::from_theme` whenever the [`Theme`] resource changes.
    ///
    /// Deliberately does *not* resync on spawn: many apps (see this crate's own `examples/`)
    /// hand-customize a `*Styles` component directly on a widget's own bundle -- there's no
    /// way to tell that apart from an untouched `#[require(... = S::default())]` value, and
    /// resyncing every newly-spawned `S` would silently clobber those per-instance
    /// customizations. The tradeoff: a widget spawned after the app already swapped away
    /// from [`Theme::default()`] renders with the old default theme's colors until the next
    /// explicit [`Theme`] swap. Toggling the theme once after startup (or setting the
    /// desired `Theme` before `WoodpeckerUIPlugin` spawns any widgets) avoids this.
    fn register_themed_style<S: ThemedStyle + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self;
}

impl ThemeRegisterExt for App {
    fn register_themed_style<S: ThemedStyle + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self {
        self.add_systems(
            PreUpdate,
            resync_themed_style::<S>.run_if(resource_changed::<Theme>),
        );
        self
    }
}

fn resync_themed_style<S: ThemedStyle + Component<Mutability = Mutable>>(
    theme: Res<Theme>,
    mut query: Query<(&mut S, Option<&ThemeOverride>)>,
) {
    for (mut style, over) in query.iter_mut() {
        match over {
            None => *style = S::from_theme(&theme),
            Some(ThemeOverride(patch)) if *patch == TokenPatch::default() => {}
            Some(ThemeOverride(patch)) => *style = S::from_theme(&patch.apply(&theme)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channels(color: Color) -> (f32, f32, f32, f32) {
        let srgba = color.to_srgba();
        (srgba.red, srgba.green, srgba.blue, srgba.alpha)
    }

    fn assert_close(a: Color, b: Color) {
        let (ar, ag, ab, aa) = channels(a);
        let (br, bg, bb, ba) = channels(b);
        assert!((ar - br).abs() < 0.002);
        assert!((ag - bg).abs() < 0.002);
        assert!((ab - bb).abs() < 0.002);
        assert!((aa - ba).abs() < 0.002);
    }

    #[test]
    fn resolved_dark_matches_the_original_hardcoded_colors_exactly() {
        let dark = Theme::dark();
        assert_eq!(dark.dark_background, colors::DARK_BACKGROUND);
        assert_eq!(dark.background, colors::BACKGROUND);
        assert_eq!(dark.primary, colors::PRIMARY);
        assert_eq!(dark.success, colors::SUCCESS);
        assert_eq!(dark.danger, colors::DANGER);
        assert_eq!(dark.warning, colors::WARNING);
        assert_eq!(dark.text, colors::TEXT);
    }

    #[test]
    fn resolved_light_accents_match_the_original_hand_typed_values() {
        let light = Theme::light();
        assert_close(light.primary, Color::Srgba(Srgba::new(0.291, 0.398, 0.753, 1.0)));
        assert_close(
            light.primary_light,
            Color::Srgba(Srgba::new(0.371, 0.462, 0.761, 1.0)),
        );
        assert_close(light.success, Color::Srgba(Srgba::new(0.273, 0.569, 0.390, 1.0)));
        assert_close(light.danger, Color::Srgba(Srgba::new(0.718, 0.312, 0.312, 1.0)));
        assert_close(light.warning, Color::Srgba(Srgba::new(0.718, 0.507, 0.195, 1.0)));
        assert_close(light.text, Color::Srgba(Srgba::new(0.114, 0.129, 0.169, 1.0)));
    }

    #[test]
    fn token_patch_default_leaves_the_theme_unchanged() {
        let theme = Theme::dark();
        assert!(TokenPatch::default().apply(&theme) == theme);
    }

    #[test]
    fn token_patch_pins_only_its_own_fields() {
        let theme = Theme::dark();
        let patch = TokenPatch {
            primary: Some(Color::WHITE),
            ..Default::default()
        };
        let patched = patch.apply(&theme);
        assert_eq!(patched.primary, Color::WHITE);
        assert_eq!(patched.text, theme.text);
        assert_eq!(patched.background, theme.background);
    }

    #[test]
    fn spacing_default_scale_matches_the_values_every_example_already_hand_rolled() {
        let spacing = Spacing::default_scale();
        assert_eq!(spacing.xs, 4.0);
        assert_eq!(spacing.sm, 8.0);
        assert_eq!(spacing.md, 16.0);
        assert_eq!(spacing.lg, 24.0);
        assert_eq!(spacing.xl, 32.0);
    }

    #[test]
    fn typography_body_matches_the_existing_font_size_token() {
        // `Typography::body` is a deliberate duplicate of `Theme::font_size`, not an alias --
        // this pins them equal so the two scales can't silently drift apart.
        assert_eq!(Typography::default_scale().body, colors::FONT_SIZE);
    }

    #[test]
    fn dark_and_light_themes_both_carry_the_default_token_scales() {
        assert!(Theme::dark().spacing == Spacing::default_scale());
        assert!(Theme::light().spacing == Spacing::default_scale());
        assert!(Theme::dark().typography == Typography::default_scale());
        assert!(Theme::light().typography == Typography::default_scale());
        assert!(Theme::dark().elevation == Elevation::default_scale());
        assert!(Theme::light().elevation == Elevation::default_scale());
    }

    #[test]
    fn elevation_tiers_grow_monotonically_and_stay_a_dark_shadow_in_both_themes() {
        let elevation = Elevation::default_scale();
        assert!(elevation.sm.blur_radius < elevation.md.blur_radius);
        assert!(elevation.md.blur_radius < elevation.lg.blur_radius);
        assert!(elevation.lg.blur_radius < elevation.xl.blur_radius);
        assert!(elevation.sm.y_offset < elevation.xl.y_offset);
        for tier in [elevation.sm, elevation.md, elevation.lg, elevation.xl] {
            let color = tier.color.to_srgba();
            assert_eq!((color.red, color.green, color.blue), (0.0, 0.0, 0.0));
        }
    }
}
