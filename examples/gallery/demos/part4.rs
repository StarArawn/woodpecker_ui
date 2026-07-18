use bevy::prelude::*;
use woodpecker_ui::prelude::*;

/// Mirrors `settings_panel.rs`'s `ColorSwatchState` pattern -- `Popover::visible` is
/// caller-owned (see its own doc comment: "the caller owns `visible` and toggles it
/// themselves"), same reasoning as [`DrawerDemo`].
#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct PopoverDemoState {
    open: bool,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_popover_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct PopoverDemo;

fn render_popover_demo(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<PopoverDemo>>,
    state_query: Query<&PopoverDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity =
        hooks.use_state(&mut commands, *current_widget, PopoverDemoState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let open = state.open;
    let current_widget = *current_widget;

    let mut content = WidgetChildren::default();
    content.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: 180.0.into(),
            padding: Edge::all(4.0),
            ..Default::default()
        },
        WidgetRender::Text {
            content: "Popover content -- click the trigger again to close.".into(),
        },
    ));
    content.add_key("text");

    *children = WidgetChildren::default();
    children
        .add::<Popover>((
            PopoverBundle {
                popover: Popover {
                    visible: open,
                    placement: PopoverPlacement::Bottom,
                },
                trigger: PassedChildren(
                    WidgetChildren::default().with_child::<WButton>(WButton::text("Toggle popover")),
                ),
                content: PopoverContent(content),
                ..Default::default()
            },
        ))
        .observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut query: Query<&mut PopoverDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.open = !state.open;
                }
            },
        );
    children.add_key("popover");

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_popover() -> WidgetChildren {
    WidgetChildren::default().with_child::<PopoverDemo>(PopoverDemo)
}

pub(super) fn demo_menu() -> WidgetChildren {
    WidgetChildren::default().with_child::<Menu>((
        Menu {
            items: vec!["Cut".into(), "Copy".into(), "Paste".into(), "Delete".into()],
        },
        PassedChildren(WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: 260.0.into(),
                height: 160.0.into(),
                background_color: Color::srgb(0.16, 0.18, 0.22),
                justify_content: Some(WidgetAlignContent::Center),
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            WidgetRender::Quad,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    color: Color::WHITE,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Right-click me".into(),
                },
            )),
        ))),
    ))
}

/// `Masonry`/`ImageList` need a real `Handle<Image>`, which plain `demo_x() -> WidgetChildren`
/// functions can't load themselves (no `Res<AssetServer>` available) -- a dedicated mini-
/// widget's own `render` system can take one as a normal system param instead.
#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_masonry_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct MasonryDemo;

fn render_masonry_demo(
    current_widget: Res<CurrentWidget>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut WidgetChildren, With<MasonryDemo>>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let handle: Handle<Image> = asset_server.load("woodpecker.jpg");
    let heights = [140.0, 90.0, 180.0, 110.0, 70.0, 150.0];
    let items = heights
        .iter()
        .enumerate()
        .map(|(i, &height)| MasonryItem {
            handle: handle.clone(),
            height,
            label: Some(format!("Photo {}", i + 1)),
        })
        .collect();

    *children = WidgetChildren::default().with_child::<Masonry>((Masonry {
        items,
        columns: 3,
        column_width: 100.0,
        gap: 8.0,
    },));
    children.apply(current_widget.as_parent());
}

pub(super) fn demo_masonry() -> WidgetChildren {
    WidgetChildren::default().with_child::<MasonryDemo>(MasonryDemo)
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_image_list_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct ImageListDemo;

fn render_image_list_demo(
    current_widget: Res<CurrentWidget>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut WidgetChildren, With<ImageListDemo>>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let handle: Handle<Image> = asset_server.load("woodpecker.jpg");
    let items = (1..=6)
        .map(|i| ImageListItem {
            handle: handle.clone(),
            label: Some(format!("Photo {i}")),
        })
        .collect();

    *children = WidgetChildren::default().with_child::<ImageList>((ImageList {
        items,
        columns: 3,
        tile_size: 90.0,
        gap: 8.0,
    },));
    children.apply(current_widget.as_parent());
}

pub(super) fn demo_image_list() -> WidgetChildren {
    WidgetChildren::default().with_child::<ImageListDemo>(ImageListDemo)
}

fn virtual_list_item_content(index: usize) -> WidgetChildren {
    let (badge_label, badge_color) = if index.is_multiple_of(7) {
        ("HOT", Srgba::new(0.85, 0.25, 0.25, 1.0).into())
    } else {
        ("NEW", colors::PRIMARY)
    };

    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            align_items: Some(WidgetAlignItems::Center),
            padding: Edge::all(0.0).left(16.0).right(16.0),
            gap: (12.0.into(), 0.0.into()),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Pixels(48.0),
                    height: Units::Pixels(20.0),
                    justify_content: Some(WidgetAlignContent::Center),
                    align_items: Some(WidgetAlignItems::Center),
                    background_color: badge_color,
                    border_radius: Corner::all(4.0),
                    ..Default::default()
                },
                WidgetRender::Quad,
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 10.0,
                        color: Srgba::WHITE.into(),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: badge_label.into(),
                    },
                )),
            ))
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 14.0,
                    color: Srgba::WHITE.into(),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("Item #{index}"),
                },
            )),
    ))
}

pub(super) fn demo_virtual_list() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 320.0.into(),
            height: 240.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<VirtualList>(VirtualList {
            item_count: 10_000,
            item_extent: 32.0,
            item_content: VirtualListItemContent::new(virtual_list_item_content),
        }),
    ))
}

/// Mirrors `examples/transfer_list.rs`'s own `TransferListDemo` shape -- `TransferList` owns
/// no state of its own (the caller owns `left`/`right`), same reasoning as [`PaginationDemo`].
#[derive(Component, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct TransferListDemoState {
    left: Vec<String>,
    right: Vec<String>,
}

impl Default for TransferListDemoState {
    fn default() -> Self {
        Self {
            left: vec!["Admin".into(), "Editor".into(), "Viewer".into()],
            right: vec!["Support".into()],
        }
    }
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_transfer_list_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct TransferListDemo;

fn render_transfer_list_demo(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<TransferListDemo>>,
    state_query: Query<&TransferListDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        TransferListDemoState::default(),
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let state = state.clone();
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children
        .add::<TransferList>((TransferList {
            left: state.left,
            right: state.right,
            left_title: "Available roles".into(),
            right_title: "Assigned roles".into(),
        },))
        .observe(
            current_widget,
            move |trigger: On<Change<TransferListChanged>>,
                  mut query: Query<&mut TransferListDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.left = trigger.data.left.clone();
                    state.right = trigger.data.right.clone();
                }
            },
        );
    children.add_key("transfer");

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_transfer_list() -> WidgetChildren {
    WidgetChildren::default().with_child::<TransferListDemo>(TransferListDemo)
}

/// Per-instance pane width state for [`SplitterDemo`] -- `Splitter` only reports a drag delta
/// (see its own `SplitterChanged::delta` doc comment), so the caller (here, this mini-widget)
/// has to own the actual pane width itself, seeded once and then updated on each drag.
#[derive(Component, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct SplitterDemoState {
    left_width: f32,
    left_width_at_drag_start: f32,
}

impl Default for SplitterDemoState {
    fn default() -> Self {
        Self {
            left_width: 160.0,
            left_width_at_drag_start: 160.0,
        }
    }
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_splitter_demo)]
#[require(WoodpeckerStyle = splitter_demo_style(), WidgetChildren)]
pub(crate) struct SplitterDemo;

fn splitter_demo_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: 420.0.into(),
        height: 160.0.into(),
        flex_direction: WidgetFlexDirection::Row,
        ..Default::default()
    }
}

fn splitter_pane(label: &str, width: WoodpeckerStyle, background_color: Color) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            height: Units::Percentage(100.0),
            background_color,
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..width
        },
        WidgetRender::Quad,
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                color: Color::WHITE,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: label.into(),
            },
        )),
    )
}

fn render_splitter_demo(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<SplitterDemo>>,
    state_query: Query<&SplitterDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        SplitterDemoState::default(),
    );
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let left_width = state.left_width;
    let current_widget = *current_widget;

    *children = WidgetChildren::default()
        .with_child::<Element>(splitter_pane(
            "Left",
            WoodpeckerStyle {
                width: left_width.into(),
                ..Default::default()
            },
            Color::srgb(0.16, 0.18, 0.22),
        ))
        .with_key("left")
        .with_child::<Splitter>(Splitter::default())
        .with_key("splitter")
        .with_observe(
            current_widget,
            move |_trigger: On<Pointer<DragStart>>, mut query: Query<&mut SplitterDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.left_width_at_drag_start = state.left_width;
                }
            },
        )
        .with_observe(
            current_widget,
            move |trigger: On<Change<SplitterChanged>>,
                  mut query: Query<&mut SplitterDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    let base = state.left_width_at_drag_start;
                    state.left_width = (base + trigger.data.delta).clamp(80.0, 300.0);
                }
            },
        )
        .with_child::<Element>(splitter_pane(
            "Right -- drag the divider",
            WoodpeckerStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
            Color::srgb(0.10, 0.11, 0.14),
        ))
        .with_key("right");

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_splitter() -> WidgetChildren {
    WidgetChildren::default().with_child::<SplitterDemo>(SplitterDemo)
}

pub(super) fn demo_color_picker() -> WidgetChildren {
    WidgetChildren::default().with_child::<ColorPicker>((ColorPicker {
        initial_color: Color::Srgba(Srgba::new(0.35, 0.55, 0.9, 1.0)),
    },))
}

pub(super) fn demo_window() -> WidgetChildren {
    WidgetChildren::default().with_child::<WindowingContextProvider>(
        WidgetChildren::default().with_child::<WoodpeckerWindow>((
            WoodpeckerWindow {
                title: "Example window".into(),
                initial_position: Vec2::new(40.0, 40.0),
                window_styles: WoodpeckerStyle {
                    min_width: 260.0.into(),
                    ..WoodpeckerWindow::default().window_styles
                },
                ..Default::default()
            },
            PassedChildren(WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    padding: Edge::all(10.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "This is a draggable, resizable window.".into(),
                },
            ))),
        )),
    )
}

