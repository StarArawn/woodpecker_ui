use bevy::prelude::*;
use woodpecker_ui::prelude::*;

/// A "Toggle Theme" button swapping `Theme::dark()`/`Theme::light()` live, plus a handful of
/// default-styled built-in widgets (no per-instance `*Styles` overrides) so the swap is
/// visibly, immediately obvious -- every `register_themed_style`-registered widget resyncs
/// in place, no despawn/respawn.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<ThemeDemo>()
        .register_watched_resource::<Theme>()
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert(
        WidgetChildren::default()
            .with_child::<ThemeDemo>(ThemeDemo)
            .with_child::<OverlayRootWidget>(OverlayRootWidget),
    );
}

/// Watches `Theme` so the "Toggle Theme" button's own label can pick a contrasting color --
/// `WButton::text()` hardcodes `Color::WHITE` (fine for `Theme::dark()`, invisible against
/// `Theme::light()`'s button background), and has no way to read the live theme itself since
/// it builds a static bundle with no `World`/`Res` access. Every other control below uses its
/// crate-default styling untouched, so its `register_themed_style` resync alone is enough.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_style(), WidgetChildren, WatchedResource<Theme>)]
struct ThemeDemo;

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        padding: Edge::all(24.0),
        flex_direction: WidgetFlexDirection::Column,
        ..Default::default()
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&WatchedResource<Theme>, &mut WidgetChildren)>,
) {
    let Ok((theme, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let theme = theme.0;
    let current_widget = *current_widget;

    *children = WidgetChildren::default()
        .with_child::<WButton>((
            WButton,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: theme.text,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Toggle Theme".into(),
                },
            )),
        ))
        .with_key("toggle")
        .with_observe(
            current_widget,
            // Reads the *live* `Theme` resource as a system param, rather than closing over
            // the `theme` this render happened to see -- `.observe()` only ever attaches its
            // observer once (cached across re-renders, see `WidgetChildren::observe`'s doc
            // comment), so a captured-by-value snapshot would permanently freeze at whatever
            // the theme was on first mount, making every click after the very first compare
            // against that stale value instead of the current one.
            |_: On<Pointer<Click>>, theme: Res<Theme>, mut commands: Commands| {
                let next = if *theme == Theme::dark() {
                    Theme::light()
                } else {
                    Theme::dark()
                };
                commands.insert_resource(next);
            },
        )
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                margin: Edge::all(0.0).top(20.0),
                flex_direction: WidgetFlexDirection::Column,
                // Without a gap, `Toggle`'s thumb -- which is visually taller than the
                // switch's own 14px flow height, via its `position: Absolute` circle --
                // overlaps whatever sits directly above it.
                gap: (0.0.into(), 16.0.into()),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Checkbox>(Checkbox)
                .with_key("checkbox")
                .with_child::<Toggle>(Toggle)
                .with_key("toggle-switch")
                .with_child::<Slider>((Slider {
                    start: 0.0,
                    end: 1.0,
                    value: 0.5,
                },))
                .with_key("slider")
                .with_child::<Dropdown>(Dropdown {
                    list: vec!["Red".into(), "Green".into(), "Blue".into()],
                    current_value: "Red".into(),
                })
                .with_key("dropdown")
                .with_child::<TextBox>((TextBox {
                    initial_value: "Type here...".into(),
                    ..Default::default()
                },))
                .with_key("textbox"),
        ))
        .with_key("controls");

    children.apply(current_widget.as_parent());
}
