use crate::prelude::*;
use bevy::prelude::*;

/// One tile in an [`ImageList`].
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct ImageListItem {
    /// The image to display.
    pub handle: Handle<Image>,
    /// An optional caption, overlaid on a scrim at the tile's bottom edge.
    pub label: Option<String>,
}

/// [`ImageList`]'s themed styles.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ImageListStyles {
    /// Caption text color.
    pub label_color: Color,
    /// Caption scrim background color.
    pub label_background: Color,
    /// Tile corner radius.
    pub corner_radius: f32,
}

impl Default for ImageListStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ImageListStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            label_color: Color::WHITE,
            label_background: Color::BLACK.with_alpha(0.55),
            corner_radius: theme.control_radius,
        }
    }
}

/// A responsive grid of images with optional captions -- e.g. a media gallery, a product
/// thumbnail grid. Uses [`GridTemplate`] directly (a fixed column count, square tiles) rather
/// than a new layout primitive -- this crate's existing grid support already covers it.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = default_style(),
    WidgetChildren,
    GridTemplate,
    ImageListStyles
)]
pub struct ImageList {
    /// The tiles, in grid order (row-major).
    pub items: Vec<ImageListItem>,
    /// Number of columns.
    pub columns: u16,
    /// Each (square) tile's side length, in logical pixels.
    pub tile_size: f32,
    /// Gap between tiles, in logical pixels.
    pub gap: f32,
}

impl Default for ImageList {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            columns: 3,
            tile_size: 120.0,
            gap: 8.0,
        }
    }
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        display: WidgetDisplay::Grid,
        ..Default::default()
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &ImageList,
        &ImageListStyles,
        &mut WoodpeckerStyle,
        &mut GridTemplate,
        &mut WidgetChildren,
    )>,
) {
    let Ok((image_list, styles, mut woodpecker_style, mut grid, mut children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    woodpecker_style.gap = (image_list.gap.into(), image_list.gap.into());
    grid.columns = vec![GridTrackSize::Pixels(image_list.tile_size); image_list.columns as usize];
    grid.auto_rows = vec![GridTrackSize::Pixels(image_list.tile_size)];

    *children = WidgetChildren::default();
    for (index, item) in image_list.items.iter().enumerate() {
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
                width: image_list.tile_size.into(),
                height: image_list.tile_size.into(),
                position: WidgetPosition::Relative,
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
    fn corner_radius_tracks_the_passed_in_theme() {
        // `label_color`/`label_background` are deliberately theme-independent (a translucent
        // black scrim + white text reads correctly over arbitrary image content regardless of
        // app theme, matching MUI's own `ImageListItemBar`) -- `corner_radius` is the only
        // field that actually varies by theme, and `control_radius` itself happens to share
        // the same constant between `Theme::dark()`/`Theme::light()`, so this checks the
        // mapping directly rather than asserting dark != light.
        let theme = Theme::dark();
        let styles = ImageListStyles::from_theme(&theme);
        assert_eq!(styles.corner_radius, theme.control_radius);
    }

    #[test]
    fn default_image_list_is_a_three_column_grid() {
        let list = ImageList::default();
        assert_eq!(list.columns, 3);
        assert!(list.items.is_empty());
    }
}
