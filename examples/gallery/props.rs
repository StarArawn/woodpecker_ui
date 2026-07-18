use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::type_docs::type_doc;

/// One documented field of a widget's own struct -- `name` and `type_str` should read exactly
/// as they appear in the struct's own source (e.g. `"selected"`, `"bool"`), `doc` is a short
/// (one clause, not a full sentence necessarily) description -- lift it from the field's own
/// `///` doc comment in source when one exists, don't invent unrelated text.
pub(crate) struct PropDoc {
    pub(crate) name: &'static str,
    pub(crate) type_str: &'static str,
    pub(crate) doc: &'static str,
}

/// Detail shown in the type-inspector `Modal` when a prop's type is clicked -- either a
/// struct's fields or an enum's variants, both represented as `PropDoc` (a variant's `name` is
/// the variant itself, `type_str` empty since it carries no value, `doc` its own doc comment).
pub(crate) struct TypeDoc {
    pub(crate) description: &'static str,
    pub(crate) fields: &'static [PropDoc],
}

/// Strips one layer of `Vec<...>`/`Option<...>` wrapping to get the "core" type name a prop's
/// `type_str` is built from -- e.g. `"Vec<ListItem>"` -> `"ListItem"`, `"Option<Color>"` ->
/// `"Color"`. Applied repeatedly by `linked_type_name` to unwrap nested cases like
/// `"Option<Vec<ListItem>>"`.
fn unwrap_generic(type_str: &'static str) -> &'static str {
    for wrapper in ["Vec<", "Option<"] {
        if let Some(inner) = type_str
            .strip_prefix(wrapper)
            .and_then(|s| s.strip_suffix('>'))
        {
            return inner;
        }
    }
    type_str
}

/// Resolves a prop's `type_str` to a cross-referenceable type name, if any -- unwraps
/// `Vec`/`Option` wrappers (up to twice, for the rare doubly-wrapped case) and checks the
/// result against [`type_doc`]. Returns the resolved (unwrapped) name, not the original
/// `type_str`, so callers can look it up directly. All `PropDoc::type_str` values are
/// `&'static str` literals, so every candidate here stays `'static` too.
fn linked_type_name(type_str: &'static str) -> Option<&'static str> {
    let once = unwrap_generic(type_str);
    let twice = unwrap_generic(once);
    [twice, once, type_str]
        .into_iter()
        .find(|name| type_doc(name).is_some())
}

/// Builds the row list for a props/fields table -- shared by the main story page's "Props"
/// section and each cross-referenced type's own body inside the "Related types" `Accordion`
/// (see `build_related_types_section`), so a field that itself references another documented
/// type gets the same treatment recursively. A prop's type column is tinted (not clickable --
/// see that its details live in the accordion below, not behind an interaction on this text)
/// when [`linked_type_name`] resolves it to a [`type_doc`] entry.
pub(crate) fn build_props_rows(theme: &Theme, props: &[PropDoc]) -> WidgetChildren {
    let mut rows = WidgetChildren::default();
    for prop in props {
        let linked = linked_type_name(prop.type_str).is_some();

        rows.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                flex_direction: WidgetFlexDirection::Row,
                align_items: Some(WidgetAlignItems::FlexStart),
                padding: Edge::all(0.0).bottom(theme.spacing.xs),
                ..Default::default()
            },
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: 200.0.into(),
                        flex_shrink: 0.0,
                        font_size: theme.typography.body_small,
                        color: theme.primary,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: prop.name.into(),
                    },
                ))
                .with_key("name")
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        flex_grow: 1.0,
                        flex_direction: WidgetFlexDirection::Column,
                        ..Default::default()
                    },
                    WidgetChildren::default()
                        .with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                font_size: theme.typography.body_small,
                                color: if linked {
                                    theme.primary_light
                                } else {
                                    theme.text.with_alpha(0.5)
                                },
                                text_wrap: TextWrap::None,
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                content: prop.type_str.into(),
                            },
                        ))
                        .with_key("type")
                        .with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                font_size: theme.typography.caption,
                                color: theme.text.with_alpha(0.65),
                                text_wrap: TextWrap::WordOrGlyph,
                                ..Default::default()
                            },
                            WidgetRender::Text {
                                content: prop.doc.into(),
                            },
                        ))
                        .with_key("doc"),
                ))
                .with_key("meta"),
        ));
        rows.add_key(prop.name);
    }
    rows
}

/// Builds a "Related Types" `Accordion` -- one collapsed `AccordionItem` per unique type
/// referenced (via [`linked_type_name`]) among `props`, expanding in place to show that type's
/// own description and field table (built with [`build_props_rows`] again, so a field that
/// itself references a further documented type nests the same way). Returns `None` when no
/// prop in `props` references a documented type, so the caller can skip the whole section.
pub(crate) fn build_related_types_section(theme: &Theme, props: &[PropDoc]) -> Option<WidgetChildren> {
    let mut names: Vec<&'static str> = Vec::new();
    for prop in props {
        if let Some(name) = linked_type_name(prop.type_str) {
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    if names.is_empty() {
        return None;
    }

    let mut items = WidgetChildren::default();
    for name in names {
        let doc = type_doc(name).expect("linked_type_name only returns names type_doc resolves");
        items
            .add::<AccordionItem>((
                AccordionItem {
                    key: name.into(),
                    label: name.into(),
                },
                PassedChildren(
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            width: Units::Percentage(100.0),
                            flex_direction: WidgetFlexDirection::Column,
                            ..Default::default()
                        },
                        WidgetChildren::default()
                            .with_child::<Element>((
                                Element,
                                WoodpeckerStyle {
                                    font_size: theme.typography.body_small,
                                    color: theme.text.with_alpha(0.75),
                                    margin: Edge::all(0.0).bottom(theme.spacing.sm),
                                    ..Default::default()
                                },
                                WidgetRender::Text {
                                    content: doc.description.into(),
                                },
                            ))
                            .with_key("description")
                            .with_child::<Element>((
                                Element,
                                WoodpeckerStyle {
                                    width: Units::Percentage(100.0),
                                    flex_direction: WidgetFlexDirection::Column,
                                    padding: Edge::all(theme.spacing.md),
                                    background_color: theme.dark_background,
                                    border_color: theme.border,
                                    border: Edge::all(1.0),
                                    border_radius: Corner::all(theme.panel_radius),
                                    ..Default::default()
                                },
                                WidgetRender::Quad,
                                build_props_rows(theme, doc.fields),
                            ))
                            .with_key("fields"),
                    )),
                ),
            ))
            .add_key(name);
    }

    Some(
        WidgetChildren::default().with_child::<Accordion>((
            Accordion {
                mode: AccordionMode::Multiple,
            },
            PassedChildren(items),
        )),
    )
}

