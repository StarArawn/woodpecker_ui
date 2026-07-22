use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::mock_data::ActivityEntry;
use crate::theme;

/// Per-widget state for [`MrrInfoPopover`] -- owns its own open/closed state (seeded closed),
/// the same "toggle on click" pattern `SpeedDial` uses, since [`Popover`] itself is purely
/// caller-controlled (`visible` is just a plain field, no internal state of its own).
#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct MrrInfoState {
    open: bool,
}

/// A small "i" info-icon that expands a rich, multi-line [`Popover`] explaining how MRR is
/// calculated -- distinct from [`Tooltip`] (used elsewhere in the dashboard for a single line
/// of plain hover text): this needs several lines of formatted content, which only `Popover`'s
/// arbitrary-content model supports.
#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_mrr_info)]
#[require(WoodpeckerStyle, WidgetChildren, WatchedResource<Theme>)]
pub struct MrrInfoPopover;

fn render_mrr_info(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    icon_font: Res<IconFont>,
    mut query: Query<(&WatchedResource<Theme>, &mut WidgetChildren), With<MrrInfoPopover>>,
    state_query: Query<&MrrInfoState>,
) {
    let Ok((watched_theme, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let theme = watched_theme.0;

    let state_entity = hooks.use_state(&mut commands, *current_widget, MrrInfoState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let open = state.open;
    let current_widget = *current_widget;

    let help_line = |text: &str| -> WidgetChildren {
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 12.0,
                color: theme::text_secondary(&theme),
                text_wrap: TextWrap::WordOrGlyph,
                margin: Edge::all(0.0).bottom(4.0),
                ..Default::default()
            },
            WidgetRender::Text {
                content: text.into(),
            },
        ))
    };

    *children = WidgetChildren::default();
    children
        .add::<Popover>((PopoverBundle {
            popover: Popover {
                visible: open,
                placement: PopoverPlacement::Bottom,
            },
            trigger: PassedChildren(WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: 16.0.into(),
                    height: 16.0.into(),
                    background_color: theme::bg_card_hover(&theme),
                    border_radius: Corner::all(100.0),
                    justify_content: Some(WidgetAlignContent::Center),
                    align_items: Some(WidgetAlignItems::Center),
                    ..Default::default()
                },
                WidgetRender::Quad,
                Pickable::default(),
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 11.0,
                        color: theme::text_muted(&theme),
                        font: Some(icon_font.0.id()),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: icons::INFO.into(),
                    },
                )),
            ))),
            content: PopoverContent(
                WidgetChildren::default()
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            width: 220.0.into(),
                            flex_direction: WidgetFlexDirection::Column,
                            ..Default::default()
                        },
                        WidgetChildren::default()
                            .with_child::<Element>((
                                Element,
                                WoodpeckerStyle {
                                    font_size: 13.0,
                                    color: theme::text_primary(&theme),
                                    margin: Edge::all(0.0).bottom(6.0),
                                    ..Default::default()
                                },
                                WidgetRender::Text {
                                    content: "How MRR is calculated".into(),
                                },
                            ))
                            .with_key("title")
                            .with_child::<Element>(help_line(
                                "Sum of every active subscription's monthly value.",
                            ))
                            .with_key("l1")
                            .with_child::<Element>(help_line(
                                "Annual plans are divided by 12 before being added.",
                            ))
                            .with_key("l2")
                            .with_child::<Element>(help_line(
                                "Excludes one-time invoices and usage overages.",
                            ))
                            .with_key("l3"),
                    ))
                    .with_key("content"),
            ),
            ..Default::default()
        },))
        .observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut state_query: Query<&mut MrrInfoState>| {
                if let Ok(mut state) = state_query.get_mut(state_entity) {
                    state.open = !state.open;
                }
            },
        );
    children.add_key("popover");

    children.apply(current_widget.as_parent());
}

struct Stat {
    label: &'static str,
    value: &'static str,
    delta: &'static str,
    positive: bool,
}

const STATS: [Stat; 4] = [
    Stat {
        label: "Monthly Recurring Revenue",
        value: "$482,910",
        delta: "+4.2%",
        positive: true,
    },
    Stat {
        label: "Active Users",
        value: "18,204",
        delta: "+1.8%",
        positive: true,
    },
    Stat {
        label: "Churn Rate",
        value: "2.4%",
        delta: "+0.3%",
        positive: false,
    },
    Stat {
        label: "Open Support Tickets",
        value: "37",
        delta: "-12%",
        positive: true,
    },
];

fn stat_card(theme: &Theme, stat: &Stat) -> impl Bundle + Clone {
    let delta_color = if stat.positive {
        theme::success(theme)
    } else {
        theme::danger(theme)
    };
    (
        Element,
        WoodpeckerStyle {
            // No explicit `width` -- this is a Row parent (the stat card strip), so
            // `flex_grow: 1.0` alone correctly shares the row's width evenly across cards.
            flex_grow: 1.0,
            ..theme::card_style(theme)
        },
        WidgetRender::Quad,
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: theme::text_secondary(theme),
                    margin: Edge::all(0.0).bottom(theme::space_sm(theme)),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: stat.label.into(),
                },
            ))
            .with_key("label")
            .with_child::<Typography>((
                Typography {
                    text: stat.value.into(),
                    variant: TypographyVariant::H1,
                    color: Some(theme::text_primary(theme)),
                },
                WoodpeckerStyle {
                    margin: Edge::all(0.0).bottom(theme::space_xs(theme)),
                    ..Default::default()
                },
            ))
            .with_key("value")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: delta_color,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("{} vs last month", stat.delta),
                },
            ))
            .with_key("delta"),
    )
}

fn activity_row(theme: &Theme, entry: &ActivityEntry) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            padding: Edge::all(theme::space_sm(theme)).left(0.0).right(0.0),
            border: Edge::all(0.0).bottom(1.0),
            border_color: theme::border(theme),
            // Row is this crate's default flex direction -- without this, the "text" and
            // "time" children below just pack against each other with no gap.
            justify_content: Some(WidgetAlignContent::SpaceBetween),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        },
        WidgetRender::Quad,
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: theme::text_primary(theme),
                    text_wrap: TextWrap::None,
                    margin: Edge::all(0.0).right(theme::space_md(theme)),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("{} {} {}", entry.actor, entry.action, entry.target),
                },
            ))
            .with_key("text")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 12.0,
                    color: theme::text_muted(theme),
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: entry.time_ago.clone(),
                },
            ))
            .with_key("time"),
    )
}

pub fn build_overview(
    theme: &Theme,
    current_widget: CurrentWidget,
    activity: &[ActivityEntry],
    revenue_this_year: &[f32],
    revenue_last_year: &[f32],
) -> WidgetChildren {
    let mut stat_row = WidgetChildren::default();
    for stat in STATS.iter() {
        stat_row.add::<Element>(stat_card(theme, stat));
        stat_row.add_key(stat.label);
    }

    let mut activity_list = WidgetChildren::default()
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                align_items: Some(WidgetAlignItems::Center),
                gap: (8.0.into(), 0.0.into()),
                margin: Edge::all(0.0).bottom(theme::space_md(theme)),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Typography>((Typography {
                    text: "Recent Activity".into(),
                    variant: TypographyVariant::H2,
                    color: Some(theme::text_primary(theme)),
                },))
                .with_key("text")
                .with_child::<Spinner>((Spinner {
                    size: 14.0,
                    ..Default::default()
                },))
                .with_key("spinner"),
        ))
        .with_key("title");
    for entry in activity {
        activity_list.add::<Element>(activity_row(theme, entry));
        activity_list.add_key(format!("{}-{}-{}", entry.actor, entry.action, entry.target));
    }
    activity_list
        .add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                justify_content: Some(WidgetAlignContent::Center),
                margin: Edge::all(0.0).top(theme::space_sm(theme)),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<Link>((Link {
                label: "View all activity".into(),
            },)),
        ))
        .observe(
            current_widget,
            |_trigger: On<Change<LinkClicked>>, mut queue: ResMut<ToastQueue>| {
                queue.push("Full activity log coming soon", BadgeVariant::Info);
            },
        );
    activity_list.add_key("view-all");

    let content = WidgetChildren::default()
        .with_child::<Alert>((
            Alert {
                variant: BadgeVariant::Warning,
                dismissible: false,
            },
            WoodpeckerStyle {
                margin: Edge::all(0.0).bottom(theme::space_lg(theme)),
                ..Default::default()
            },
            PassedChildren(
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        text_wrap: TextWrap::WordOrGlyph,
                        // Not left to `WoodpeckerStyle::default()`'s white -- `Alert`'s own
                        // internal `color: alert_styles.text_color` wrapper doesn't propagate
                        // to `PassedChildren` content (no style inheritance in this crate), so
                        // this needs its own explicit, theme-correct color.
                        color: theme::text_primary(theme),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "You're on the legacy pricing plan. Upgrade for higher rate \
                              limits and priority support."
                            .into(),
                    },
                )),
            ),
        ))
        .with_key("plan-banner")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Row,
                gap: (theme::space_md(theme).into(), 0.0.into()),
                margin: Edge::all(0.0).bottom(theme::space_lg(theme)),
                ..Default::default()
            },
            stat_row,
        ))
        .with_key("stats")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Row,
                gap: (theme::space_md(theme).into(), 0.0.into()),
                margin: Edge::all(0.0).bottom(theme::space_lg(theme)),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Card>((
                    Card {
                        title: Some("Current Plan".into()),
                        subtitle: Some("Pro \u{2014} billed monthly".into()),
                        elevation: 1,
                    },
                    WoodpeckerStyle {
                        flex_grow: 1.0,
                        ..Default::default()
                    },
                    PassedChildren(WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: 13.0,
                            color: theme::text_secondary(theme),
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: "18,204 of 25,000 seats used this cycle.".into(),
                        },
                    ))),
                    CardActions(
                        WidgetChildren::default()
                            .with_child::<WButton>((
                                WButton,
                                theme::secondary_button_styles(theme),
                                WidgetChildren::default().with_child::<Element>((
                                    Element,
                                    WoodpeckerStyle {
                                        font_size: 13.0,
                                        // Not left to white -- `secondary_button_styles`'s
                                        // background is `bg_input` (light in light mode), so
                                        // this label needs its own theme-correct color.
                                        color: theme::text_primary(theme),
                                        ..Default::default()
                                    },
                                    WidgetRender::Text {
                                        content: "Manage billing".into(),
                                    },
                                )),
                            ))
                            .with_observe(
                                current_widget,
                                |_trigger: On<Pointer<Click>>, mut queue: ResMut<ToastQueue>| {
                                    queue.push("Billing settings coming soon", BadgeVariant::Info);
                                },
                            )
                            .with_key("manage")
                            .with_child::<WButton>((
                                WButton,
                                theme::primary_button_styles(theme),
                                WidgetChildren::default().with_child::<Element>((
                                    Element,
                                    WoodpeckerStyle {
                                        font_size: 13.0,
                                        ..Default::default()
                                    },
                                    WidgetRender::Text {
                                        content: "Upgrade".into(),
                                    },
                                )),
                            ))
                            .with_observe(
                                current_widget,
                                |_trigger: On<Pointer<Click>>, mut queue: ResMut<ToastQueue>| {
                                    queue.push("Upgrade flow coming soon", BadgeVariant::Info);
                                },
                            )
                            .with_key("upgrade"),
                    ),
                ))
                .with_key("plan")
                .with_child::<Paper>((
                    Paper { elevation: 1 },
                    WoodpeckerStyle {
                        flex_grow: 1.0,
                        padding: Edge::all(theme::space_lg(theme)),
                        flex_direction: WidgetFlexDirection::Column,
                        ..Default::default()
                    },
                    WidgetChildren::default()
                        .with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                font_size: 14.0,
                                color: theme::text_primary(theme),
                                margin: Edge::all(0.0).bottom(6.0),
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                content: "Quick tip".into(),
                            },
                        ))
                        .with_key("title")
                        .with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                font_size: 13.0,
                                color: theme::text_secondary(theme),
                                text_wrap: TextWrap::WordOrGlyph,
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                content: "Right-click a row in the Users table for quick \
                                          actions like editing or copying an email address."
                                    .into(),
                            },
                        ))
                        .with_key("body"),
                ))
                .with_key("tip"),
        ))
        .with_key("plan-and-tip")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Column,
                margin: Edge::all(0.0).bottom(theme::space_lg(theme)),
                ..theme::card_style(theme)
            },
            WidgetRender::Quad,
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        justify_content: Some(WidgetAlignContent::SpaceBetween),
                        align_items: Some(WidgetAlignItems::Center),
                        ..Default::default()
                    },
                    WidgetChildren::default()
                        .with_child::<Element>((
                            Element,
                            theme::section_title_style(theme),
                            WidgetRender::Text {
                                content: "Revenue Trend".into(),
                            },
                        ))
                        .with_key("text")
                        .with_child::<ButtonGroup>((ButtonGroup {
                            options: vec!["3mo".into(), "6mo".into(), "12mo".into()],
                            selected: 2,
                        },))
                        .with_observe(
                            current_widget,
                            |trigger: On<Change<ButtonGroupChanged>>,
                             mut queue: ResMut<ToastQueue>| {
                                let range =
                                    ["3 months", "6 months", "12 months"][trigger.data.index];
                                queue.push(
                                    format!("Showing last {range} (chart data coming soon)"),
                                    BadgeVariant::Info,
                                );
                            },
                        )
                        .with_key("range"),
                ))
                .with_key("title")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 12.0,
                        color: theme::text_muted(theme),
                        margin: Edge::all(0.0).bottom(theme::space_md(theme)),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Last 12 months, MRR in thousands".into(),
                    },
                ))
                .with_key("subtitle")
                .with_child::<LineChart>((
                    LineChart {
                        series: vec![
                            ChartSeries::new(
                                "This year",
                                revenue_this_year.to_vec(),
                                theme::accent(theme),
                            ),
                            ChartSeries::new(
                                "Last year",
                                revenue_last_year.to_vec(),
                                theme::text_muted(theme),
                            ),
                        ],
                        show_legend: true,
                    },
                    // Two overlapping gradient fills get muddy fast -- lines + points alone
                    // read more clearly for a year-over-year comparison like this one.
                    ChartStyles {
                        show_fill: false,
                        ..Default::default()
                    },
                ))
                .with_key("chart"),
        ))
        .with_key("trend")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                ..theme::card_style(theme)
            },
            WidgetRender::Quad,
            activity_list,
        ))
        .with_key("activity")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Column,
                margin: Edge::all(0.0).top(theme::space_lg(theme)),
                ..theme::card_style(theme)
            },
            WidgetRender::Quad,
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    theme::section_title_style(theme),
                    WidgetRender::Text {
                        content: "Recent Deployments".into(),
                    },
                ))
                .with_key("title")
                .with_child::<Table>((
                    Table {
                        columns: vec![
                            TableColumn::fixed("Service", 180.0),
                            TableColumn::fixed("Version", 100.0),
                            TableColumn::flexible("Deployed by"),
                            TableColumn::fixed("When", 90.0),
                        ],
                        rows: vec![
                            vec![
                                "billing-service".into(),
                                "v2.4.1".into(),
                                "Ava Thompson".into(),
                                "2m ago".into(),
                            ]
                            .into(),
                            vec![
                                "auth-middleware".into(),
                                "v1.9.0".into(),
                                "Marcus Chen".into(),
                                "18m ago".into(),
                            ]
                            .into(),
                            vec![
                                "worker-pool".into(),
                                "v3.1.2".into(),
                                "System".into(),
                                "1h ago".into(),
                            ]
                            .into(),
                        ]
                        .into(),
                        ..Default::default()
                    },
                    TableStyles {
                        header_background: theme::bg_card_hover(theme),
                        zebra_background: theme::bg_card_hover(theme),
                        border_color: theme::border(theme),
                        corner_radius: theme::radius_md(theme),
                        ..Default::default()
                    },
                ))
                .with_key("table"),
        ))
        .with_key("deployments");

    theme::scrollable_page(content)
}
