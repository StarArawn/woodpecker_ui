use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::mock_data::PermissionAssignment;
use crate::theme;

/// A `TransferList` bound to the `PermissionAssignment` resource -- lets an admin move
/// permissions between "available" and "granted".
#[derive(Widget, Component, Reflect, Clone, Default, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_permissions_panel)]
#[require(
    WidgetChildren,
    WoodpeckerStyle = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        ..Default::default()
    },
    WatchedResource<PermissionAssignment>
)]
pub struct PermissionsPanel;

fn render_permissions_panel(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&WatchedResource<PermissionAssignment>, &mut WidgetChildren)>,
) {
    let Ok((assignment, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let assignment = assignment.0.clone();
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children
        .add::<TransferList>((TransferList {
            left: assignment.available,
            right: assignment.granted,
            left_title: "Available".into(),
            right_title: "Granted".into(),
        },))
        .observe(
            current_widget,
            |trigger: On<Change<TransferListChanged>>,
             mut assignment: ResMut<PermissionAssignment>| {
                assignment.available = trigger.data.left.clone();
                assignment.granted = trigger.data.right.clone();
            },
        );
    children.add_key("permissions");

    children.apply(current_widget.as_parent());
}

const THEMES: [&str; 3] = ["Light", "Dark", "System"];
const TIMEZONES: [&str; 4] = [
    "UTC",
    "America/New_York",
    "America/Los_Angeles",
    "Europe/London",
];
const COUNTRIES: [&str; 8] = [
    "United States",
    "United Kingdom",
    "Canada",
    "Germany",
    "France",
    "Japan",
    "Brazil",
    "Australia",
];

fn section(theme: &Theme, title: &'static str, rows: WidgetChildren) -> impl Bundle + Clone {
    (
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
                theme::section_title_style(theme),
                WidgetRender::Text {
                    content: title.into(),
                },
            ))
            .with_key("title")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    max_width: Units::Pixels(640.0),
                    flex_direction: WidgetFlexDirection::Column,
                    ..Default::default()
                },
                rows,
            ))
            .with_key("rows"),
    )
}

fn form_row(
    theme: &Theme,
    label: &'static str,
    description: &'static str,
    control: impl Bundle + Clone,
) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            align_items: Some(WidgetAlignItems::Center),
            padding: Edge::all(theme::space_sm(theme)).left(0.0).right(0.0),
            border: Edge::all(0.0).bottom(1.0),
            border_color: theme::border(theme),
            ..Default::default()
        },
        WidgetRender::Quad,
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
                            font_size: 14.0,
                            color: theme::text_primary(theme),
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: label.into(),
                        },
                    ))
                    .with_key("label")
                    .with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: 12.0,
                            color: theme::text_muted(theme),
                            margin: Edge::all(0.0).top(2.0),
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: description.into(),
                        },
                    ))
                    .with_key("description"),
            ))
            .with_key("info")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_grow: 1.0,
                    justify_content: Some(WidgetAlignContent::End),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<Element>(control),
            ))
            .with_key("control"),
    )
}

fn labeled_control(width: f32, control: impl Bundle + Clone) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: width.into(),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Element>(control),
    )
}

/// A small round "i" -- used as a [`Popover`] trigger for inline help text.
fn info_icon(theme: &Theme) -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: 16.0.into(),
            height: 16.0.into(),
            background_color: theme::bg_card_hover(theme),
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
                color: theme::text_muted(theme),
                ..Default::default()
            },
            WidgetRender::Text {
                content: "i".into(),
            },
        )),
    ))
}

const AUDIT_LOG_ROWS: usize = 500;
const AUDIT_LOG_ACTIONS: [&str; 5] = [
    "signed in",
    "updated billing settings",
    "invited a teammate",
    "rotated an API key",
    "exported a report",
];

/// One synthetic audit-log row -- only the rows actually in view (plus a small overscan) are
/// ever spawned, since [`VirtualList`] windows them (see `examples/virtual_list.rs` for the
/// same technique with 10,000 rows).
fn audit_log_row(theme: &Theme, index: usize) -> WidgetChildren {
    let action = AUDIT_LOG_ACTIONS[index % AUDIT_LOG_ACTIONS.len()];
    let stripe = if index % 2 == 1 {
        theme::bg_card_hover(theme).with_alpha(0.35)
    } else {
        Color::NONE
    };
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            align_items: Some(WidgetAlignItems::Center),
            justify_content: Some(WidgetAlignContent::SpaceBetween),
            padding: Edge::all(0.0)
                .left(theme::space_sm(theme))
                .right(theme::space_sm(theme)),
            background_color: stripe,
            ..Default::default()
        },
        WidgetRender::Quad,
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 12.0,
                    color: theme::text_secondary(theme),
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("user-{:03} {action}", index % 137),
                },
            ))
            .with_key("text")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 11.0,
                    color: theme::text_muted(theme),
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("{}m ago", index + 1),
                },
            ))
            .with_key("time"),
    ))
}

/// Tracks the left pane's width for the "Preview" section's [`Splitter`] demo -- `Splitter`
/// only reports a drag delta, not an absolute width, so the caller owns the actual value (same
/// pattern as `examples/splitter.rs`).
#[derive(Resource, Reflect, Clone, Copy, PartialEq)]
pub struct PreviewSplitWidth(f32);

impl Default for PreviewSplitWidth {
    fn default() -> Self {
        Self(220.0)
    }
}

/// The left pane's width at the moment the current drag started -- `SplitterChanged::delta` is
/// the *total* distance since drag start, not a per-frame increment (see its own doc comment),
/// so this fixed snapshot is what the resize handler adds it to each time.
#[derive(Resource, Default)]
pub struct PreviewSplitWidthAtDragStart(f32);

/// A resizable two-pane "Preview" demo -- watches `PreviewSplitWidth` so dragging the divider
/// live-resizes the left pane, and `Theme` so a theme swap recolors it.
#[derive(Widget, Component, Reflect, Clone, Default, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_preview_splitter)]
#[require(
    WidgetChildren,
    WoodpeckerStyle = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: 160.0.into(),
        flex_direction: WidgetFlexDirection::Row,
        ..Default::default()
    },
    WatchedResource<PreviewSplitWidth>,
    WatchedResource<Theme>
)]
pub struct PreviewSplitter;

fn preview_pane(
    theme: &Theme,
    label: &str,
    style: WoodpeckerStyle,
    background: Color,
) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            height: Units::Percentage(100.0),
            background_color: background,
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..style
        },
        WidgetRender::Quad,
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 13.0,
                color: theme::text_secondary(theme),
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: label.into(),
            },
        )),
    )
}

fn render_preview_splitter(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &WatchedResource<PreviewSplitWidth>,
        &WatchedResource<Theme>,
        &mut WidgetChildren,
    )>,
) {
    let Ok((left_width, watched_theme, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let theme = watched_theme.0;
    let current_widget = *current_widget;

    *children = WidgetChildren::default()
        .with_child::<Element>(preview_pane(
            &theme,
            "Editor",
            WoodpeckerStyle {
                width: left_width.0 .0.into(),
                ..Default::default()
            },
            theme::bg_input(&theme),
        ))
        .with_key("left")
        .with_child::<Splitter>(Splitter::default())
        .with_key("splitter")
        .with_observe(
            current_widget,
            |_trigger: On<Pointer<DragStart>>,
             left_width: Res<PreviewSplitWidth>,
             mut base: ResMut<PreviewSplitWidthAtDragStart>| {
                base.0 = left_width.0;
            },
        )
        .with_observe(
            current_widget,
            |trigger: On<Change<SplitterChanged>>,
             mut left_width: ResMut<PreviewSplitWidth>,
             base: Res<PreviewSplitWidthAtDragStart>| {
                left_width.0 = (base.0 + trigger.data.delta).clamp(120.0, 400.0);
            },
        )
        .with_child::<Element>(preview_pane(
            &theme,
            "Preview",
            WoodpeckerStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
            theme::bg_card_hover(&theme),
        ))
        .with_key("right");

    children.apply(current_widget.as_parent());
}

pub fn build_settings_page(
    theme: &Theme,
    current_widget: CurrentWidget,
    brand_image: Handle<Image>,
) -> WidgetChildren {
    let profile_rows = WidgetChildren::default()
        .with_child::<Element>(form_row(
            theme,
            "Display name",
            "Shown across the console and in audit logs",
            labeled_control(
                240.0,
                (
                    Element,
                    // `width: 100%` (not `Auto`) so this wrapper fills `labeled_control`'s
                    // definite 240px box; `TextBox` is `width: 100%` internally too.
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        ..Default::default()
                    },
                    WidgetChildren::default().with_child::<TextBox>((TextBox {
                        initial_value: "Ava Thompson".into(),
                        ..Default::default()
                    },)),
                ),
            ),
        ))
        .with_key("display-name")
        .with_child::<Element>(form_row(
            theme,
            "Timezone",
            "Used for scheduling and activity timestamps",
            labeled_control(
                200.0,
                (
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        ..Default::default()
                    },
                    WidgetChildren::default().with_child::<Dropdown>((Dropdown {
                        list: TIMEZONES.iter().map(|t| t.to_string()).collect(),
                        current_value: TIMEZONES[0].into(),
                    },)),
                ),
            ),
        ))
        .with_key("timezone")
        .with_child::<Element>(form_row(
            theme,
            "Country",
            "Type to search, or browse the full list",
            labeled_control(
                200.0,
                (
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        ..Default::default()
                    },
                    WidgetChildren::default().with_child::<ComboBox>((ComboBox {
                        current_value: COUNTRIES[0].into(),
                        list: COUNTRIES.iter().map(|c| c.to_string()).collect(),
                        match_mode: ComboBoxMatchMode::Contains,
                    },)),
                ),
            ),
        ))
        .with_key("country")
        .with_child::<Element>(form_row(
            theme,
            "Profile completeness",
            "Add a photo and bio to finish setting up your profile",
            (
                Element,
                WoodpeckerStyle {
                    width: 200.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<ProgressBar>((ProgressBar { value: 0.8 },)),
            ),
        ))
        .with_key("completeness");

    let preference_rows = WidgetChildren::default()
        .with_child::<Element>(form_row(
            theme,
            "Appearance",
            "Choose how the console looks on this device",
            labeled_control(
                160.0,
                (
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        ..Default::default()
                    },
                    WidgetChildren::default().with_child::<Dropdown>((Dropdown {
                        list: THEMES.iter().map(|t| t.to_string()).collect(),
                        current_value: THEMES[1].into(),
                    },)),
                ),
            ),
        ))
        .with_key("appearance")
        .with_child::<Element>(form_row(
            theme,
            "Session timeout",
            "Automatically sign out after this many hours idle",
            (
                Element,
                WoodpeckerStyle {
                    width: 200.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<Slider>((Slider {
                    start: 1.0,
                    end: 24.0,
                    value: 8.0,
                },)),
            ),
        ))
        .with_key("session-timeout")
        .with_child::<Element>(form_row(
            theme,
            "Max login attempts",
            "Lock the account after this many failed sign-ins",
            (
                Element,
                WoodpeckerStyle {
                    width: 140.0.into(),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<NumberInput>((NumberInput {
                    value: 5.0,
                    min: 1.0,
                    max: 10.0,
                    step: 1.0,
                    ..Default::default()
                },)),
            ),
        ))
        .with_key("max-attempts")
        .with_child::<Element>(form_row(
            theme,
            "Email notifications",
            "Weekly digests and critical alerts",
            (
                Element,
                WoodpeckerStyle::default(),
                WidgetChildren::default().with_child::<Checkbox>(Checkbox),
            ),
        ))
        .with_key("email-notifications")
        .with_child::<Element>(form_row(
            theme,
            "Maintenance mode",
            "Show a maintenance banner and block new sign-ins",
            (
                Element,
                WoodpeckerStyle::default(),
                WidgetChildren::default().with_child::<Toggle>(Toggle),
            ),
        ))
        .with_key("maintenance-mode")
        .with_child::<Element>(form_row(
            theme,
            "Two-factor authentication",
            "Require a verification code at sign-in",
            (
                Element,
                WoodpeckerStyle {
                    align_items: Some(WidgetAlignItems::Center),
                    gap: (8.0.into(), 0.0.into()),
                    ..Default::default()
                },
                WidgetChildren::default()
                    .with_child::<Tooltip>((
                        Tooltip {
                            text: "Adds a one-time code step at sign-in".into(),
                            placement: PopoverPlacement::Left,
                        },
                        PassedChildren(info_icon(theme)),
                    ))
                    .with_key("info")
                    .with_child::<Checkbox>(Checkbox)
                    .with_key("checkbox"),
            ),
        ))
        .with_key("two-factor")
        .with_child::<Element>(form_row(
            theme,
            "Default landing page",
            "Which page opens first when you sign in",
            (
                Element,
                WoodpeckerStyle::default(),
                WidgetChildren::default().with_child::<RadioGroup>((RadioGroup {
                    options: vec!["Overview".into(), "Users".into(), "Settings".into()],
                    selected: 0,
                    horizontal: true,
                },)),
            ),
        ))
        .with_key("landing-page");

    let branding_rows = WidgetChildren::default().with_child::<Element>(form_row(
        theme,
        "Accent color",
        "Used for links, primary buttons, and highlights",
        (
            Element,
            WoodpeckerStyle::default(),
            WidgetChildren::default().with_child::<ColorPicker>((ColorPicker {
                initial_color: theme::accent(theme),
            },)),
        ),
    ));

    let storage_rows = WidgetChildren::default().with_child::<Element>(form_row(
        theme,
        "Storage",
        "6.8 GB of 10 GB used",
        (
            Element,
            WoodpeckerStyle {
                align_items: Some(WidgetAlignItems::Center),
                gap: (8.0.into(), 0.0.into()),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Spinner>((Spinner {
                    size: 32.0,
                    mode: SpinnerMode::Determinate(0.68),
                },))
                .with_key("ring")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 13.0,
                        color: theme::text_primary(theme),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "68%".into(),
                    },
                ))
                .with_key("percent"),
        ),
    ));

    let integration_rows = {
        let mut rows = WidgetChildren::default();
        for name in ["Slack", "GitHub", "Figma"] {
            rows.add::<Element>(form_row(
                theme,
                name,
                "Checking connection status\u{2026}",
                (
                    Element,
                    WoodpeckerStyle {
                        flex_direction: WidgetFlexDirection::Row,
                        align_items: Some(WidgetAlignItems::Center),
                        gap: (8.0.into(), 0.0.into()),
                        ..Default::default()
                    },
                    WidgetChildren::default()
                        .with_child::<Skeleton>((Skeleton {
                            variant: SkeletonVariant::Circle,
                            width: 20.0,
                            height: 20.0,
                        },))
                        .with_key("icon")
                        .with_child::<Skeleton>((Skeleton {
                            variant: SkeletonVariant::Text,
                            width: 60.0,
                            height: 12.0,
                        },))
                        .with_key("status"),
                ),
            ));
            rows.add_key(name);
        }
        rows
    };

    let doc_tree = vec![
        TreeNode {
            key: "getting-started".into(),
            label: "Getting Started".into(),
            children: vec![
                TreeNode {
                    key: "getting-started/setup".into(),
                    label: "Workspace setup".into(),
                    children: vec![],
                },
                TreeNode {
                    key: "getting-started/invite".into(),
                    label: "Inviting your team".into(),
                    children: vec![],
                },
            ],
        },
        TreeNode {
            key: "guides".into(),
            label: "Guides".into(),
            children: vec![
                TreeNode {
                    key: "guides/api".into(),
                    label: "REST API".into(),
                    children: vec![],
                },
                TreeNode {
                    key: "guides/webhooks".into(),
                    label: "Webhooks".into(),
                    children: vec![],
                },
                TreeNode {
                    key: "guides/sso".into(),
                    label: "SSO & SCIM".into(),
                    children: vec![],
                },
            ],
        },
        TreeNode {
            key: "billing-docs".into(),
            label: "Billing".into(),
            children: vec![TreeNode {
                key: "billing-docs/plans".into(),
                label: "Plans & pricing".into(),
                children: vec![],
            }],
        },
    ];

    let faq = WidgetChildren::default()
        .with_child::<AccordionItem>((
            AccordionItem {
                key: "billing".into(),
                label: "How is billing calculated?".into(),
            },
            PassedChildren(WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: theme::text_secondary(theme),
                    text_wrap: TextWrap::WordOrGlyph,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "You're billed monthly based on active seats and usage tier.".into(),
                },
            ))),
        ))
        .with_key("billing")
        .with_child::<AccordionItem>((
            AccordionItem {
                key: "export".into(),
                label: "Can I export my data?".into(),
            },
            PassedChildren(WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: theme::text_secondary(theme),
                    text_wrap: TextWrap::WordOrGlyph,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content:
                        "Yes -- request a full export from Settings > Data at any time.".into(),
                },
            ))),
        ))
        .with_key("export");

    // `VirtualList::item_content` closures capture-by-value (see `VirtualListItemContent`'s own
    // doc comment: changing the closure alone is never itself a reason to re-render, so a stale
    // captured `Theme` only self-corrects once something else -- item_count/item_extent, a
    // scroll -- triggers a real re-render of the affected rows) -- `audit_log_row` itself keeps
    // the plain `fn(&Theme, usize) -> WidgetChildren` shape the rest of this module uses, and
    // gets adapted to `VirtualListItemContent`'s required `Fn(usize) -> WidgetChildren` here.
    let audit_log_theme = *theme;

    let content = WidgetChildren::default()
        .with_child::<Element>((
            Element,
            theme::section_title_style(theme),
            WidgetRender::Text {
                content: "Settings".into(),
            },
        ))
        .with_key("header")
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
                    theme::section_title_style(theme),
                    WidgetRender::Text {
                        content: "Workspace Setup".into(),
                    },
                ))
                .with_key("title")
                .with_child::<Stepper>((Stepper {
                    steps: vec![
                        "Create account".into(),
                        "Verify email".into(),
                        "Invite team".into(),
                        "Go live".into(),
                    ],
                    active: 2,
                    orientation: StepperOrientation::Horizontal,
                    clickable: false,
                },))
                .with_key("stepper"),
        ))
        .with_key("setup")
        .with_child::<Element>(section(theme, "Profile", profile_rows))
        .with_key("profile")
        .with_child::<Element>(section(theme, "Preferences", preference_rows))
        .with_key("preferences")
        .with_child::<Element>(section(theme, "Branding", branding_rows))
        .with_key("branding")
        .with_child::<Element>(section(
            theme,
            "Brand Assets",
            WidgetChildren::default()
                .with_child::<ImageList>((ImageList {
                    items: (1..=6)
                        .map(|i| ImageListItem {
                            handle: brand_image.clone(),
                            label: Some(format!("asset-{i:02}.jpg")),
                        })
                        .collect(),
                    columns: 4,
                    tile_size: 90.0,
                    gap: 8.0,
                },))
                .with_key("gallery"),
        ))
        .with_key("brand-assets")
        .with_child::<Element>(section(
            theme,
            "Design Moodboard",
            WidgetChildren::default()
                .with_child::<Masonry>((Masonry {
                    items: [140.0, 90.0, 180.0, 110.0, 70.0, 150.0]
                        .into_iter()
                        .enumerate()
                        .map(|(i, height)| MasonryItem {
                            handle: brand_image.clone(),
                            height,
                            label: Some(format!("ref-{:02}.jpg", i + 1)),
                        })
                        .collect(),
                    columns: 3,
                    column_width: 90.0,
                    gap: 8.0,
                },))
                .with_key("board"),
        ))
        .with_key("moodboard")
        .with_child::<Element>(section(theme, "Storage", storage_rows))
        .with_key("storage")
        .with_child::<Element>(section(theme, "Integrations", integration_rows))
        .with_key("integrations")
        .with_child::<Element>(section(
            theme,
            "Permissions",
            WidgetChildren::default()
                .with_child::<PermissionsPanel>(PermissionsPanel)
                .with_key("panel"),
        ))
        .with_key("permissions")
        .with_child::<Element>(section(
            theme,
            "Documentation",
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Pixels(320.0),
                        ..Default::default()
                    },
                    WidgetChildren::default().with_child::<TreeView>(TreeView {
                        nodes: doc_tree,
                        indent: 18.0,
                        ..Default::default()
                    }),
                ))
                .with_key("tree"),
        ))
        .with_key("documentation")
        .with_child::<Element>(section(
            theme,
            "Audit Log",
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        height: 240.0.into(),
                        border: Edge::all(1.0),
                        border_color: theme::border(theme),
                        border_radius: Corner::all(theme::radius_sm(theme)),
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                    WidgetChildren::default().with_child::<VirtualList>(VirtualList {
                        item_count: AUDIT_LOG_ROWS,
                        item_extent: 28.0,
                        item_content: VirtualListItemContent::new(move |index| {
                            audit_log_row(&audit_log_theme, index)
                        }),
                    }),
                ))
                .with_key("log"),
        ))
        .with_key("audit-log")
        .with_child::<Element>(section(
            theme,
            "Preview",
            WidgetChildren::default()
                .with_child::<PreviewSplitter>(PreviewSplitter)
                .with_key("splitter"),
        ))
        .with_key("preview")
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
                    theme::section_title_style(theme),
                    WidgetRender::Text {
                        content: "Help & FAQ".into(),
                    },
                ))
                .with_key("title")
                .with_child::<Accordion>((
                    Accordion {
                        mode: AccordionMode::Single,
                    },
                    WoodpeckerStyle {
                        max_width: Units::Pixels(640.0),
                        ..Default::default()
                    },
                    PassedChildren(faq),
                ))
                .with_key("faq"),
        ))
        .with_key("help")
        .with_child::<WButton>((
            WButton,
            theme::primary_button_styles(theme),
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 14.0,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Save changes".into(),
                },
            )),
        ))
        .with_observe(
            current_widget,
            |_trigger: On<Pointer<Click>>, mut queue: ResMut<ToastQueue>| {
                queue.push("Settings saved", BadgeVariant::Success);
            },
        )
        .with_key("save");

    theme::scrollable_page(content)
}
