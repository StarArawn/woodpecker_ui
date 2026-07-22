mod tab_button;
mod tab_content;

use crate::prelude::*;
use bevy::prelude::*;

pub use tab_button::*;
pub use tab_content::*;

/// [`TabContextProvider`]'s themed panel colors -- a separate sibling component (rather than
/// the hardcoded `colors::BACKGROUND_LIGHT` fallback this used before) so a *newly-spawned*
/// provider's default background/border tracks the active [`Theme`] instead of the static
/// `colors` module. `render()` only applies these once, the first time `WoodpeckerStyle`'s
/// `background_color`/`border_color` is still fully transparent (its `#[require]`-inserted
/// starting value) -- both to avoid clobbering a caller's own customization on the provider's
/// bundle, and as a side effect meaning an *already-rendered* provider won't re-pick-up a
/// later `Theme` swap (same "no resync on spawn" tradeoff `ThemeRegisterExt::register_themed_style`
/// documents, plus this widget's own preserve-caller-customization design on top of it).
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct TabStyles {
    /// Default panel background color.
    pub background_color: Color,
    /// Default panel border color.
    pub border_color: Color,
}

impl Default for TabStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for TabStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_color: theme.background_light,
            border_color: theme.background_light,
        }
    }
}

/// Tab context
#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct TabContext {
    /// The current tab selected
    current_index: usize,
}

/// Tab context provider
/// Provides the context for the tab widgets.
#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WidgetRender = WidgetRender::Quad, TabStyles)]
pub struct TabContextProvider {
    /// The initial tab that is selected.
    initial_tab_index: usize,
}

/// A tab context provider bundle
#[derive(Bundle, Clone, Default)]
pub struct TabContextProviderBundle {
    /// Tab Context Provider
    pub provider: TabContextProvider,
    /// Internal styles
    pub styles: WoodpeckerStyle,
    /// Children passed in
    pub children: PassedChildren,
    /// Internal Children
    pub internal_children: WidgetChildren,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &TabContextProvider,
        &TabStyles,
        &mut WidgetChildren,
        &mut WoodpeckerStyle,
        &PassedChildren,
    )>,
) {
    let Ok((provider, tab_styles, mut children, mut styles, passed_children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let _context_entity = hooks.use_context(
        &mut commands,
        *current_widget,
        TabContext {
            current_index: provider.initial_tab_index,
        },
    );

    // Mutate in place rather than replacing the whole struct so caller-provided
    // `TabContextProviderBundle::styles` customization isn't discarded. Compared against
    // `WoodpeckerStyle::DEFAULT`'s actual zero/transparent values, not `Color`/`Corner`/`Edge`
    // derived defaults, which never equal them and so would never fire.
    //
    // Deliberately NOT `== Color::NONE`: `Color::NONE` is Bevy's `LinearRgba` variant, but
    // `WoodpeckerStyle::DEFAULT`'s `background_color`/`border_color` are the `Srgba` variant
    // (`Color::Srgba(Srgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 })`) -- visually
    // identical (both fully transparent black) but a *different enum variant*, so `==` is
    // always `false` and this check never fired, leaving the panel permanently transparent/
    // borderless regardless of `styles.background_color`. `.alpha() == 0.0` compares the
    // actual transparency instead, which works across every `Color` variant.
    if styles.background_color.alpha() == 0.0 {
        styles.background_color = tab_styles.background_color;
    }
    if styles.border_color.alpha() == 0.0 {
        styles.border_color = tab_styles.border_color;
    }
    if styles.border == Edge::all(0.0) {
        styles.border = Edge::all(2.0);
    }
    if styles.border_radius == Corner::all(0.0) {
        styles.border_radius = Corner::all(8.0);
    }
    if styles.width == Units::Auto {
        styles.width = Units::Percentage(100.0);
    }
    if styles.height == Units::Auto {
        styles.height = Units::Percentage(100.0);
    }
    let border_radius = styles.border_radius;

    *children = WidgetChildren::default().with_child::<Clip>((
        Clip,
        WoodpeckerStyle {
            border_radius,
            flex_direction: WidgetFlexDirection::Column,
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            ..Default::default()
        },
        passed_children.0.clone(),
    ));
    children.add_key("clip");

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_background_light() {
        let dark = TabStyles::from_theme(&Theme::dark());
        assert_eq!(dark.background_color, Theme::dark().background_light);
        assert_eq!(dark.border_color, Theme::dark().background_light);

        let light = TabStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.background_color, light.background_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
