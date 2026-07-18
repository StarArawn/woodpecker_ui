use bevy::prelude::*;
use woodpecker_ui::prelude::*;

pub(super) fn demo_button() -> WidgetChildren {
    WidgetChildren::default().with_child::<WButton>(WButton::text("Click me"))
}

pub(super) fn demo_checkbox() -> WidgetChildren {
    WidgetChildren::default().with_child::<Checkbox>(Checkbox)
}

pub(super) fn demo_icon_button() -> WidgetChildren {
    WidgetChildren::default().with_child::<IconButton>((
        IconButton,
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..Default::default()
            },
            WidgetRender::Text {
                content: "+".into(),
            },
        )),
    ))
}

/// Per-instance state for [`ToggleButtonDemo`] -- a dedicated mini-widget (rather than inline
/// `hooks.use_state` in `demo_toggle_button`) since reading the state back to decide each
/// `ToggleButton::selected` needs a `Query`, which `demo_x()` functions don't have; mirrors
/// `examples/toggle_button.rs`'s own shape exactly.
#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ToggleButtonDemoState {
    bold: bool,
    italic: bool,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_toggle_button_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct ToggleButtonDemo;

fn render_toggle_button_demo(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<ToggleButtonDemo>>,
    state_query: Query<&ToggleButtonDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        ToggleButtonDemoState::default(),
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let state = *state;
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children
        .add::<ToggleButton>((
            ToggleButton {
                selected: state.bold,
            },
            PassedChildren(WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle::default(),
                WidgetRender::Text {
                    content: "Bold".into(),
                },
            ))),
        ))
        .observe(
            current_widget,
            move |trigger: On<Change<ToggleButtonChanged>>,
                  mut query: Query<&mut ToggleButtonDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.bold = trigger.data.selected;
                }
            },
        );
    children.add_key("bold");
    children
        .add::<ToggleButton>((
            ToggleButton {
                selected: state.italic,
            },
            PassedChildren(WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle::default(),
                WidgetRender::Text {
                    content: "Italic".into(),
                },
            ))),
        ))
        .observe(
            current_widget,
            move |trigger: On<Change<ToggleButtonChanged>>,
                  mut query: Query<&mut ToggleButtonDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.italic = trigger.data.selected;
                }
            },
        );
    children.add_key("italic");

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_toggle_button() -> WidgetChildren {
    WidgetChildren::default().with_child::<ToggleButtonDemo>(ToggleButtonDemo)
}

pub(super) fn demo_speed_dial() -> WidgetChildren {
    WidgetChildren::default().with_child::<SpeedDial>((SpeedDial {
        icon: "+".into(),
        actions: vec![
            SpeedDialAction {
                icon: "\u{1F4C4}".into(),
            },
            SpeedDialAction {
                icon: "\u{1F4C1}".into(),
            },
            SpeedDialAction {
                icon: "\u{2B06}".into(),
            },
        ],
        placement: PopoverPlacement::Top,
    },))
}

pub(super) fn demo_link() -> WidgetChildren {
    WidgetChildren::default().with_child::<Link>((Link {
        label: "View full changelog".into(),
    },))
}

pub(super) fn demo_radio() -> WidgetChildren {
    WidgetChildren::default().with_child::<RadioGroup>((RadioGroup {
        options: vec!["Small".into(), "Medium".into(), "Large".into()],
        selected: 1,
        horizontal: true,
    },))
}

pub(super) fn demo_toggle() -> WidgetChildren {
    WidgetChildren::default().with_child::<Toggle>(Toggle)
}

pub(super) fn demo_slider() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 240.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Slider>(Slider {
            start: 0.0,
            end: 1.0,
            value: 0.5,
        }),
    ))
}

pub(super) fn demo_combo_box() -> WidgetChildren {
    let languages = vec!["Rust", "Zig", "C", "C++", "Go", "Python", "TypeScript"]
        .into_iter()
        .map(String::from)
        .collect();
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 240.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<ComboBox>(ComboBox {
            current_value: "Rust".into(),
            list: languages,
            match_mode: ComboBoxMatchMode::Contains,
        }),
    ))
}

pub(super) fn demo_dropdown() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 200.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Dropdown>(Dropdown {
            list: vec!["Red".into(), "Green".into(), "Blue".into()],
            current_value: "Red".into(),
        }),
    ))
}

pub(super) fn demo_date_picker() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 260.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<DatePicker>(DatePicker {
            selected_date: None,
            min_date: Some(CalendarDate::new(2024, 1, 1)),
            max_date: Some(CalendarDate::new(2024, 12, 31)),
            initial_view: CalendarDate::new(2024, 6, 1),
            placeholder: "Pick a date".into(),
        }),
    ))
}

pub(super) fn demo_number_input() -> WidgetChildren {
    WidgetChildren::default().with_child::<NumberInput>((NumberInput {
        value: 5.0,
        min: 0.0,
        max: 20.0,
        step: 1.0,
        ..Default::default()
    },))
}

pub(super) fn demo_text_box() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 240.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<TextBox>(TextBox::default()),
    ))
}

pub(super) fn demo_tabs() -> WidgetChildren {
    let mut tab_buttons = WidgetChildren::default();
    let mut tab_content = WidgetChildren::default();
    for i in 0..3 {
        tab_buttons.add::<TabButton>(TabButtonBundle {
            tab_button: TabButton {
                index: i,
                title: format!("Tab {}", i + 1),
            },
            ..Default::default()
        });
        tab_buttons.add_key(format!("tab-{i}"));
        tab_content.add::<TabContent>(TabContentBundle {
            tab_content: TabContent { index: i },
            children: PassedChildren(WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 14.0,
                    color: Srgba::WHITE.into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("Content for tab {}.", i + 1),
                },
            ))),
            ..Default::default()
        });
        tab_content.add_key(format!("tab-{i}"));
    }

    WidgetChildren::default().with_child::<TabContextProvider>((TabContextProviderBundle {
        children: PassedChildren(
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        flex_direction: WidgetFlexDirection::Row,
                        ..Default::default()
                    },
                    tab_buttons,
                ))
                .with_key("tab-buttons-row")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: 320.0.into(),
                        ..Default::default()
                    },
                    tab_content,
                ))
                .with_key("tab-content"),
        ),
        ..Default::default()
    },))
}

pub(super) fn demo_bottom_navigation() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 320.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<BottomNavigation>(BottomNavigation {
            items: vec![
                BottomNavItem {
                    icon: icons::HOUSE.into(),
                    label: "Home".into(),
                },
                BottomNavItem {
                    icon: icons::MAGNIFYING_GLASS.into(),
                    label: "Search".into(),
                },
                BottomNavItem {
                    icon: icons::STAR.into(),
                    label: "Saved".into(),
                },
                BottomNavItem {
                    icon: icons::GEAR.into(),
                    label: "Settings".into(),
                },
            ],
            ..Default::default()
        }),
    ))
}

pub(super) fn demo_navigation_rail() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            height: 320.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<NavigationRail>(NavigationRail {
            items: vec![
                NavigationRailItem {
                    icon: icons::HOUSE.into(),
                    label: "Home".into(),
                },
                NavigationRailItem {
                    icon: icons::MAGNIFYING_GLASS.into(),
                    label: "Search".into(),
                },
                NavigationRailItem {
                    icon: icons::STAR.into(),
                    label: "Saved".into(),
                },
                NavigationRailItem {
                    icon: icons::GEAR.into(),
                    label: "Settings".into(),
                },
            ],
            ..Default::default()
        }),
    ))
}

pub(super) fn demo_breadcrumbs() -> WidgetChildren {
    WidgetChildren::default().with_child::<Breadcrumbs>((Breadcrumbs {
        items: vec![
            "Settings".into(),
            "Billing".into(),
            "Invoices".into(),
            "INV-2049".into(),
        ],
    },))
}

/// Mirrors `examples/pagination.rs`'s own `PaginationDemo` shape -- `Pagination` has no
/// internal fallback state (the caller owns `page`), so the demo needs its own mini-widget to
/// read the current page back out of a `Query` when re-rendering.
#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct PaginationDemoState {
    page: usize,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_pagination_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct PaginationDemo;

fn render_pagination_demo(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<PaginationDemo>>,
    state_query: Query<&PaginationDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        PaginationDemoState::default(),
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let page = state.page;
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 13.0,
            color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
            margin: Edge::all(0.0).bottom(12.0),
            ..Default::default()
        },
        WidgetRender::Text {
            content: format!("Page {} of 12", page + 1),
        },
    ));
    children.add_key("label");

    children
        .add::<Pagination>((Pagination {
            page,
            page_count: 12,
        },))
        .observe(
            current_widget,
            move |trigger: On<Change<PaginationChanged>>,
                  mut query: Query<&mut PaginationDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.page = trigger.data.page;
                }
            },
        );
    children.add_key("pagination");

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_pagination() -> WidgetChildren {
    WidgetChildren::default().with_child::<PaginationDemo>(PaginationDemo)
}

const STEPPER_STEPS: [&str; 4] = ["Account", "Payment", "Review", "Done"];

/// Mirrors `examples/stepper.rs`'s own `StepperDemo` shape -- `Stepper` has no internal
/// fallback state (the caller owns `active`), same reasoning as [`PaginationDemo`].
#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct StepperDemoState {
    active: usize,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_stepper_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct StepperDemo;

fn render_stepper_demo(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(&mut WoodpeckerStyle, &mut WidgetChildren), With<StepperDemo>>,
    state_query: Query<&StepperDemoState>,
) {
    let Ok((mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity =
        hooks.use_state(&mut commands, *current_widget, StepperDemoState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let active = state.active;
    let current_widget = *current_widget;

    styles.flex_direction = WidgetFlexDirection::Column;

    *children = WidgetChildren::default();
    children
        .add::<Stepper>((Stepper {
            steps: STEPPER_STEPS.iter().map(|s| s.to_string()).collect(),
            active,
            orientation: StepperOrientation::Horizontal,
            clickable: true,
        },))
        .observe(
            current_widget,
            move |trigger: On<Change<StepperChanged>>, mut query: Query<&mut StepperDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.active = trigger.data.step;
                }
            },
        );
    children.add_key("stepper");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: 420.0.into(),
            flex_direction: WidgetFlexDirection::Row,
            justify_content: Some(WidgetAlignContent::SpaceBetween),
            margin: Edge::all(0.0).top(24.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<WButton>(WButton::text("Back"))
            .with_observe(
                current_widget,
                move |_trigger: On<Pointer<Click>>, mut query: Query<&mut StepperDemoState>| {
                    if let Ok(mut state) = query.get_mut(state_entity) {
                        state.active = state.active.saturating_sub(1);
                    }
                },
            )
            .with_key("back")
            .with_child::<WButton>(WButton::text("Next"))
            .with_observe(
                current_widget,
                move |_trigger: On<Pointer<Click>>, mut query: Query<&mut StepperDemoState>| {
                    if let Ok(mut state) = query.get_mut(state_entity) {
                        state.active = (state.active + 1).min(STEPPER_STEPS.len() - 1);
                    }
                },
            )
            .with_key("next"),
    ));
    children.add_key("actions");

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_stepper() -> WidgetChildren {
    WidgetChildren::default().with_child::<StepperDemo>(StepperDemo)
}

