use bevy::prelude::*;
use woodpecker_ui::prelude::*;

pub(super) fn demo_avatar() -> WidgetChildren {
    WidgetChildren::default()
        .with_child::<Avatar>((Avatar {
            initials: "AT".into(),
            color: None,
            size: 40.0,
        },))
        .with_key("default")
        .with_child::<Avatar>((Avatar {
            initials: "JP".into(),
            color: Some(Srgba::new(0.3, 0.6, 0.4, 1.0).into()),
            size: 40.0,
        },))
        .with_key("custom-color")
}

pub(super) fn demo_badge() -> WidgetChildren {
    WidgetChildren::default()
        .with_child::<Badge>((Badge {
            label: "New".into(),
            variant: BadgeVariant::Info,
        },))
        .with_key("info")
        .with_child::<Badge>((Badge {
            label: "Active".into(),
            variant: BadgeVariant::Success,
        },))
        .with_key("success")
        .with_child::<Badge>((Badge {
            label: "Offline".into(),
            variant: BadgeVariant::Danger,
        },))
        .with_key("danger")
}

/// Mirrors `examples/chip.rs`'s own `ChipDemo` shape -- `Chip` "has no internal selection
/// state of its own" (see its own doc comment), same reasoning as [`PaginationDemo`].
#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ChipDemoState {
    filters_selected: Vec<bool>,
}

impl ChipDemoState {
    fn initial() -> Self {
        Self {
            filters_selected: vec![true, false, false],
        }
    }
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_chip_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct ChipDemo;

const CHIP_FILTERS: [&str; 3] = ["Design", "Backend", "Urgent"];

fn render_chip_demo(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<ChipDemo>>,
    state_query: Query<&ChipDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity = hooks.use_state(&mut commands, *current_widget, ChipDemoState::initial());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let state = state.clone();
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    for (index, label) in CHIP_FILTERS.iter().enumerate() {
        children
            .add::<Chip>((Chip {
                label: (*label).into(),
                variant: BadgeVariant::Info,
                selected: state.filters_selected[index],
                ..Default::default()
            },))
            .observe(
                current_widget,
                move |_trigger: On<Change<ChipClicked>>, mut query: Query<&mut ChipDemoState>| {
                    if let Ok(mut state) = query.get_mut(state_entity) {
                        state.filters_selected[index] = !state.filters_selected[index];
                    }
                },
            );
        children.add_key(*label);
    }

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_chip() -> WidgetChildren {
    WidgetChildren::default().with_child::<ChipDemo>(ChipDemo)
}

pub(super) fn demo_list() -> WidgetChildren {
    let entries = [
        ("Ava Thompson deployed billing-service", "2m ago"),
        ("Marcus Chen merged auth-middleware", "18m ago"),
        ("Priya Patel commented on #482", "3h ago"),
    ];
    let mut rows = WidgetChildren::default();
    for (index, (text, time_ago)) in entries.iter().enumerate() {
        rows.add::<ListItem>((
            ListItem {
                primary_text: (*text).into(),
                selected: index == 0,
                ..Default::default()
            },
            ListItemTrailing(WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 12.0,
                    color: Srgba::new(0.6, 0.63, 0.7, 1.0).into(),
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: (*time_ago).into(),
                },
            ))),
        ));
        rows.add_key(*text);
    }

    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 380.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<List>((List, PassedChildren(rows))),
    ))
}

pub(super) fn demo_table() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 420.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Table>((Table {
            columns: vec![
                TableColumn::flexible("Name"),
                TableColumn::fixed("Status", 90.0),
            ],
            rows: vec![
                TableRow::from(vec!["Ava Thompson".to_string(), "Active".to_string()]),
                TableRow::from(vec!["Marcus Chen".to_string(), "Away".to_string()]),
                TableRow::from(vec!["Priya Patel".to_string(), "Active".to_string()]),
            ]
            .into(),
            ..Default::default()
        },)),
    ))
}

pub(super) fn demo_chart() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 420.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<LineChart>((LineChart {
            series: vec![
                ChartSeries::new(
                    "Revenue",
                    vec![10.0, 14.0, 12.0, 18.0, 22.0, 20.0],
                    Srgba::new(0.35, 0.55, 0.9, 1.0).into(),
                ),
                ChartSeries::new(
                    "Costs",
                    vec![8.0, 9.0, 10.0, 11.0, 12.0, 13.0],
                    Srgba::new(0.9, 0.4, 0.35, 1.0).into(),
                ),
            ],
            show_legend: true,
        },)),
    ))
}

pub(super) fn demo_bar_chart() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 420.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<BarChart>((BarChart {
            categories: vec![
                "Mon".into(),
                "Tue".into(),
                "Wed".into(),
                "Thu".into(),
                "Fri".into(),
            ],
            series: vec![
                BarSeries::new(
                    "Revenue",
                    vec![12.0, 18.0, 14.0, 22.0, 20.0],
                    Srgba::new(0.35, 0.55, 0.9, 1.0).into(),
                ),
                BarSeries::new(
                    "Costs",
                    vec![8.0, 9.0, 10.0, 11.0, 12.0],
                    Srgba::new(0.9, 0.4, 0.35, 1.0).into(),
                ),
            ],
            mode: BarMode::Grouped,
            show_legend: true,
        },)),
    ))
}

pub(super) fn demo_area_chart() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 420.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<AreaChart>((AreaChart {
            series: vec![
                AreaSeries::new(
                    "Organic",
                    vec![10.0, 14.0, 12.0, 18.0, 22.0, 20.0],
                    Srgba::new(0.35, 0.55, 0.9, 1.0).into(),
                ),
                AreaSeries::new(
                    "Paid",
                    vec![4.0, 6.0, 5.0, 8.0, 9.0, 10.0],
                    Srgba::new(0.4, 0.8, 0.5, 1.0).into(),
                ),
            ],
            show_legend: true,
        },)),
    ))
}

pub(super) fn demo_timeline() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 320.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Timeline>((Timeline {
            items: vec![
                TimelineItem {
                    title: "Order placed".into(),
                    subtitle: Some("Jul 5, 9:12 AM".into()),
                    dot_color: None,
                },
                TimelineItem {
                    title: "Payment confirmed".into(),
                    subtitle: Some("Jul 5, 9:13 AM".into()),
                    dot_color: None,
                },
                TimelineItem {
                    title: "Payment failed, retrying".into(),
                    subtitle: Some("Jul 5, 11:02 AM".into()),
                    dot_color: Some(Srgba::new(0.9, 0.3, 0.3, 1.0).into()),
                },
                TimelineItem {
                    title: "Shipped".into(),
                    subtitle: Some("Jul 6, 2:45 PM".into()),
                    dot_color: None,
                },
            ],
        },)),
    ))
}

pub(super) fn demo_tree_view() -> WidgetChildren {
    let nodes = vec![
        TreeNode {
            key: "src".into(),
            label: "src".into(),
            children: vec![
                TreeNode {
                    key: "src/lib.rs".into(),
                    label: "lib.rs".into(),
                    children: vec![],
                },
                TreeNode {
                    key: "src/widgets".into(),
                    label: "widgets".into(),
                    children: vec![TreeNode {
                        key: "src/widgets/button.rs".into(),
                        label: "button.rs".into(),
                        children: vec![],
                    }],
                },
            ],
        },
        TreeNode {
            key: "Cargo.toml".into(),
            label: "Cargo.toml".into(),
            children: vec![],
        },
    ];

    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 260.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<TreeView>(TreeView {
            nodes,
            selected_key: Some("src/lib.rs".into()),
            indent: 18.0,
            ..Default::default()
        }),
    ))
}

pub(super) fn demo_typography() -> WidgetChildren {
    WidgetChildren::default()
        .with_child::<Typography>((Typography {
            text: "Heading 1".into(),
            variant: TypographyVariant::H1,
            color: None,
        },))
        .with_key("h1")
        .with_child::<Typography>((Typography {
            text: "Heading 2".into(),
            variant: TypographyVariant::H2,
            color: None,
        },))
        .with_key("h2")
        .with_child::<Typography>((Typography {
            text: "Body text, the default variant for ordinary copy.".into(),
            variant: TypographyVariant::Body,
            color: None,
        },))
        .with_key("body")
        .with_child::<Typography>((Typography {
            text: "Caption, the smallest legible size.".into(),
            variant: TypographyVariant::Caption,
            color: None,
        },))
        .with_key("caption")
}

pub(super) fn demo_markdown() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 480.0.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Markdown>((Markdown {
            content: "## Release notes\n\nSupports **fenced code**, lists, and quotes:\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\n- Headings\n- Paragraphs\n- Code blocks\n\n> A block quote renders as a callout.".into(),
            ..Default::default()
        },)),
    ))
}

pub(super) fn demo_divider() -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 260.0.into(),
            flex_direction: WidgetFlexDirection::Column,
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
                    margin: Edge::all(0.0).bottom(10.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Section one".into(),
                },
            ))
            .with_key("above")
            .with_child::<Divider>(Divider::default())
            .with_key("divider")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
                    margin: Edge::all(0.0).top(10.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Section two".into(),
                },
            ))
            .with_key("below"),
    ))
}

pub(super) fn demo_tooltip() -> WidgetChildren {
    WidgetChildren::default().with_child::<Tooltip>((
        Tooltip {
            text: "This action can't be undone".into(),
            placement: PopoverPlacement::Top,
        },
        PassedChildren(WidgetChildren::default().with_child::<WButton>(WButton::text("Hover me"))),
    ))
}

/// Mirrors `examples/rating.rs`'s own `RatingDemo` shape -- `Rating` has no internal fallback
/// state (the caller owns `value`), same reasoning as [`PaginationDemo`].
#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct RatingDemoState {
    value: f32,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_rating_demo)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub(crate) struct RatingDemo;

fn render_rating_demo(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<&mut WidgetChildren, With<RatingDemo>>,
    state_query: Query<&RatingDemoState>,
) {
    let Ok(mut children) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity = hooks.use_state(&mut commands, *current_widget, RatingDemoState { value: 3.5 });
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let value = state.value;
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children
        .add::<Rating>((Rating {
            value,
            max: 5,
            read_only: false,
        },))
        .observe(
            current_widget,
            move |trigger: On<Change<RatingChanged>>, mut query: Query<&mut RatingDemoState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.value = trigger.data.value;
                }
            },
        );
    children.add_key("rating");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 13.0,
            color: Srgba::new(0.72, 0.75, 0.8, 1.0).into(),
            margin: Edge::all(0.0).top(8.0),
            ..Default::default()
        },
        WidgetRender::Text {
            content: format!("{value} / 5"),
        },
    ));
    children.add_key("label");

    children.apply(current_widget.as_parent());
}

pub(super) fn demo_rating() -> WidgetChildren {
    WidgetChildren::default().with_child::<RatingDemo>(RatingDemo)
}

pub(super) fn alert_text(content: &str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            text_wrap: TextWrap::WordOrGlyph,
            ..Default::default()
        },
        WidgetRender::Text {
            content: content.into(),
        },
    )
}

