use std::cmp::Reverse;

use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::mock_data::{
    CompactRows, EditingUser, UserPage, UserRecord, UserSearchQuery, Users, ROLES,
};
use crate::theme;

/// Rows shown per page -- deliberately small (rather than the mock dataset's full 100 users)
/// so `Pagination`'s ellipsis-collapse behavior actually has something to demonstrate.
const PAGE_SIZE: usize = 8;

/// Renders the user table. Watches `Users` so inline edits repaint the table with no
/// separate "local copy" to keep in sync, `UserSearchQuery` so the topbar search box
/// live-filters the list, `UserPage` so its own `Pagination` row below the table slices
/// the visible rows, and `Theme` so a theme swap recolors it.
#[derive(Widget, Component, Reflect, Clone, Default, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_users_table)]
#[require(
    WidgetChildren,
    WoodpeckerStyle = WoodpeckerStyle {
        flex_direction: WidgetFlexDirection::Column,
        width: Units::Percentage(100.0),
        ..Default::default()
    },
    WatchedResource<Users>,
    WatchedResource<UserSearchQuery>,
    WatchedResource<UserPage>,
    WatchedResource<CompactRows>,
    WatchedResource<Theme>
)]
pub struct UsersTable;

/// Scores `target` as a fuzzy match against `query` (case-insensitive), or `None` if
/// `query`'s characters don't all appear in `target` in order. Consecutive runs and matches
/// at the start of `target` score higher.
fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let query = query.to_lowercase();
    let target = target.to_lowercase();
    let mut score = 0i32;
    let mut last_match: Option<usize> = None;
    let mut consecutive = 0i32;
    let mut target_chars = target.char_indices();

    'query: for qc in query.chars() {
        for (i, tc) in &mut target_chars {
            if tc != qc {
                continue;
            }
            score += match last_match {
                Some(last) if i == last + 1 => {
                    consecutive += 1;
                    5 + consecutive * 2
                }
                Some(_) => {
                    consecutive = 0;
                    1
                }
                None => {
                    consecutive = 0;
                    if i == 0 {
                        10
                    } else {
                        1
                    }
                }
            };
            last_match = Some(i);
            continue 'query;
        }
        return None;
    }

    Some(score)
}

/// Best fuzzy score for `query` across a user's name, email, and role, or `None` if it
/// matches none of them.
fn user_match_score(query: &str, user: &UserRecord) -> Option<i32> {
    [
        fuzzy_score(query, &user.name),
        fuzzy_score(query, &user.email),
        fuzzy_score(query, &user.role),
    ]
    .into_iter()
    .flatten()
    .max()
}

/// First letter of up to the first two words of `name` (e.g. "Ava Thompson" -> "AT").
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

/// A row of role filter [`Chip`]s above the table -- clicking one sets `UserSearchQuery` to
/// that role name, reusing `UsersTable`'s existing fuzzy-match-against-role filtering rather
/// than a second, parallel filtering mechanism. Clicking the already-selected chip (or "All")
/// clears the query. Watches `UserSearchQuery` so its own `selected` chip stays in sync if the
/// topbar search box is used instead (they share the same underlying filter).
#[derive(Widget, Component, Reflect, Clone, Default, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_role_filter_chips)]
#[require(
    WidgetChildren,
    WoodpeckerStyle = WoodpeckerStyle {
        flex_direction: WidgetFlexDirection::Row,
        gap: (8.0.into(), 0.0.into()),
        // `8.0` (not `theme::space_sm(..)`) -- a `#[require]` default has no `&Theme` to read
        // (it's evaluated with no ECS access at all, unlike a normal render system), and
        // `Theme::dark()`/`Theme::light()`'s spacing scale is identical anyway (see
        // `theming.rs`'s own tests), so hardcoding this one spacing value here costs nothing.
        margin: Edge::all(0.0).bottom(8.0),
        ..Default::default()
    },
    WatchedResource<UserSearchQuery>
)]
pub struct RoleFilterChips;

fn render_role_filter_chips(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&WatchedResource<UserSearchQuery>, &mut WidgetChildren)>,
) {
    let Ok((search, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_query = search.0 .0.clone();
    let current_widget = *current_widget;

    *children = WidgetChildren::default();
    children
        .add::<Chip>((Chip {
            label: "All".into(),
            variant: BadgeVariant::Info,
            selected: current_query.is_empty(),
            ..Default::default()
        },))
        .observe(
            current_widget,
            |_trigger: On<Change<ChipClicked>>,
             mut search: ResMut<UserSearchQuery>,
             mut page: ResMut<UserPage>| {
                search.0.clear();
                page.0 = 0;
            },
        );
    children.add_key("all");

    for role in ROLES {
        children
            .add::<Chip>((Chip {
                label: role.into(),
                variant: BadgeVariant::Info,
                selected: current_query == role,
                ..Default::default()
            },))
            .observe(
                current_widget,
                move |_trigger: On<Change<ChipClicked>>,
                      mut search: ResMut<UserSearchQuery>,
                      mut page: ResMut<UserPage>| {
                    search.0 = role.into();
                    page.0 = 0;
                },
            );
        children.add_key(role);
    }

    children.apply(current_widget.as_parent());
}

fn header_cell(theme: &Theme, label: &'static str, width: f32) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: width.into(),
            flex_shrink: 0.0,
            font_size: 11.0,
            color: theme::text_muted(theme),
            ..Default::default()
        },
        WidgetRender::Text {
            content: label.into(),
        },
    )
}

/// Like [`header_cell`], but expands to fill whatever space the fixed-width columns don't
/// use, instead of a fixed width -- for the one column (`User`) whose content genuinely
/// benefits from more room rather than wrapping/truncating. `flex_basis: 0` means its share of
/// space comes entirely from `flex_grow`, not from its own natural content width, matching
/// `USER_COLUMN_MIN_WIDTH`'s use on the row's own equivalent cell below.
fn header_cell_flex(theme: &Theme, label: &'static str) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: Units::Pixels(0.0),
            min_width: USER_COLUMN_MIN_WIDTH.into(),
            font_size: 11.0,
            color: theme::text_muted(theme),
            ..Default::default()
        },
        WidgetRender::Text {
            content: label.into(),
        },
    )
}

/// Shared between [`header_cell_flex`] and the `User` column's row cell in [`add_user_row`] --
/// keeps the avatar + name/email from being squeezed unreadably narrow on a smaller window,
/// while still letting the column grow past this floor to fill available space.
const USER_COLUMN_MIN_WIDTH: f32 = 180.0;

fn render_users_table(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &WatchedResource<Users>,
        &WatchedResource<UserSearchQuery>,
        &WatchedResource<UserPage>,
        &WatchedResource<CompactRows>,
        &WatchedResource<Theme>,
        &mut WidgetChildren,
    )>,
) {
    let Ok((users, search, user_page, compact_rows, watched_theme, mut children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };
    let users = users.0.clone();
    let search_query = search.0 .0.clone();
    let compact = compact_rows.0 .0;
    let theme = watched_theme.0;
    let current_widget = *current_widget;

    // Keeps each surviving user's *original* index so row actions mutate the right entry
    // in `Users`, not its position in this filtered-and-sorted view.
    let mut matches: Vec<(usize, &UserRecord, i32)> = users
        .iter()
        .enumerate()
        .filter_map(|(i, user)| user_match_score(&search_query, user).map(|score| (i, user, score)))
        .collect();
    matches.sort_by_key(|m| Reverse(m.2));

    let total_pages = matches.len().div_ceil(PAGE_SIZE).max(1);
    // Clamp rather than trust the resource verbatim -- a search that narrows the result set
    // can leave a previously-valid page number out of range.
    let page = user_page.0 .0.min(total_pages - 1);
    let page_start = page * PAGE_SIZE;
    matches = matches
        .into_iter()
        .skip(page_start)
        .take(PAGE_SIZE)
        .collect();

    *children = WidgetChildren::default();

    // A distinct header band (rather than just a bottom divider) so it reads as a table
    // header rather than another row -- rounded on top to match the card it flushes into.
    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            background_color: theme::bg_card_hover(&theme),
            border_radius: Corner::all(0.0)
                .top_left(theme::radius_md(&theme))
                .top_right(theme::radius_md(&theme)),
            padding: Edge::all(theme::space_sm(&theme))
                .left(theme::space_md(&theme))
                .right(theme::space_md(&theme)),
            gap: (theme::space_md(&theme).into(), 0.0.into()),
            border: Edge::all(0.0).bottom(1.0),
            border_color: theme::border(&theme),
            ..Default::default()
        },
        WidgetRender::Quad,
        WidgetChildren::default()
            .with_child::<Element>(header_cell_flex(&theme, "User"))
            .with_key("h-user")
            .with_child::<Element>(header_cell(&theme, "Role", 160.0))
            .with_key("h-role")
            .with_child::<Element>(header_cell(&theme, "Status", 120.0))
            .with_key("h-status")
            .with_child::<Element>(header_cell(&theme, "", 80.0))
            .with_key("h-actions"),
    ));
    children.add_key("header");

    if matches.is_empty() {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                padding: Edge::all(theme::space_lg(&theme)),
                justify_content: Some(WidgetAlignContent::Center),
                color: theme::text_muted(&theme),
                ..Default::default()
            },
            WidgetRender::Text {
                content: format!("No users match \"{search_query}\"."),
            },
        ));
        children.add_key("empty");
    } else {
        // The last data row only gets the rounded-bottom-corner treatment when there's no
        // `Pagination` row following it to be the actual visual bottom instead.
        let shown_count = matches.len();
        let last_row = shown_count - 1;
        let rounds_bottom = total_pages <= 1;
        for (row_pos, (original_index, user, _score)) in matches.into_iter().enumerate() {
            add_user_row(
                &theme,
                &mut children,
                current_widget,
                original_index,
                rounds_bottom && row_pos == last_row,
                user,
                compact,
            );
            // Keyed by *position*, not `user.email` -- a page change (or a search that
            // reorders matches) shows an entirely different set of users at each position, so
            // an identity key would despawn and respawn every row from scratch on every page
            // change, and a brand-new widget subtree needs a frame or two for its layout to
            // fully commit (this crate's own multi-frame text/intrinsic-sizing convergence
            // characteristic -- see `layout::system::run`'s doc comments), visible as the
            // whole table collapsing for a frame before refilling. A position key instead
            // reuses the same row entities across a page/search change -- only their content
            // (name, avatar, role, ...) updates in place, and their already-settled layout
            // carries over rather than restarting from zero.
            children.add_key(format!("row-{row_pos}"));
        }

        // Pad a short last page back up to a full page's worth of rows -- see
        // `add_filler_row`'s doc comment for why.
        if total_pages > 1 {
            for filler_index in shown_count..PAGE_SIZE {
                add_filler_row(&theme, &mut children, compact);
                children.add_key(format!("filler-{filler_index}"));
            }
        }
    }

    if total_pages > 1 {
        children
            .add::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    justify_content: Some(WidgetAlignContent::Center),
                    padding: Edge::all(theme::space_sm(&theme)),
                    border_radius: Corner::all(0.0)
                        .bottom_left(theme::radius_md(&theme))
                        .bottom_right(theme::radius_md(&theme)),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<Pagination>((Pagination {
                    page,
                    page_count: total_pages,
                },)),
            ))
            .observe(
                current_widget,
                |trigger: On<Change<PaginationChanged>>, mut user_page: ResMut<UserPage>| {
                    user_page.0 = trigger.data.page;
                },
            );
        children.add_key("pagination");
    }

    children.apply(current_widget.as_parent());
}

#[allow(clippy::too_many_arguments)]
fn add_user_row(
    theme: &Theme,
    children: &mut WidgetChildren,
    current_widget: CurrentWidget,
    index: usize,
    is_last: bool,
    user: &UserRecord,
    compact: bool,
) {
    let role = user.role.clone();
    let active = user.active;
    let avatar_color = user.avatar_color;
    let name = user.name.clone();
    let email = user.email.clone();
    let email_for_menu = user.email.clone();

    let row_children = WidgetChildren::default()
        // User (avatar + name/email) -- expands to fill whatever space the fixed-width
        // columns below don't use (see `header_cell_flex`'s doc comment).
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Units::Pixels(0.0),
                min_width: USER_COLUMN_MIN_WIDTH.into(),
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Avatar>((
                    Avatar {
                        initials: initials(&name),
                        color: Some(avatar_color),
                        size: 30.0,
                    },
                    WoodpeckerStyle {
                        margin: Edge::all(0.0).right(theme::space_sm(theme)),
                        ..Default::default()
                    },
                ))
                .with_key("avatar")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        flex_direction: WidgetFlexDirection::Column,
                        ..Default::default()
                    },
                    WidgetChildren::default()
                        .with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                font_size: 14.0,
                                color: theme::text_primary(theme),
                                text_wrap: TextWrap::None,
                                ..Default::default()
                            },
                            WidgetRender::Text { content: name },
                        ))
                        .with_key("name")
                        .with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                font_size: 12.0,
                                color: theme::text_muted(theme),
                                text_wrap: TextWrap::None,
                                ..Default::default()
                            },
                            WidgetRender::Text { content: email },
                        ))
                        .with_key("email"),
                ))
                .with_key("identity"),
        ))
        .with_key("user")
        // Role dropdown -- live, applies immediately.
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: 160.0.into(),
                flex_shrink: 0.0,
                ..Default::default()
            },
            WidgetChildren::default().with_child::<Dropdown>((Dropdown {
                list: ROLES.iter().map(|r| r.to_string()).collect(),
                current_value: role,
            },)),
        ))
        .with_observe(
            current_widget,
            move |trigger: On<Change<DropdownChanged>>, mut users: ResMut<Users>| {
                if let Some(user) = users.0.get_mut(index) {
                    user.role = trigger.data.value.clone();
                }
            },
        )
        .with_key("role")
        // Active status -- a clickable badge (Toggle has no externally-settable initial
        // state, so a badge driven straight off `Users` is what actually reflects the data).
        // Wrapped in a fixed-width, non-shrinking cell so it actually lines up under the
        // "Status" header instead of hugging the badge's own (variable) natural width.
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: 120.0.into(),
                flex_shrink: 0.0,
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Badge>((status_badge(active), Pickable::default()))
                .with_observe(
                    current_widget,
                    move |_trigger: On<Pointer<Click>>, mut users: ResMut<Users>| {
                        if let Some(user) = users.0.get_mut(index) {
                            user.active = !user.active;
                        }
                    },
                )
                .with_key("badge"),
        ))
        .with_key("status")
        // Edit button -- same fixed-width-cell treatment as "status" above, to line up under
        // the (unlabeled) actions header column.
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: 80.0.into(),
                flex_shrink: 0.0,
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<WButton>((
                    WButton,
                    theme::secondary_button_styles(theme),
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: 13.0,
                            // Not left to white -- `secondary_button_styles`'s background is
                            // `bg_input` (light in light mode), so this label needs its own
                            // theme-correct color.
                            color: theme::text_primary(theme),
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: "Edit".into(),
                        },
                    )),
                ))
                .with_observe(
                    current_widget,
                    move |_trigger: On<Pointer<Click>>, mut editing: ResMut<EditingUser>| {
                        editing.0 = Some(index);
                    },
                )
                .with_key("button"),
        ))
        .with_key("actions");

    // Zebra striping for scanability, and a rounded bottom on the last row so a flush table
    // still matches the rounded card it sits inside.
    let stripe = if index % 2 == 1 {
        theme::bg_card_hover(theme).with_alpha(0.35)
    } else {
        Color::NONE
    };
    let row_bundle = (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            align_items: Some(WidgetAlignItems::Center),
            background_color: stripe,
            border_radius: if is_last {
                Corner::all(0.0)
                    .bottom_left(theme::radius_md(theme))
                    .bottom_right(theme::radius_md(theme))
            } else {
                Corner::all(0.0)
            },
            padding: Edge::all(if compact {
                theme::space_xs(theme)
            } else {
                theme::space_sm(theme) + 2.0
            })
            .left(theme::space_md(theme))
            .right(theme::space_md(theme)),
            gap: (theme::space_md(theme).into(), 0.0.into()),
            border: if is_last {
                Edge::all(0.0)
            } else {
                Edge::all(0.0).bottom(1.0)
            },
            border_color: theme::border(theme),
            ..Default::default()
        },
        WidgetRender::Quad,
        row_children,
    );

    // Right-click the row for the same actions the "Edit" button and status badge expose,
    // plus a "Copy email" shortcut -- purely additive, doesn't change the existing left-click
    // affordances above.
    children
        .add::<Menu>((
            Menu {
                items: vec![
                    "Edit".into(),
                    if active { "Deactivate" } else { "Activate" }.into(),
                    "Copy email".into(),
                ],
            },
            // `Menu` defaults to `Auto` width -- without this override, the row Element's own
            // `width: 100%` (and its User column's `flex_grow`) has nothing real to size
            // against, since an `Auto` (hug-content) parent has no "100%" of its own to hand
            // down and no spare space for `flex_grow` to distribute. Matches `row_bundle`'s own
            // width so the row actually spans the full column-header width instead of shrinking
            // to roughly its fixed-width columns' combined size.
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                ..Default::default()
            },
            PassedChildren(WidgetChildren::default().with_child::<Element>(row_bundle)),
        ))
        .observe(
            current_widget,
            move |trigger: On<Change<MenuItemSelected>>,
                  mut editing: ResMut<EditingUser>,
                  mut users: ResMut<Users>,
                  mut queue: ResMut<ToastQueue>| {
                match trigger.data.index {
                    0 => editing.0 = Some(index),
                    1 => {
                        if let Some(user) = users.0.get_mut(index) {
                            user.active = !user.active;
                        }
                    }
                    _ => {
                        queue.push(format!("Copied {email_for_menu}"), BadgeVariant::Info);
                    }
                }
            },
        );
}

/// An empty, invisible row-height spacer -- used to pad a short last page (fewer than
/// `PAGE_SIZE` matches) back up to a full page's worth of rows.
///
/// Without this, the table's total content height shrinks whenever the current page has fewer
/// rows than a full one (routine on the last page whenever the match count isn't a multiple of
/// `PAGE_SIZE`), which shrinks the surrounding `ScrollBox`'s scrollable content height for
/// that one page -- visibly moving/resizing the scrollbar and, if scrolled down at all,
/// snapping the scroll offset back to fit the new (shorter) range. Reusing a real row's exact
/// padding/min-height (rather than a computed pixel constant) keeps this pixel-identical to an
/// actual row's height without depending on a hardcoded formula that could drift out of sync
/// with `add_user_row`'s own styling.
fn add_filler_row(theme: &Theme, children: &mut WidgetChildren, compact: bool) {
    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            min_height: 30.0.into(),
            padding: Edge::all(if compact {
                theme::space_xs(theme)
            } else {
                theme::space_sm(theme) + 2.0
            })
            .left(theme::space_md(theme))
            .right(theme::space_md(theme)),
            border: Edge::all(0.0).bottom(1.0),
            border_color: theme::border(theme),
            ..Default::default()
        },
        WidgetRender::Quad,
    ));
}

fn status_badge(active: bool) -> Badge {
    Badge {
        label: if active { "Active" } else { "Inactive" }.into(),
        variant: if active {
            BadgeVariant::Success
        } else {
            BadgeVariant::Neutral
        },
    }
}

/// The edit/detail modal for whichever user (if any) `EditingUser` currently points at.
/// Watches both `EditingUser` (open/closed + which row) and `Users` (latest data), plus
/// `Theme` so a theme swap recolors it.
#[derive(Widget, Component, Reflect, Clone, Default, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render_user_editor)]
#[require(
    WidgetChildren,
    WoodpeckerStyle,
    WatchedResource<EditingUser>,
    WatchedResource<Users>,
    WatchedResource<Theme>
)]
pub struct UserEditorModal;

/// Caches the last real `UserRecord` this modal displayed, so its ~250ms close fade keeps
/// showing that content instead of an empty placeholder. `EditingUser` resets to `None` (and
/// `index` along with it) the instant "Done"/a row's close action is clicked -- well before
/// `Modal`'s own close transition finishes -- and `render_user_editor` re-renders on that same
/// `EditingUser` change (it's a `WatchedResource`) while the transition is still playing.
/// Without this, `index.and_then(...)` falls through to a brand-new blank `UserRecord` mid-fade,
/// visibly blanking the form fields and resizing the modal around the now-shorter content.
#[derive(Component, Default, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct EditorFormCache {
    last_user: Option<UserRecord>,
}

fn render_user_editor(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &WatchedResource<EditingUser>,
        &WatchedResource<Users>,
        &WatchedResource<Theme>,
        &mut WidgetChildren,
    )>,
    mut cache_query: Query<&mut EditorFormCache>,
) {
    let Ok((editing, users, watched_theme, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let theme = watched_theme.0;

    let index = editing.0 .0;
    let live_user = index.and_then(|i| users.0.get(i).cloned());

    let cache_entity = hooks.use_state(&mut commands, *current_widget, EditorFormCache::default());
    let Ok(mut cache) = cache_query.get_mut(cache_entity) else {
        return;
    };
    if let Some(live_user) = live_user.clone() {
        cache.last_user = Some(live_user);
    }

    let user = live_user
        .or_else(|| cache.last_user.clone())
        .unwrap_or(UserRecord {
            name: String::new(),
            email: String::new(),
            role: ROLES[0].into(),
            active: false,
            avatar_color: theme::accent(&theme),
            rating: 0.0,
            joined: CalendarDate::new(2024, 1, 1),
            notes: String::new(),
        });

    *children = WidgetChildren::default();
    children.add::<Modal>((
        Modal {
            visible: index.is_some(),
            title: "User details".into(),
            min_size: Vec2::new(380.0, 300.0),
            ..Default::default()
        },
        PassedChildren(
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        flex_direction: WidgetFlexDirection::Column,
                        padding: Edge::all(theme::space_md(&theme)),
                        ..Default::default()
                    },
                    build_editor_form(&theme, *current_widget, index, &user),
                ))
                .with_key("form"),
        ),
    ));
    children.apply(current_widget.as_parent());
}

fn build_editor_form(
    theme: &Theme,
    current_widget: CurrentWidget,
    index: Option<usize>,
    user: &UserRecord,
) -> WidgetChildren {
    WidgetChildren::default()
        .with_child::<Element>(form_row(
            theme,
            "Name",
            form_text_input(theme, user.name.clone()),
        ))
        .with_key("name")
        .with_child::<Element>(form_row(
            theme,
            "Email",
            form_text_input(theme, user.email.clone()),
        ))
        .with_key("email")
        .with_child::<Element>(form_row(
            theme,
            "Role",
            form_role_dropdown(theme, user.role.clone()),
        ))
        .with_observe(
            current_widget,
            move |trigger: On<Change<DropdownChanged>>, mut users: ResMut<Users>| {
                let Some(i) = index else { return };
                if let Some(user) = users.0.get_mut(i) {
                    user.role = trigger.data.value.clone();
                }
            },
        )
        .with_key("role")
        .with_child::<Element>(form_row(
            theme,
            "Joined",
            (
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<DatePicker>((
                    DatePicker {
                        selected_date: Some(user.joined),
                        initial_view: user.joined,
                        placeholder: "Pick a date".into(),
                        ..Default::default()
                    },
                    DatePickerStyles::from_theme(theme),
                )),
            ),
        ))
        .with_observe(
            current_widget,
            move |trigger: On<Change<DateChanged>>, mut users: ResMut<Users>| {
                let Some(i) = index else { return };
                if let Some(user) = users.0.get_mut(i) {
                    user.joined = trigger.data.date;
                }
            },
        )
        .with_key("joined")
        .with_child::<Element>(form_row(
            theme,
            "Active",
            (
                Element,
                WoodpeckerStyle::default(),
                WidgetChildren::default().with_child::<Badge>((
                    status_badge(user.active),
                    Pickable::default(),
                    BadgeStyles::from_theme(theme),
                )),
            ),
        ))
        .with_observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut users: ResMut<Users>| {
                let Some(i) = index else { return };
                if let Some(user) = users.0.get_mut(i) {
                    user.active = !user.active;
                }
            },
        )
        .with_key("active")
        .with_child::<Element>(form_row(
            theme,
            "Performance",
            (
                Element,
                WoodpeckerStyle::default(),
                WidgetChildren::default().with_child::<Rating>((
                    Rating {
                        value: user.rating,
                        max: 5,
                        read_only: false,
                    },
                    RatingStyles::from_theme(theme),
                )),
            ),
        ))
        .with_observe(
            current_widget,
            move |trigger: On<Change<RatingChanged>>, mut users: ResMut<Users>| {
                let Some(i) = index else { return };
                if let Some(user) = users.0.get_mut(i) {
                    user.rating = trigger.data.value;
                }
            },
        )
        .with_key("rating")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Column,
                margin: Edge::all(0.0)
                    .top(theme::space_sm(theme))
                    .bottom(theme::space_sm(theme)),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    theme::label_style(theme),
                    WidgetRender::Text {
                        content: "Notes".into(),
                    },
                ))
                .with_key("label")
                // Clips the note to a fixed 3-line-ish window rather than letting an
                // arbitrarily long note blow out the modal's height.
                .with_child::<Clip>((
                    Clip,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        height: 46.0.into(),
                        ..Default::default()
                    },
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: 12.0,
                            color: theme::text_secondary(theme),
                            text_wrap: TextWrap::WordOrGlyph,
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: user.notes.clone(),
                        },
                    )),
                ))
                .with_key("notes-clip"),
        ))
        .with_key("notes")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Column,
                margin: Edge::all(0.0)
                    .top(theme::space_sm(theme))
                    .bottom(theme::space_sm(theme)),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    theme::label_style(theme),
                    WidgetRender::Text {
                        content: "Account history".into(),
                    },
                ))
                .with_key("label")
                .with_child::<Timeline>((
                    Timeline {
                        items: vec![
                            TimelineItem {
                                title: "Account created".into(),
                                subtitle: Some("Onboarded via invite".into()),
                                dot_color: None,
                            },
                            TimelineItem {
                                title: "Role changed".into(),
                                subtitle: Some(format!("Now {}", user.role)),
                                dot_color: None,
                            },
                            TimelineItem {
                                title: if user.active {
                                    "Active".into()
                                } else {
                                    "Deactivated".into()
                                },
                                subtitle: None,
                                dot_color: if user.active {
                                    None
                                } else {
                                    Some(theme::danger(theme))
                                },
                            },
                        ],
                    },
                    TimelineStyles::from_theme(theme),
                ))
                .with_key("timeline"),
        ))
        .with_key("history")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                justify_content: Some(WidgetAlignContent::End),
                margin: Edge::all(0.0).top(theme::space_lg(theme)),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<WButton>((
                WButton,
                theme::primary_button_styles(theme),
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 13.0,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "Done".into(),
                    },
                )),
            )),
        ))
        .with_observe(
            current_widget,
            |_trigger: On<Pointer<Click>>, mut editing: ResMut<EditingUser>| {
                editing.0 = None;
            },
        )
        .with_key("actions")
}

fn form_row(
    theme: &Theme,
    label: &'static str,
    control: impl Bundle + Clone,
) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            align_items: Some(WidgetAlignItems::Center),
            margin: Edge::all(0.0).bottom(theme::space_sm(theme)),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                theme::label_style(theme),
                WidgetRender::Text {
                    content: label.into(),
                },
            ))
            .with_key("label")
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_grow: 1.0,
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<Element>(control),
            ))
            .with_key("control"),
    )
}

fn form_text_input(theme: &Theme, value: String) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<TextBox>((
            TextBox {
                initial_value: value,
                ..Default::default()
            },
            TextboxStyles::from_theme(theme),
        )),
    )
}

fn form_role_dropdown(theme: &Theme, value: String) -> impl Bundle + Clone {
    (
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            ..Default::default()
        },
        WidgetChildren::default().with_child::<Dropdown>((
            Dropdown {
                list: ROLES.iter().map(|r| r.to_string()).collect(),
                current_value: value,
            },
            DropdownStyles::from_theme(theme),
        )),
    )
}

pub fn build_users_page(theme: &Theme, current_widget: CurrentWidget) -> WidgetChildren {
    let content = WidgetChildren::default()
        .with_child::<Element>((
            Element,
            theme::section_title_style(theme),
            WidgetRender::Text {
                content: "Users".into(),
            },
        ))
        .with_key("header")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                justify_content: Some(WidgetAlignContent::SpaceBetween),
                align_items: Some(WidgetAlignItems::Center),
                margin: Edge::all(0.0).bottom(theme::space_sm(theme)),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<RoleFilterChips>(RoleFilterChips)
                .with_key("chips")
                .with_child::<ToggleButton>((
                    ToggleButton { selected: false },
                    PassedChildren(WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: 13.0,
                            // Not left to `WoodpeckerStyle::default()` -- its `color` is
                            // `Color::WHITE`, fine against dark mode's toggle background,
                            // nearly invisible against light mode's pale one.
                            color: theme::text_primary(theme),
                            ..Default::default()
                        },
                        WidgetRender::Text {
                            content: "Compact".into(),
                        },
                    ))),
                ))
                .with_observe(
                    current_widget,
                    |trigger: On<Change<ToggleButtonChanged>>, mut compact: ResMut<CompactRows>| {
                        compact.0 = trigger.data.selected;
                    },
                )
                .with_key("compact-toggle"),
        ))
        .with_key("toolbar")
        .with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Column,
                // No padding: the table handles its own inset and corner rounding so it
                // sits flush inside this card.
                padding: Edge::all(0.0),
                ..theme::card_style(theme)
            },
            WidgetRender::Quad,
            WidgetChildren::default()
                .with_child::<UsersTable>(UsersTable)
                .with_key("table"),
        ))
        .with_key("panel")
        .with_child::<UserEditorModal>(UserEditorModal)
        .with_key("editor");

    theme::scrollable_page(content)
}
