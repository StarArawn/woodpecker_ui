use crate::{prelude::*, widgets::colors};
use bevy::prelude::*;

use super::DevtoolsState;

fn row_label(text: &str) -> (Element, WoodpeckerStyle, WidgetRender) {
    (
        Element,
        WoodpeckerStyle {
            flex_grow: 1.0,
            font_size: 12.0,
            color: colors::TEXT.with_alpha(0.85),
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: text.into(),
        },
    )
}

fn row_container_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        align_items: Some(WidgetAlignItems::Center),
        padding: Edge::all(0.0).left(8.0).right(8.0).top(2.0).bottom(2.0),
        gap: (8.0.into(), 0.0.into()),
        ..Default::default()
    }
}

/// A section heading between field groups (Size, Spacing, Color, ...) so a ~30-row form
/// doesn't read as one undifferentiated list.
fn section(parent: &mut WidgetChildren, key: &str, title: &str) {
    parent.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            padding: Edge::all(0.0).left(8.0).top(10.0).bottom(2.0),
            font_size: 11.0,
            color: colors::PRIMARY_LIGHT,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: title.into(),
        },
    ));
    parent.add_key(key);
}

/// A single editable numeric row, wired to write straight back into `target`'s own
/// `WoodpeckerStyle` component via `set`. Keyed by `{target}-{key}`, not just `key` -- both
/// `NumberInput` and `ColorPicker` only read their initial value once, at mount, so a row must
/// remount (become a fresh entity) whenever the *selection* changes, or it would keep showing
/// a stale value left over from whatever was previously selected.
#[allow(clippy::too_many_arguments)]
fn numeric_row<S>(
    current_widget_val: CurrentWidget,
    target: Entity,
    parent: &mut WidgetChildren,
    key: &str,
    label_text: &str,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    set: S,
) where
    S: Fn(&mut WoodpeckerStyle, f32) + Send + Sync + Clone + 'static,
{
    let mut row = WidgetChildren::default();
    row.add::<Element>(row_label(label_text));
    row.add_key("label");
    row.add::<NumberInput>(NumberInput {
        value,
        min,
        max,
        step,
        ..Default::default()
    })
    .observe(
        current_widget_val,
        move |trigger: On<Change<NumberInputChanged>>, mut query: Query<&mut WoodpeckerStyle>| {
            if let Ok(mut style) = query.get_mut(target) {
                set(&mut style, trigger.data.value);
            }
        },
    );
    row.add_key("value");

    parent.add::<Element>((Element, row_container_style(), row));
    parent.add_key(format!("{target}-{key}"));
}

fn swatch_style(value: Color) -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: 36.0.into(),
        height: 20.0.into(),
        background_color: value,
        border: Edge::all(1.0),
        border_color: colors::BORDER,
        border_radius: Corner::all(4.0),
        ..Default::default()
    }
}

/// A color field row: a small swatch that opens `ColorPicker` in a `Popover` rather than
/// embedding the (large -- ~320x400px) picker inline in the row. `open_color_field` is the
/// currently-open row's own key (`DevtoolsState::open_color_field`, at most one open at a
/// time), compared against this row's own key to decide whether its popover is visible.
///
/// `PopoverPlacement::Left` (opens toward screen-center, away from the panel's own docked-right
/// edge) rather than the more common `Bottom` -- the picker is ~320px wide, wider than this
/// panel itself, so opening downward from a swatch near the panel's right edge would push most
/// of it off-screen.
#[allow(clippy::too_many_arguments)]
fn color_row<S>(
    current_widget_val: CurrentWidget,
    target: Entity,
    parent: &mut WidgetChildren,
    key: &str,
    label_text: &str,
    value: Color,
    open_color_field: Option<&str>,
    set: S,
) where
    S: Fn(&mut WoodpeckerStyle, Color) + Send + Sync + Clone + 'static,
{
    let mut row = WidgetChildren::default();
    row.add::<Element>(row_label(label_text));
    row.add_key("label");

    let row_key = format!("{target}-{key}");
    let is_open = open_color_field == Some(row_key.as_str());
    let toggle_key = row_key.clone();

    let mut swatch = WidgetChildren::default();
    swatch.add::<Element>((
        Element,
        swatch_style(value),
        WidgetRender::Quad,
        // Without this, the swatch is invisible to bevy_picking entirely -- it never receives
        // `Pointer<Click>` (or any other pointer event), so the observer below never fires.
        Pickable::default(),
    ));
    swatch
        .observe(
            current_widget_val,
            move |mut trigger: On<Pointer<Click>>, mut state: ResMut<DevtoolsState>| {
                trigger.propagate(false);
                state.open_color_field =
                    if state.open_color_field.as_deref() == Some(toggle_key.as_str()) {
                        None
                    } else {
                        Some(toggle_key.clone())
                    };
            },
        )
        .hover_cursor(current_widget_val, SystemCursorIcon::Pointer);

    row.add::<Popover>(PopoverBundle {
        popover: Popover {
            visible: is_open,
            placement: PopoverPlacement::Left,
        },
        trigger: PassedChildren(swatch),
        content: PopoverContent(
            WidgetChildren::default().with_child::<ColorPicker>(ColorPicker {
                initial_color: value,
            }),
        ),
        ..Default::default()
    })
    .observe(
        current_widget_val,
        move |trigger: On<Change<ColorPickerChanged>>, mut query: Query<&mut WoodpeckerStyle>| {
            if let Ok(mut style) = query.get_mut(target) {
                set(&mut style, trigger.data.color);
            }
        },
    );
    row.add_key("value");

    parent.add::<Element>((Element, row_container_style(), row));
    parent.add_key(row_key);
}

const PX_MIN: f32 = -100_000.0;
const PX_MAX: f32 = 100_000.0;

/// Builds an editable form for `style`'s most commonly-tweaked fields (size, spacing, color,
/// text) directly into `parent`, wired to write straight back into `target`'s own
/// `WoodpeckerStyle` component -- edits apply live, the same frame they're made.
///
/// Deliberately not a fully generic reflection-based editor: structural fields (`position`,
/// `display`, `flex_direction`, alignment, grid placement, and so on) are left out of this v1
/// and still show up in `dump::dump_entity`'s read-only field-list fallback instead. Every
/// field edited here is treated as a plain pixel value (`Units::Pixels`) -- editing a
/// `Percentage` or `Auto`-valued field switches it to a fixed pixel value, which is the
/// obviously-intended behavior for a "type a number, see it move" tool.
pub(crate) fn build(
    current_widget_val: CurrentWidget,
    target: Entity,
    style: &WoodpeckerStyle,
    open_color_field: Option<&str>,
    parent: &mut WidgetChildren,
) {
    section(parent, "sec-size", "Size");
    numeric_row(
        current_widget_val,
        target,
        parent,
        "width",
        "width",
        style.width.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.width = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "height",
        "height",
        style.height.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.height = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "min_width",
        "min width",
        style.min_width.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.min_width = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "min_height",
        "min height",
        style.min_height.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.min_height = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "max_width",
        "max width",
        style.max_width.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.max_width = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "max_height",
        "max height",
        style.max_height.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.max_height = Units::Pixels(v),
    );

    section(parent, "sec-position", "Position offset");
    numeric_row(
        current_widget_val,
        target,
        parent,
        "left",
        "left",
        style.left.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.left = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "right",
        "right",
        style.right.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.right = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "top",
        "top",
        style.top.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.top = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "bottom",
        "bottom",
        style.bottom.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.bottom = Units::Pixels(v),
    );

    section(parent, "sec-padding", "Padding");
    numeric_row(
        current_widget_val,
        target,
        parent,
        "padding-top",
        "top",
        style.padding.top.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.padding.top = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "padding-right",
        "right",
        style.padding.right.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.padding.right = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "padding-bottom",
        "bottom",
        style.padding.bottom.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.padding.bottom = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "padding-left",
        "left",
        style.padding.left.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.padding.left = Units::Pixels(v),
    );

    section(parent, "sec-margin", "Margin");
    numeric_row(
        current_widget_val,
        target,
        parent,
        "margin-top",
        "top",
        style.margin.top.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.margin.top = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "margin-right",
        "right",
        style.margin.right.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.margin.right = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "margin-bottom",
        "bottom",
        style.margin.bottom.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.margin.bottom = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "margin-left",
        "left",
        style.margin.left.value_or(0.0),
        PX_MIN,
        PX_MAX,
        1.0,
        |s, v| s.margin.left = Units::Pixels(v),
    );

    section(parent, "sec-border", "Border width");
    numeric_row(
        current_widget_val,
        target,
        parent,
        "border-top",
        "top",
        style.border.top.value_or(0.0),
        0.0,
        PX_MAX,
        1.0,
        |s, v| s.border.top = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "border-right",
        "right",
        style.border.right.value_or(0.0),
        0.0,
        PX_MAX,
        1.0,
        |s, v| s.border.right = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "border-bottom",
        "bottom",
        style.border.bottom.value_or(0.0),
        0.0,
        PX_MAX,
        1.0,
        |s, v| s.border.bottom = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "border-left",
        "left",
        style.border.left.value_or(0.0),
        0.0,
        PX_MAX,
        1.0,
        |s, v| s.border.left = Units::Pixels(v),
    );

    section(parent, "sec-radius", "Border radius");
    numeric_row(
        current_widget_val,
        target,
        parent,
        "radius-tl",
        "top left",
        style.border_radius.top_left.value_or(0.0),
        0.0,
        PX_MAX,
        1.0,
        |s, v| s.border_radius.top_left = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "radius-tr",
        "top right",
        style.border_radius.top_right.value_or(0.0),
        0.0,
        PX_MAX,
        1.0,
        |s, v| s.border_radius.top_right = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "radius-bl",
        "bottom left",
        style.border_radius.bottom_left.value_or(0.0),
        0.0,
        PX_MAX,
        1.0,
        |s, v| s.border_radius.bottom_left = Units::Pixels(v),
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "radius-br",
        "bottom right",
        style.border_radius.bottom_right.value_or(0.0),
        0.0,
        PX_MAX,
        1.0,
        |s, v| s.border_radius.bottom_right = Units::Pixels(v),
    );

    section(parent, "sec-text", "Text");
    numeric_row(
        current_widget_val,
        target,
        parent,
        "font_size",
        "font size",
        style.font_size,
        1.0,
        200.0,
        1.0,
        |s, v| s.font_size = v,
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "opacity",
        "opacity",
        style.opacity,
        0.0,
        1.0,
        0.05,
        |s, v| s.opacity = v,
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "flex_grow",
        "flex grow",
        style.flex_grow,
        0.0,
        100.0,
        1.0,
        |s, v| s.flex_grow = v,
    );
    numeric_row(
        current_widget_val,
        target,
        parent,
        "flex_shrink",
        "flex shrink",
        style.flex_shrink,
        0.0,
        100.0,
        1.0,
        |s, v| s.flex_shrink = v,
    );

    section(parent, "sec-color", "Color");
    color_row(
        current_widget_val,
        target,
        parent,
        "background_color",
        "background",
        style.background_color,
        open_color_field,
        |s, v| s.background_color = v,
    );
    color_row(
        current_widget_val,
        target,
        parent,
        "border_color",
        "border",
        style.border_color,
        open_color_field,
        |s, v| s.border_color = v,
    );
    color_row(
        current_widget_val,
        target,
        parent,
        "text_color",
        "text",
        style.color,
        open_color_field,
        |s, v| s.color = v,
    );
}
