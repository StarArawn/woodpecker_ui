//! A Storybook-style gallery: a sidebar lists a "Guide" section (how Woodpecker UI works, how
//! to build your own widgets) followed by every widget the crate has, grouped into categories;
//! clicking one shows either a guide page or a live demo in the main panel. Run with
//! `cargo run --example gallery`.

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

mod demos;
mod guide;
mod props;
mod stories;
mod type_docs;

use demos::{
    ChipDemo, DrawerDemo, ImageListDemo, MasonryDemo, ModalDemo, PaginationDemo, PopoverDemo,
    RatingDemo, SplitterDemo, StepperDemo, ToastDemo, ToggleButtonDemo, TransferListDemo,
};
use guide::{guide_title, markdown_source, GuideTopic, GUIDE_TOPICS};
use props::{build_props_rows, build_related_types_section};
use stories::{story_doc, story_props, Story, STORIES};

const SIDEBAR_WIDTH: f32 = 260.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WoodpeckerUIPlugin::default())
        .register_widget::<Gallery>()
        .register_widget::<ToggleButtonDemo>()
        .register_widget::<PaginationDemo>()
        .register_widget::<StepperDemo>()
        .register_widget::<ChipDemo>()
        .register_widget::<RatingDemo>()
        .register_widget::<ToastDemo>()
        .register_widget::<DrawerDemo>()
        .register_widget::<ModalDemo>()
        .register_widget::<PopoverDemo>()
        .register_widget::<MasonryDemo>()
        .register_widget::<ImageListDemo>()
        .register_widget::<TransferListDemo>()
        .register_widget::<SplitterDemo>()
        .add_systems(Startup, startup)
        .run();
}

/// A single sidebar destination -- either a conceptual guide page or a widget's own catalog
/// entry. Kept as one enum (rather than two separately-tracked `Option`s) so `GalleryState`
/// only ever needs to track one selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
enum Page {
    Guide(GuideTopic),
    Widget(Story),
}

#[derive(Component, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct GalleryState {
    selected: Option<Page>,
}

#[derive(Widget, Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
struct Gallery;

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    theme: Res<Theme>,
    mut query: Query<(&mut WoodpeckerStyle, &mut WidgetChildren), With<Gallery>>,
    state_query: Query<&GalleryState>,
) {
    let Ok((mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let state_entity = hooks.use_state(&mut commands, *current_widget, GalleryState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };
    let selected = state.selected;
    let current_widget = *current_widget;

    *styles = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        flex_direction: WidgetFlexDirection::Column,
        background_color: theme.dark_background,
        ..*styles
    };

    *children = WidgetChildren::default();

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: 64.0.into(),
            flex_direction: WidgetFlexDirection::Row,
            align_items: Some(WidgetAlignItems::Center),
            padding: Edge::all(0.0).left(theme.spacing.lg).right(theme.spacing.lg),
            background_color: theme.dark_background,
            border_color: theme.border,
            border: Edge::all(0.0).bottom(1.0),
            ..Default::default()
        },
        WidgetRender::Quad,
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: theme.typography.h2,
                    color: Color::WHITE,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Woodpecker UI".into(),
                },
            ))
            .with_key("brand")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: theme.typography.body,
                    color: theme.text.with_alpha(0.6),
                    margin: Edge::all(0.0).left(theme.spacing.sm),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Component Gallery".into(),
                },
            ))
            .with_key("subtitle"),
    ));
    children.add_key("header");

    let mut body = WidgetChildren::default();

    let nav_entries: Vec<(&'static str, &'static str, Page)> = GUIDE_TOPICS
        .iter()
        .map(|&(label, topic)| ("Guide", label, Page::Guide(topic)))
        .chain(
            STORIES
                .iter()
                .map(|&(category, label, story)| (category, label, Page::Widget(story))),
        )
        .collect();

    let mut sidebar_items = WidgetChildren::default();
    let mut last_category: Option<&str> = None;
    for (category, label, page) in nav_entries {
        if last_category != Some(category) {
            sidebar_items.add::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 11.0,
                    color: theme.text.with_alpha(0.4),
                    padding: Edge::all(0.0)
                        .left(theme.spacing.md)
                        .right(theme.spacing.md)
                        .bottom(theme.spacing.xs),
                    margin: Edge::all(0.0).top(if last_category.is_none() {
                        theme.spacing.md
                    } else {
                        theme.spacing.xl
                    }),
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: category.to_uppercase(),
                },
            ));
            sidebar_items.add_key(format!("header-{category}"));
            last_category = Some(category);
        }
        let is_selected = selected == Some(page);
        let mut list_item_child = WidgetChildren::default();
        list_item_child.add::<ListItem>((
            ListItem {
                primary_text: label.into(),
                selected: is_selected,
                dense: true,
                ..Default::default()
            },
            WoodpeckerStyle {
                padding: Edge::all(0.0)
                    .left(theme.spacing.md - 3.0)
                    .right(theme.spacing.md),
                ..Default::default()
            },
        ));
        list_item_child.observe(
            current_widget,
            move |_: On<Change<ListItemClicked>>, mut query: Query<&mut GalleryState>| {
                if let Ok(mut state) = query.get_mut(state_entity) {
                    state.selected = Some(page);
                }
            },
        );
        list_item_child.add_key("item");

        sidebar_items.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                border: Edge::all(0.0).left(3.0),
                border_color: if is_selected {
                    theme.primary
                } else {
                    Color::NONE
                },
                ..Default::default()
            },
            list_item_child,
        ));
        sidebar_items.add_key(format!("item-{category}-{label}"));
    }

    body.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: SIDEBAR_WIDTH.into(),
            height: Units::Percentage(100.0),
            padding: Edge::all(theme.spacing.sm),
            background_color: theme.background,
            border_color: theme.border,
            border: Edge::all(0.0).right(1.0),
            ..Default::default()
        },
        WidgetRender::Quad,
        WidgetChildren::default().with_child::<ScrollBox>((
            ScrollBox::default(),
            PassedChildren(sidebar_items),
        )),
    ));
    body.add_key("sidebar");

    let Some(page) = selected else {
        body.add::<Element>((
            Element,
            WoodpeckerStyle {
                flex_grow: 1.0,
                height: Units::Percentage(100.0),
                align_items: Some(WidgetAlignItems::Center),
                justify_content: Some(WidgetAlignContent::Center),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: theme.typography.body,
                    color: theme.text.with_alpha(0.6),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: "Pick a page from the sidebar".into(),
                },
            )),
        ));
        body.add_key("content-empty");

        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_grow: 1.0,
                flex_direction: WidgetFlexDirection::Row,
                ..Default::default()
            },
            body,
        ));
        children.add_key("body");
        children.apply(current_widget.as_parent());
        return;
    };

    let mut page_children = WidgetChildren::default();
    let (eyebrow, title) = match page {
        Page::Guide(topic) => ("Guide", guide_title(topic)),
        Page::Widget(story) => {
            let (category, label, _) = STORIES.iter().find(|(_, _, s)| *s == story).unwrap();
            (*category, *label)
        }
    };

    page_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 12.0,
            color: theme.primary,
            margin: Edge::all(0.0).bottom(theme.spacing.xs),
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: eyebrow.to_uppercase(),
        },
    ));
    page_children.add_key("eyebrow");
    page_children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: theme.typography.h1,
            color: Color::WHITE,
            margin: Edge::all(0.0).bottom(theme.spacing.sm),
            ..Default::default()
        },
        WidgetRender::Text {
            content: title.into(),
        },
    ));
    page_children.add_key("title");

    match page {
        Page::Guide(topic) => {
            page_children.add::<Markdown>((Markdown {
                content: markdown_source(topic).into(),
                ..Default::default()
            },));
            page_children.add_key(format!("guide-content-{topic:?}"));
        }
        Page::Widget(story) => {
            page_children.add::<Markdown>((
                Markdown {
                    content: story_doc(story).into(),
                    ..Default::default()
                },
                WoodpeckerStyle {
                    max_width: 640.0.into(),
                    margin: Edge::all(0.0).bottom(theme.spacing.lg),
                    ..Default::default()
                },
            ));
            page_children.add_key(format!("description-{story:?}"));

            let props = story_props(story);
            if !props.is_empty() {
                page_children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: theme.typography.h3,
                        color: Color::WHITE,
                        margin: Edge::all(0.0).bottom(theme.spacing.sm),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Props".into(),
                    },
                ));
                page_children.add_key("props-header");

                let rows = build_props_rows(&theme, props);

                page_children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        flex_direction: WidgetFlexDirection::Column,
                        margin: Edge::all(0.0).bottom(theme.spacing.lg),
                        padding: Edge::all(theme.spacing.md),
                        background_color: theme.background,
                        border_color: theme.border,
                        border: Edge::all(1.0),
                        border_radius: Corner::all(theme.panel_radius),
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                    rows,
                ));
                page_children.add_key("props");

                if let Some(related) = build_related_types_section(&theme, props) {
                    page_children.add::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: theme.typography.h3,
                            color: Color::WHITE,
                            margin: Edge::all(0.0).bottom(theme.spacing.sm),
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: "Related Types".into(),
                        },
                    ));
                    page_children.add_key("related-types-header");

                    page_children.add::<Element>((
                        Element,
                        WoodpeckerStyle {
                            width: Units::Percentage(100.0),
                            flex_direction: WidgetFlexDirection::Column,
                            margin: Edge::all(0.0).bottom(theme.spacing.lg),
                            ..Default::default()
                        },
                        related,
                    ));
                    page_children.add_key("related-types");
                }
            }

            page_children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    flex_direction: WidgetFlexDirection::Row,
                    flex_wrap: WidgetFlexWrap::Wrap,
                    align_items: Some(WidgetAlignItems::FlexStart),
                    gap: (theme.spacing.lg.into(), theme.spacing.lg.into()),
                    padding: Edge::all(theme.spacing.lg),
                    background_color: theme.background,
                    border_color: theme.border,
                    border: Edge::all(1.0),
                    border_radius: Corner::all(theme.panel_radius),
                    ..Default::default()
                },
                WidgetRender::Quad,
                demos::story_demo(story),
            ));
            page_children.add_key("canvas");
        }
    }

    body.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_grow: 1.0,
            height: Units::Percentage(100.0),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<ScrollBox>((
                ScrollBox::default(),
                PassedChildren(
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            width: Units::Percentage(100.0),
                            padding: Edge::all(theme.spacing.xl),
                            flex_direction: WidgetFlexDirection::Column,
                            ..Default::default()
                        },
                        page_children,
                    )),
                ),
            ))
            .with_key(format!("content-{page:?}")),
    ));
    body.add_key("content");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_grow: 1.0,
            flex_direction: WidgetFlexDirection::Row,
            ..Default::default()
        },
        body,
    ));
    children.add_key("body");

    children.apply(current_widget.as_parent());
}

fn startup(mut commands: Commands, mut ui_context: ResMut<WoodpeckerContext>) {
    commands.spawn((Camera2d, WoodpeckerView));

    let root_widget = ui_context.spawn_root(&mut commands);
    commands.entity(*root_widget).insert((
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Row,
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Gallery>(Gallery)
            .with_child::<OverlayRootWidget>(OverlayRootWidget)
            .with_child::<ToastViewport>(ToastViewport),
    ));
}
