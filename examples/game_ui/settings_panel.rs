use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::mock_data::*;
use crate::theme;

fn setting_row(label: &str, control: WidgetChildren) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Row,
            align_items: Some(WidgetAlignItems::Center),
            margin: Edge::all(0.0).bottom(theme::SPACE_SM),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                theme::label_style(),
                WidgetRender::Text {
                    content: label.into(),
                },
            ))
            .with_key("label")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: 220.0.into(),
                    ..Default::default()
                },
                control,
            ))
            .with_key("control"),
    )
}

fn section_divider() -> impl Bundle + Clone {
    (
        Divider { vertical: false },
        DividerStyles {
            color: theme::BORDER_DARK,
        },
        WoodpeckerStyle {
            margin: Edge::all(0.0).top(theme::SPACE_SM).bottom(theme::SPACE_MD),
            ..Default::default()
        },
    )
}

/// Plain builder function (no custom widget needed) -- none of these controls need to read
/// reactive state back except the accent color, watched independently by the HUD.
pub fn build_settings_panel(current_widget: CurrentWidget) -> WidgetChildren {
    WidgetChildren::default()
        .with_child::<Element>((
            Element,
            theme::section_title_style(),
            WidgetRender::Text {
                content: "Audio".into(),
            },
        ))
        .with_key("audio_title")
        .with_child::<Element>(setting_row(
            "Master Volume",
            WidgetChildren::default()
                .with_child::<Slider>((Slider {
                    start: 0.0,
                    end: 1.0,
                    value: 0.8,
                },))
                .with_observe(
                    current_widget,
                    |trigger: On<Change<SliderChanged>>, mut settings: ResMut<GameSettings>| {
                        settings.master_volume = trigger.data.value;
                    },
                ),
        ))
        .with_key("master_volume")
        .with_child::<Element>(setting_row(
            "Music Volume",
            WidgetChildren::default()
                .with_child::<Slider>((Slider {
                    start: 0.0,
                    end: 1.0,
                    value: 0.6,
                },))
                .with_observe(
                    current_widget,
                    |trigger: On<Change<SliderChanged>>, mut settings: ResMut<GameSettings>| {
                        settings.music_volume = trigger.data.value;
                    },
                ),
        ))
        .with_key("music_volume")
        .with_child::<Element>(section_divider())
        .with_key("divider1")
        .with_child::<Element>((
            Element,
            theme::section_title_style(),
            WidgetRender::Text {
                content: "Video".into(),
            },
        ))
        .with_key("video_title")
        .with_child::<Element>(setting_row(
            "Graphics Preset",
            WidgetChildren::default()
                .with_child::<Dropdown>((Dropdown {
                    list: GRAPHICS_PRESETS.iter().map(|p| p.to_string()).collect(),
                    current_value: GRAPHICS_PRESETS[2].into(),
                },))
                .with_observe(
                    current_widget,
                    |trigger: On<Change<DropdownChanged>>, mut settings: ResMut<GameSettings>| {
                        if let Some(index) = GRAPHICS_PRESETS
                            .iter()
                            .position(|p| *p == trigger.data.value)
                        {
                            settings.graphics_preset_index = index;
                        }
                    },
                ),
        ))
        .with_key("graphics")
        .with_child::<Element>(section_divider())
        .with_key("divider2")
        .with_child::<Element>((
            Element,
            theme::section_title_style(),
            WidgetRender::Text {
                content: "Gameplay".into(),
            },
        ))
        .with_key("gameplay_title")
        .with_child::<Element>(setting_row(
            "Difficulty",
            WidgetChildren::default()
                .with_child::<RadioGroup>((RadioGroup {
                    options: DIFFICULTIES.iter().map(|d| d.to_string()).collect(),
                    selected: 1,
                    // Horizontal doesn't fit 4 options (incl. "Nightmare") in this row's
                    // control column width -- it overflowed straight past the window's edge.
                    horizontal: false,
                },))
                .with_observe(
                    current_widget,
                    |trigger: On<Change<RadioChanged>>, mut settings: ResMut<GameSettings>| {
                        settings.difficulty_index = trigger.data.index;
                    },
                ),
        ))
        .with_key("difficulty")
        .with_child::<Element>(setting_row(
            // Text overflows its declared width rather than clipping, so a longer label here
            // would visually collide with the checkbox next to it.
            "Damage Numbers",
            WidgetChildren::default()
                .with_child::<Checkbox>((Checkbox,))
                .with_observe(
                    current_widget,
                    |trigger: On<Change<CheckboxChanged>>, mut settings: ResMut<GameSettings>| {
                        settings.show_damage_numbers = trigger.data.checked;
                    },
                ),
        ))
        .with_key("damage_numbers")
        .with_child::<Element>(setting_row(
            "Invert Mouse Y",
            WidgetChildren::default()
                .with_child::<Toggle>((Toggle,))
                .with_observe(
                    current_widget,
                    |trigger: On<Change<ToggleChanged>>, mut settings: ResMut<GameSettings>| {
                        settings.invert_mouse_y = trigger.data.checked;
                    },
                ),
        ))
        .with_key("invert_y")
        .with_child::<Element>(section_divider())
        .with_key("divider3")
        .with_child::<Element>((
            Element,
            theme::section_title_style(),
            WidgetRender::Text {
                content: "Appearance".into(),
            },
        ))
        .with_key("appearance_title")
        .with_child::<Element>(setting_row(
            "Accent Color",
            WidgetChildren::default().with_child::<AccentColorSwatch>((AccentColorSwatch,)),
        ))
        .with_key("accent_color")
}

/// Per-swatch open/closed state -- click-toggled rather than hover, since the floating
/// `ColorPicker` needs to be interacted with (dragging its gradient tracks).
#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ColorSwatchState {
    open: bool,
}

/// A small color swatch that opens `woodpecker_ui`'s `ColorPicker` (320px wide -- too wide to
/// embed inline in a settings row) in a `Popover` on click, closing again on a second click.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(accent_color_swatch_render)]
#[require(WidgetChildren, WoodpeckerStyle, WatchedResource<GameSettings>)]
pub(crate) struct AccentColorSwatch;

const COLOR_PICKER_SIZE: Vec2 = Vec2::new(320.0, 340.0);

/// Picks whichever of `Popover`'s 4 fixed placements has enough room on both axes, based on
/// the swatch's last-computed screen position vs. the viewport. Falls back to `Left` if no
/// layout is available yet (e.g. the very first frame) or nothing fully fits.
fn best_placement(
    swatch_layout: Option<&WidgetLayout>,
    viewport_layout: Option<&WidgetLayout>,
) -> PopoverPlacement {
    let (Some(swatch), Some(viewport)) = (swatch_layout, viewport_layout) else {
        return PopoverPlacement::Left;
    };

    let space_above = swatch.location.y - viewport.location.y;
    let space_below = (viewport.location.y + viewport.size.y) - (swatch.location.y + swatch.size.y);
    let space_left = swatch.location.x - viewport.location.x;
    let space_right = (viewport.location.x + viewport.size.x) - (swatch.location.x + swatch.size.x);

    // (placement, growth-axis space available, growth-axis space needed,
    //             fixed-axis space available, fixed-axis space needed)
    let candidates = [
        (
            PopoverPlacement::Left,
            space_left,
            COLOR_PICKER_SIZE.x,
            space_below + swatch.size.y,
            COLOR_PICKER_SIZE.y,
        ),
        (
            PopoverPlacement::Right,
            space_right,
            COLOR_PICKER_SIZE.x,
            space_below + swatch.size.y,
            COLOR_PICKER_SIZE.y,
        ),
        (
            PopoverPlacement::Top,
            space_above,
            COLOR_PICKER_SIZE.y,
            space_right + swatch.size.x,
            COLOR_PICKER_SIZE.x,
        ),
        (
            PopoverPlacement::Bottom,
            space_below,
            COLOR_PICKER_SIZE.y,
            space_right + swatch.size.x,
            COLOR_PICKER_SIZE.x,
        ),
    ];

    candidates
        .iter()
        .find(
            |(_, growth_space, growth_needed, fixed_space, fixed_needed)| {
                growth_space >= growth_needed && fixed_space >= fixed_needed
            },
        )
        .or_else(|| {
            candidates
                .iter()
                .max_by(|a, b| (a.1 - a.2).partial_cmp(&(b.1 - b.2)).unwrap())
        })
        .map(|(placement, ..)| *placement)
        .unwrap_or(PopoverPlacement::Left)
}

fn accent_color_swatch_render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    layout_query: Query<&WidgetLayout>,
    // Queried via the `WoodpeckerApp` marker rather than `Res<WoodpeckerContext>`: a widget's
    // render system can't access that resource, since `runner.rs` holds it via a scoped
    // `&mut` reference outside the `World` for the whole render pass.
    root_layout_query: Query<&WidgetLayout, With<WoodpeckerApp>>,
    mut query: Query<(&WatchedResource<GameSettings>, &mut WidgetChildren)>,
    state_query: Query<&ColorSwatchState>,
) {
    let Ok((settings, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;

    let placement = best_placement(
        layout_query.get(*current_widget).ok(),
        root_layout_query.single().ok(),
    );

    let state_entity = hooks.use_state(&mut commands, current_widget, ColorSwatchState::default());
    let default_state = ColorSwatchState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    let mut content = WidgetChildren::default();
    content.add::<ColorPicker>((ColorPicker {
        initial_color: settings.accent_color,
    },));
    content.add_key("picker");
    content.observe(
        current_widget,
        |trigger: On<Change<ColorPickerChanged>>, mut settings: ResMut<GameSettings>| {
            settings.accent_color = trigger.data.color;
        },
    );

    *children = WidgetChildren::default()
        .with_child::<Popover>((
            Popover {
                visible: state.open,
                placement,
            },
            WidgetRender::Quad,
            Pickable::default(),
            WoodpeckerStyle {
                width: 60.0.into(),
                height: 30.0.into(),
                background_color: settings.accent_color,
                border: Edge::all(2.0),
                border_color: theme::BORDER_DARK,
                border_radius: Corner::all(theme::RADIUS_SM),
                ..Default::default()
            },
            PassedChildren(WidgetChildren::default()),
            PopoverContent(content),
        ))
        .with_key("popover")
        .with_observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut state_query: Query<&mut ColorSwatchState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                state.open = !state.open;
            },
        )
        .with_hover_cursor(current_widget, SystemCursorIcon::Pointer);

    children.apply(current_widget.as_parent());
}
