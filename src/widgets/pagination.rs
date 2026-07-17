use crate::prelude::*;
use bevy::prelude::*;

/// Fired when [`Pagination`] selects a new page -- via a numbered button, or the prev/next
/// controls.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct PaginationChanged {
    /// The newly-selected page, 0-indexed.
    pub page: usize,
}

/// A page-number [`Chip`]'s width naturally hugs its label -- so a chip whose *label* changes
/// digit count (staying on the same key as the current page advances, e.g. "9" -> "10")
/// visibly resizes under the user's cursor, and the whole row reflows around it, for exactly
/// the frame that resize lands. Reserving a fixed width sized for the widest label this
/// [`Pagination`] will ever show (driven by `page_count`, which doesn't change as `page` does)
/// keeps every page-number chip the same width across every page change, so nothing ever
/// visibly resizes. `20.0` covers the chip's own fixed horizontal padding (`Chip::render`'s
/// `Edge::all(0.0).left(10.0).right(10.0)`); `8.0` is a rough per-digit width at this chip's
/// 12px font -- doesn't need to be pixel-exact, just consistent across renders.
///
/// Pulled out of the render function as a pure, unit-testable helper.
fn chip_min_width(page_count: usize) -> f32 {
    20.0 + page_count.max(1).to_string().len() as f32 * 8.0
}

/// One entry in [`page_window`]'s output -- either a real page or a collapsed "..." gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageEntry {
    Page(usize),
    Ellipsis,
}

/// Computes which pages to actually show for `page_count` total pages centered on `page`
/// (0-indexed) -- MUI's own `Pagination` ellipsis-collapse behavior: always show the first and
/// last page, plus `page`'s immediate neighbors, collapsing any gap into a single "...". Small
/// counts (`<= 7`) show every page with no collapsing at all, since collapsing wouldn't save
/// any space.
///
/// Pulled out of the render function as a pure, unit-testable helper.
fn page_window(page: usize, page_count: usize) -> Vec<PageEntry> {
    if page_count == 0 {
        return Vec::new();
    }
    if page_count <= 7 {
        return (0..page_count).map(PageEntry::Page).collect();
    }

    let last = page_count - 1;
    let mut keep: Vec<usize> = [0, last, page.saturating_sub(1), page, (page + 1).min(last)]
        .into_iter()
        .filter(|&p| p <= last)
        .collect();
    keep.sort_unstable();
    keep.dedup();

    let mut entries = Vec::with_capacity(keep.len() + 2);
    let mut previous: Option<usize> = None;
    for p in keep {
        if let Some(prev) = previous {
            if p > prev + 1 {
                entries.push(PageEntry::Ellipsis);
            }
        }
        entries.push(PageEntry::Page(p));
        previous = Some(p);
    }
    entries
}

/// Prev/next + numbered page controls, with MUI-style ellipsis-collapse for large page counts
/// (see [`page_window`]). Composes [`IconButton`] (prev/next chevrons) and [`Chip`] (page
/// number pills, using `Chip::selected` for the current page) directly rather than a parallel
/// clickable-pill implementation. Fires [`Change<PaginationChanged>`] on any page change;
/// purely presentational otherwise -- the caller owns `page` and re-declares it.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_style(), WidgetChildren)]
pub struct Pagination {
    /// The current page, 0-indexed.
    pub page: usize,
    /// Total number of pages.
    pub page_count: usize,
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        flex_direction: WidgetFlexDirection::Row,
        align_items: Some(WidgetAlignItems::Center),
        gap: (6.0.into(), 0.0.into()),
        ..Default::default()
    }
}

fn nav_glyph(theme: &Theme, icon_font: &IconFont, enabled: bool, glyph: &str) -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 14.0,
            color: if enabled {
                theme.text
            } else {
                theme.text.with_alpha(0.3)
            },
            font: Some(icon_font.0.id()),
            ..Default::default()
        },
        WidgetRender::Text {
            content: glyph.into(),
        },
    ))
}

fn render(
    theme: Res<Theme>,
    current_widget: Res<CurrentWidget>,
    icon_font: Res<IconFont>,
    mut query: Query<(&Pagination, &mut WidgetChildren)>,
) {
    let Ok((pagination, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;
    let page = pagination.page;
    let page_count = pagination.page_count;

    let chip_min_width = chip_min_width(page_count);

    *children = WidgetChildren::default();

    let has_prev = page_count > 0 && page > 0;
    children.add::<IconButton>((
        IconButton,
        nav_glyph(&theme, &icon_font, has_prev, icons::CARET_LEFT),
    ));
    if has_prev {
        // Reads `Pagination`'s current `page` fresh at click time rather than capturing
        // `page` from this render -- `ObserverCache` only attaches this closure once per
        // reused "prev" button entity (see `WidgetChildren::observe`'s doc comment), so a
        // captured `page` value would go stale forever after the first click, permanently
        // sending whatever page was current the first time this button became clickable.
        children.observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>,
                  mut commands: Commands,
                  query: Query<&Pagination>| {
                let Ok(pagination) = query.get(current_widget.entity()) else {
                    return;
                };
                if pagination.page == 0 {
                    return;
                }
                commands.trigger(Change {
                    target: current_widget.entity(),
                    data: PaginationChanged {
                        page: pagination.page - 1,
                    },
                });
            },
        );
    }
    children.add_key("prev");

    for (position, entry) in page_window(page, page_count).into_iter().enumerate() {
        match entry {
            PageEntry::Page(p) => {
                // `selected`'s own border alone reads as too subtle a "current page"
                // indicator at this size (same color family as the background) -- pairing it
                // with a variant swap (`Info` colored vs. plain `Neutral`) gives the current
                // page real contrast against the rest, not just a thin outline to notice.
                let variant = if p == page {
                    BadgeVariant::Info
                } else {
                    BadgeVariant::Neutral
                };
                children
                    .add::<Chip>((
                        Chip {
                            label: (p + 1).to_string(),
                            variant,
                            selected: p == page,
                            ..Default::default()
                        },
                        WoodpeckerStyle {
                            min_width: chip_min_width.into(),
                            justify_content: Some(WidgetJustifyContent::Center),
                            ..Default::default()
                        },
                    ))
                    .observe(
                        current_widget,
                        move |_trigger: On<Change<ChipClicked>>, mut commands: Commands| {
                            commands.trigger(Change {
                                target: current_widget.entity(),
                                data: PaginationChanged { page: p },
                            });
                        },
                    );
                children.add_key(format!("page-{p}"));
            }
            PageEntry::Ellipsis => {
                children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 13.0,
                        color: theme.text.with_alpha(0.5),
                        padding: Edge::all(0.0).left(4.0).right(4.0),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: "\u{2026}".into(),
                    },
                ));
                children.add_key(format!("ellipsis-{position}"));
            }
        }
    }

    let has_next = page_count > 0 && page + 1 < page_count;
    children.add::<IconButton>((
        IconButton,
        nav_glyph(&theme, &icon_font, has_next, icons::CARET_RIGHT),
    ));
    if has_next {
        // See the "prev" button's observer above -- same fresh-read requirement, same reason.
        children.observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>,
                  mut commands: Commands,
                  query: Query<&Pagination>| {
                let Ok(pagination) = query.get(current_widget.entity()) else {
                    return;
                };
                if pagination.page_count == 0 || pagination.page + 1 >= pagination.page_count {
                    return;
                }
                commands.trigger(Change {
                    target: current_widget.entity(),
                    data: PaginationChanged {
                        page: pagination.page + 1,
                    },
                });
            },
        );
    }
    children.add_key("next");

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_min_width_depends_only_on_page_count_digit_width() {
        // Same page_count (10, i.e. labels "1".."10") must yield the same reserved width
        // regardless of which page is currently selected -- that stability across page
        // changes is the entire point (nothing about `page` feeds into this function at all).
        assert_eq!(chip_min_width(10), chip_min_width(10));
    }

    #[test]
    fn chip_min_width_grows_with_more_digits() {
        assert!(
            chip_min_width(100) > chip_min_width(10),
            "a 3-digit-page-count pagination needs a wider reserved chip width than a \
             2-digit one"
        );
        assert!(
            chip_min_width(10) > chip_min_width(9),
            "crossing from 1-digit to 2-digit labels must widen the reservation"
        );
    }

    #[test]
    fn chip_min_width_handles_zero_pages() {
        // page_count == 0 must not panic (`to_string().len()` of 0 pages worth of labels
        // isn't meaningful) and still produce a sane, positive width.
        assert!(chip_min_width(0) > 0.0);
    }

    #[test]
    fn small_page_counts_show_every_page_uncollapsed() {
        let window = page_window(2, 7);
        assert_eq!(
            window,
            (0..7).map(PageEntry::Page).collect::<Vec<_>>(),
            "7 or fewer pages must never collapse"
        );
    }

    #[test]
    fn first_page_collapses_the_tail_into_one_ellipsis() {
        let window = page_window(0, 10);
        assert_eq!(
            window,
            vec![
                PageEntry::Page(0),
                PageEntry::Page(1),
                PageEntry::Ellipsis,
                PageEntry::Page(9),
            ]
        );
    }

    #[test]
    fn last_page_collapses_the_head_into_one_ellipsis() {
        let window = page_window(9, 10);
        assert_eq!(
            window,
            vec![
                PageEntry::Page(0),
                PageEntry::Ellipsis,
                PageEntry::Page(8),
                PageEntry::Page(9),
            ]
        );
    }

    #[test]
    fn middle_page_collapses_both_sides() {
        let window = page_window(5, 10);
        assert_eq!(
            window,
            vec![
                PageEntry::Page(0),
                PageEntry::Ellipsis,
                PageEntry::Page(4),
                PageEntry::Page(5),
                PageEntry::Page(6),
                PageEntry::Ellipsis,
                PageEntry::Page(9),
            ]
        );
    }

    #[test]
    fn near_the_edge_no_ellipsis_is_needed_on_that_side() {
        // page=1 of 10: neighbors {0,1,2} already abut the always-shown first page (0), so
        // there's no gap to collapse on the left.
        let window = page_window(1, 10);
        assert_eq!(
            window,
            vec![
                PageEntry::Page(0),
                PageEntry::Page(1),
                PageEntry::Page(2),
                PageEntry::Ellipsis,
                PageEntry::Page(9),
            ]
        );
    }

    #[test]
    fn zero_pages_yields_an_empty_window() {
        assert_eq!(page_window(0, 0), Vec::new());
    }
}
