use crate::{
    prelude::*,
    widgets::badge::{BadgeStyles, BadgeVariant},
};
use bevy::prelude::*;

/// Fired when a [`Chip`]'s delete control is clicked. `Chip` doesn't remove itself -- same
/// "caller owns the lifecycle" model [`crate::widgets::Alert`]/[`crate::widgets::ToastQueue`]
/// use -- the caller's own observer despawns/hides the chip entity in response.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct ChipDeleted;

/// Fired when a [`Chip`]'s own body (not its delete control) is clicked -- lets a caller use a
/// `Chip` as a toggleable filter/selection control by flipping `Chip::selected` in response.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct ChipClicked;

/// Optional leading content (e.g. an [`Avatar`](super::Avatar) or icon) shown before a
/// [`Chip`]'s label. Absent by default.
#[derive(Component, Default, PartialEq, Clone)]
pub struct ChipLeading(pub WidgetChildren);

/// A small, pill-shaped input/filter/tag control -- reuses [`BadgeStyles`]/[`BadgeVariant`]
/// directly rather than a near-identical color scheme, matching [`crate::widgets::Alert`]'s own
/// choice. Optionally shows a leading slot (via [`ChipLeading`]) and/or a trailing delete glyph
/// (`deletable: true`, firing [`Change<ChipDeleted>`]); the chip's own body always fires
/// [`Change<ChipClicked>`] on click, letting a caller drive a `selected` toggle for filter-chip
/// use without a separate "selectable" flag.
///
/// The delete control is a plain clickable [`icons::X`] glyph, not a true `IconButton` --
/// same reasoning as `Alert`'s dismiss control.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, Pickable, BadgeStyles, WidgetRender = WidgetRender::Quad)]
pub struct Chip {
    /// The text shown inside the chip.
    pub label: String,
    /// The semantic color variant (reused from [`BadgeVariant`]).
    pub variant: BadgeVariant,
    /// Whether to show a trailing delete glyph.
    pub deletable: bool,
    /// Whether this chip is in its "selected" state -- drawn with a border in the variant's
    /// text color. The caller owns this (typically flipped in a [`Change<ChipClicked>`]
    /// observer); `Chip` has no internal selection state of its own.
    pub selected: bool,
}

fn render(
    current_widget: Res<CurrentWidget>,
    icon_font: Res<IconFont>,
    mut query: Query<(
        &Chip,
        &BadgeStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
        Option<&ChipLeading>,
    )>,
) {
    let Ok((chip, badge_styles, mut styles, mut children, leading)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let current_widget = *current_widget;
    let (background_color, text_color) = badge_styles.colors(chip.variant);

    styles.background_color = background_color;
    styles.border_radius = Corner::all(100.0);
    styles.padding = Edge::all(0.0).left(10.0).right(10.0).top(4.0).bottom(4.0);
    styles.border = Edge::all(if chip.selected { 2.0 } else { 0.0 });
    styles.border_color = text_color;
    styles.flex_direction = WidgetFlexDirection::Row;
    styles.align_items.get_or_insert(WidgetAlignItems::Center);

    *children = WidgetChildren::default();

    if let Some(leading) = leading {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                margin: Edge::all(0.0).right(6.0),
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            leading.0.clone(),
        ));
        children.add_key("leading");
    }

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 12.0,
            color: text_color,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: chip.label.clone(),
        },
    ));
    children.add_key("label");

    if chip.deletable {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: 12.0,
                color: text_color,
                margin: Edge::all(0.0).left(6.0),
                font: Some(icon_font.0.id()),
                ..Default::default()
            },
            WidgetRender::Text {
                content: icons::X.into(),
            },
            Pickable::default(),
        ));
        children.add_key("delete");
        children
            .hover_cursor(current_widget, SystemCursorIcon::Pointer)
            .observe(
                current_widget,
                move |mut trigger: On<Pointer<Click>>, mut commands: Commands| {
                    // Stops this click from also bubbling into the self_observe below --
                    // deleting a chip must not also fire `ChipClicked`.
                    trigger.propagate(false);
                    commands.trigger(Change {
                        target: current_widget.entity(),
                        data: ChipDeleted,
                    });
                },
            );
    }

    children
        .self_hover_cursor(current_widget, SystemCursorIcon::Pointer)
        .self_observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut commands: Commands| {
                commands.trigger(Change {
                    target: current_widget.entity(),
                    data: ChipClicked,
                });
            },
        );

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_chip_is_unselected_and_not_deletable() {
        let chip = Chip::default();
        assert!(!chip.selected);
        assert!(!chip.deletable);
        assert_eq!(chip.variant, BadgeVariant::Neutral);
    }
}
