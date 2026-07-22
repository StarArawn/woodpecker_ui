use crate::prelude::*;
use bevy::prelude::*;

/// One tile in a [`Masonry`] grid. Unlike [`ImageListItem`], `height` is required up front --
/// `Masonry::render()` must assign every tile's absolute position in a single declarative pass,
/// before taffy has computed any layout, so there is no synchronous "measure this arbitrary
/// subtree's natural height" hook to fall back on mid-render (the only such hook this crate has,
/// `LayoutMeasure`, is wired up for text glyphs only). Real image-gallery masonry grids
/// typically know each image's aspect ratio up front for exactly this reason.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct MasonryItem {
    /// The image to display.
    pub handle: Handle<Image>,
    /// The tile's rendered height, in logical pixels, at [`Masonry::column_width`].
    pub height: f32,
    /// An optional caption, overlaid on a scrim at the tile's bottom edge.
    pub label: Option<String>,
}

/// [`Masonry`]'s themed styles.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct MasonryStyles {
    /// Caption text color.
    pub label_color: Color,
    /// Caption scrim background color.
    pub label_background: Color,
    /// Tile corner radius.
    pub corner_radius: f32,
}

impl Default for MasonryStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for MasonryStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            label_color: Color::WHITE,
            label_background: Color::BLACK.with_alpha(0.55),
            corner_radius: theme.control_radius,
        }
    }
}

/// Assigns each item (in order) to whichever column currently has the smallest accumulated
/// height -- the classic shortest-column-first masonry bin-packing algorithm (e.g. Pinterest-
/// style grids). Returns each item's `(column_index, y_offset)`, in the same order as `heights`.
///
/// A pure function (no ECS/widget dependency) so it's exhaustively unit-testable on its own.
pub fn pack_masonry(heights: &[f32], columns: u16, gap: f32) -> Vec<(u16, f32)> {
    let columns = (columns as usize).max(1);
    let mut column_heights = vec![0.0_f32; columns];
    heights
        .iter()
        .map(|&height| {
            let (index, &y) = column_heights
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.total_cmp(b))
                .expect("columns is clamped to at least 1");
            column_heights[index] = y + height + gap;
            (index as u16, y)
        })
        .collect()
}

/// A Pinterest-style grid that packs variable-height image tiles into the shortest available
/// column, rather than a uniform row/column grid -- see [`ImageList`] for the fixed-tile-size
/// case, which uses [`GridTemplate`] directly instead of this bin-packing layout.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_style(), WidgetChildren, MasonryStyles)]
pub struct Masonry {
    /// The tiles, in packing order (not necessarily the order they end up visually top-to-bottom
    /// within a column, since each item goes to whichever column is shortest *at that point*).
    pub items: Vec<MasonryItem>,
    /// Number of columns.
    pub columns: u16,
    /// Each column's fixed width, in logical pixels.
    pub column_width: f32,
    /// Gap between tiles (both axes), in logical pixels.
    pub gap: f32,
}

impl Default for Masonry {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            columns: 3,
            column_width: 120.0,
            gap: 8.0,
        }
    }
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        position: WidgetPosition::Relative,
        ..Default::default()
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &Masonry,
        &MasonryStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
) {
    let Ok((masonry, styles, mut woodpecker_style, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let heights: Vec<f32> = masonry.items.iter().map(|item| item.height).collect();
    let placements = pack_masonry(&heights, masonry.columns, masonry.gap);

    let columns = masonry.columns.max(1) as f32;
    woodpecker_style.width =
        (columns * masonry.column_width + (columns - 1.0) * masonry.gap).into();
    woodpecker_style.height = placements
        .iter()
        .zip(masonry.items.iter())
        .map(|((_, y), item)| y + item.height)
        .fold(0.0_f32, f32::max)
        .into();

    *children = WidgetChildren::default();
    for (index, (item, (column, y))) in masonry.items.iter().zip(placements.iter()).enumerate() {
        let mut tile_children = WidgetChildren::default();
        if let Some(label) = &item.label {
            tile_children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    position: WidgetPosition::Absolute,
                    left: 0.0.into(),
                    right: 0.0.into(),
                    bottom: 0.0.into(),
                    background_color: styles.label_background,
                    padding: Edge::all(4.0),
                    ..Default::default()
                },
                WidgetRender::Quad,
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 11.0,
                        color: styles.label_color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: label.clone(),
                    },
                )),
            ));
            tile_children.add_key("caption");
        }

        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                left: (*column as f32 * (masonry.column_width + masonry.gap)).into(),
                top: (*y).into(),
                width: masonry.column_width.into(),
                height: item.height.into(),
                border_radius: Corner::all(styles.corner_radius),
                ..Default::default()
            },
            WidgetRender::Image {
                handle: item.handle.clone(),
            },
            tile_children,
        ));
        children.add_key(format!("tile-{index}"));
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_masonry_uses_shortest_column_first() {
        // col0=0, col1=0 -> tie, first (col0) wins: item0 -> col0, col0=10
        // col0=10, col1=0 -> item1 -> col1, col1=20
        // col0=10, col1=20 -> item2 -> col0, col0=25
        let placements = pack_masonry(&[10.0, 20.0, 15.0], 2, 0.0);
        assert_eq!(placements, vec![(0, 0.0), (1, 0.0), (0, 10.0)]);
    }

    #[test]
    fn pack_masonry_accumulates_gap_within_a_single_column() {
        let placements = pack_masonry(&[5.0, 5.0, 5.0], 1, 2.0);
        assert_eq!(placements, vec![(0, 0.0), (0, 7.0), (0, 14.0)]);
    }

    #[test]
    fn pack_masonry_clamps_zero_columns_to_one() {
        let placements = pack_masonry(&[10.0, 10.0], 0, 0.0);
        assert_eq!(placements, vec![(0, 0.0), (0, 10.0)]);
    }

    #[test]
    fn pack_masonry_of_no_items_is_empty() {
        assert!(pack_masonry(&[], 3, 8.0).is_empty());
    }

    #[test]
    fn corner_radius_tracks_the_passed_in_theme() {
        // Mirrors `ImageListStyles`'s equivalent test: `label_color`/`label_background` are
        // deliberately theme-independent (a translucent scrim needs to work over arbitrary
        // image content), so only `corner_radius` varies -- and it maps directly to
        // `theme.control_radius`, which happens to be identical between `Theme::dark()`/
        // `Theme::light()`, so this checks the direct mapping rather than dark != light.
        let theme = Theme::dark();
        let styles = MasonryStyles::from_theme(&theme);
        assert_eq!(styles.corner_radius, theme.control_radius);
    }

    #[test]
    fn default_masonry_is_a_three_column_grid() {
        let masonry = Masonry::default();
        assert_eq!(masonry.columns, 3);
        assert!(masonry.items.is_empty());
    }
}
