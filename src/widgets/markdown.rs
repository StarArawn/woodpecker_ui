use crate::prelude::*;
use bevy::prelude::*;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};

/// Renders a block of markdown source as a column of styled widgets -- headings, paragraphs
/// (with inline `code` spans tinted a distinct color), fenced code blocks (syntax-highlighted
/// via [`crate::highlight`]), block quotes, bullet/numbered lists, and horizontal rules
/// (rendered as a [`Divider`]). Covers the common subset of CommonMark actually needed for
/// prose documentation; it does not attempt tables, images, or nested block quotes.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren)]
pub struct Markdown {
    /// The markdown source to render.
    pub content: String,
    /// The [`crate::highlight`] theme name used for fenced code blocks.
    pub code_theme: String,
}

impl Default for Markdown {
    fn default() -> Self {
        Self {
            content: String::new(),
            code_theme: "dracula".into(),
        }
    }
}

/// One run of inline text within a block -- `Plain` is ordinary prose, `Code` is the content of
/// an inline `` `code` `` span with the backticks stripped, kept separate so the caller can
/// render each run in a different color instead of showing the raw backticks.
enum Segment {
    Plain(String),
    Code(String),
}

fn push_plain(segments: &mut Vec<Segment>, s: &str) {
    if let Some(Segment::Plain(last)) = segments.last_mut() {
        last.push_str(s);
    } else {
        segments.push(Segment::Plain(s.to_string()));
    }
}

enum Block {
    Heading(HeadingLevel, Vec<Segment>),
    Paragraph(Vec<Segment>),
    CodeBlock { language: String, code: String },
    BlockQuote(Vec<Segment>),
    ListItem { ordered: Option<u64>, segments: Vec<Segment> },
    Rule,
}

fn parse_blocks(content: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut segments: Vec<Segment> = Vec::new();
    let mut code = String::new();
    let mut language = String::new();
    let mut in_code_block = false;
    let mut in_block_quote = false;
    let mut in_list_item = false;
    let mut list_ordered_next: Option<Option<u64>> = None;

    for event in Parser::new(content) {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code.clear();
                language = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                blocks.push(Block::CodeBlock {
                    language: std::mem::take(&mut language),
                    code: std::mem::take(&mut code),
                });
            }
            Event::Start(Tag::List(start)) => {
                list_ordered_next = Some(start);
            }
            Event::End(TagEnd::List(_)) => {
                list_ordered_next = None;
            }
            Event::Start(Tag::Item) => {
                in_list_item = true;
                segments.clear();
            }
            Event::End(TagEnd::Item) => {
                in_list_item = false;
                blocks.push(Block::ListItem {
                    ordered: list_ordered_next.flatten(),
                    segments: std::mem::take(&mut segments),
                });
                if let Some(Some(n)) = list_ordered_next {
                    list_ordered_next = Some(Some(n + 1));
                }
            }
            Event::Start(Tag::BlockQuote(_)) => {
                in_block_quote = true;
                segments.clear();
            }
            Event::Start(Tag::Heading { .. } | Tag::Paragraph) => {
                if !in_block_quote && !in_list_item {
                    segments.clear();
                }
            }
            Event::End(TagEnd::Heading(level)) => {
                blocks.push(Block::Heading(level, std::mem::take(&mut segments)));
            }
            Event::End(TagEnd::Paragraph) => {
                if !in_block_quote && !in_list_item {
                    blocks.push(Block::Paragraph(std::mem::take(&mut segments)));
                }
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                in_block_quote = false;
                blocks.push(Block::BlockQuote(std::mem::take(&mut segments)));
            }
            Event::Text(t) => {
                if in_code_block {
                    code.push_str(&t);
                } else {
                    push_plain(&mut segments, &t);
                }
            }
            Event::Code(t) => {
                segments.push(Segment::Code(t.to_string()));
            }
            Event::SoftBreak | Event::HardBreak => {
                push_plain(&mut segments, " ");
            }
            Event::Rule => {
                blocks.push(Block::Rule);
            }
            _ => {}
        }
    }

    blocks
}

fn build_rich_text(segments: &[Segment], plain_color: Color, code_color: Color) -> RichText {
    let mut rich_text = RichText::new();
    for segment in segments {
        rich_text = match segment {
            Segment::Plain(s) => rich_text.with_color_text(s, plain_color),
            Segment::Code(s) => rich_text.with_color_text(s, code_color),
        };
    }
    rich_text
}

const CODE_BACKGROUND: Srgba = Srgba::new(0.157, 0.165, 0.212, 1.0);
const CODE_BORDER: Srgba = Srgba::new(0.267, 0.278, 0.353, 1.0);

fn render(
    theme: Res<Theme>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&Markdown, &mut WoodpeckerStyle, &mut WidgetChildren)>,
) {
    let Ok((markdown, mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };
    let current_widget = *current_widget;

    *styles = WoodpeckerStyle {
        width: Units::Percentage(100.0),
        flex_direction: WidgetFlexDirection::Column,
        ..*styles
    };

    *children = WidgetChildren::default();
    for (index, block) in parse_blocks(&markdown.content).into_iter().enumerate() {
        match block {
            Block::Heading(level, segments) => {
                let font_size = match level {
                    HeadingLevel::H1 => theme.typography.h1,
                    HeadingLevel::H2 => theme.typography.h2,
                    HeadingLevel::H3 => theme.typography.h3,
                    _ => theme.typography.body_small,
                };
                children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size,
                        margin: Edge::all(0.0)
                            .top(theme.spacing.lg)
                            .bottom(theme.spacing.sm),
                        ..Default::default()
                    },
                    WidgetRender::RichText {
                        content: build_rich_text(&segments, Color::WHITE, theme.primary_light),
                    },
                ));
            }
            Block::Paragraph(segments) => {
                children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: theme.typography.body,
                        text_wrap: TextWrap::WordOrGlyph,
                        width: Units::Percentage(100.0),
                        margin: Edge::all(0.0).bottom(theme.spacing.md),
                        ..Default::default()
                    },
                    WidgetRender::RichText {
                        content: build_rich_text(
                            &segments,
                            theme.text.with_alpha(0.85),
                            theme.primary_light,
                        ),
                    },
                ));
            }
            Block::CodeBlock { language, code } => {
                let language = if language.is_empty() { "text" } else { &language };
                let code = code.trim_end_matches('\n').to_string();
                children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        padding: Edge::all(16.0),
                        margin: Edge::all(0.0).bottom(theme.spacing.md),
                        background_color: CODE_BACKGROUND.into(),
                        border_color: CODE_BORDER.into(),
                        border: Edge::all(1.0),
                        border_radius: Corner::all(8.0),
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            font_size: 13.0,
                            text_wrap: TextWrap::None,
                            ..Default::default()
                        },
                        WidgetRender::RichText {
                            content: RichText::from_hightlighted(
                                &code,
                                highlight(language, &code, &markdown.code_theme),
                            ),
                        },
                    )),
                ));
            }
            Block::BlockQuote(segments) => {
                children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        width: Units::Percentage(100.0),
                        padding: Edge::all(theme.spacing.md),
                        margin: Edge::all(0.0).bottom(theme.spacing.md),
                        background_color: theme.primary.with_alpha(0.08),
                        border_color: theme.primary.with_alpha(0.4),
                        border: Edge::all(0.0).left(3.0),
                        border_radius: Corner::all(theme.panel_radius),
                        ..Default::default()
                    },
                    WidgetRender::Quad,
                    WidgetChildren::default().with_child::<Element>((
                        Element,
                        WoodpeckerStyle {
                            width: Units::Percentage(100.0),
                            ..Default::default()
                        },
                        WidgetChildren::default().with_child::<Element>((
                            Element,
                            WoodpeckerStyle {
                                font_size: theme.typography.body_small,
                                text_wrap: TextWrap::WordOrGlyph,
                                ..Default::default()
                            },
                            WidgetRender::RichText {
                                content: build_rich_text(
                                    &segments,
                                    theme.text.with_alpha(0.85),
                                    theme.primary_light,
                                ),
                            },
                        )),
                    )),
                ));
            }
            Block::ListItem { ordered, segments } => {
                let prefix = match ordered {
                    Some(n) => format!("{n}. "),
                    None => "\u{2022} ".to_string(),
                };
                let mut full_segments = vec![Segment::Plain(prefix)];
                full_segments.extend(segments);
                children.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: theme.typography.body,
                        text_wrap: TextWrap::WordOrGlyph,
                        width: Units::Percentage(100.0),
                        margin: Edge::all(0.0).left(theme.spacing.md).bottom(theme.spacing.xs),
                        ..Default::default()
                    },
                    WidgetRender::RichText {
                        content: build_rich_text(
                            &full_segments,
                            theme.text.with_alpha(0.85),
                            theme.primary_light,
                        ),
                    },
                ));
            }
            Block::Rule => {
                children.add::<Divider>((
                    Divider::default(),
                    WoodpeckerStyle {
                        margin: Edge::all(0.0).top(theme.spacing.sm).bottom(theme.spacing.md),
                        ..Default::default()
                    },
                ));
            }
        }
        children.add_key(index.to_string());
    }

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flattens segments back into one string for assertions that don't care about the
    /// plain/code split.
    fn flatten(segments: &[Segment]) -> String {
        segments
            .iter()
            .map(|s| match s {
                Segment::Plain(s) | Segment::Code(s) => s.as_str(),
            })
            .collect()
    }

    #[test]
    fn headings_and_paragraph_parse_in_order() {
        let blocks = parse_blocks("# Title\n\nSome body text.\n\n## Subhead");
        assert!(
            matches!(&blocks[0], Block::Heading(HeadingLevel::H1, s) if flatten(s) == "Title")
        );
        assert!(matches!(&blocks[1], Block::Paragraph(s) if flatten(s) == "Some body text."));
        assert!(
            matches!(&blocks[2], Block::Heading(HeadingLevel::H2, s) if flatten(s) == "Subhead")
        );
    }

    #[test]
    fn inline_code_span_becomes_its_own_segment() {
        let blocks = parse_blocks("Call `foo()` to start.");
        let Block::Paragraph(segments) = &blocks[0] else {
            panic!("expected a paragraph");
        };
        assert!(matches!(&segments[0], Segment::Plain(s) if s == "Call "));
        assert!(matches!(&segments[1], Segment::Code(s) if s == "foo()"));
        assert!(matches!(&segments[2], Segment::Plain(s) if s == " to start."));
    }

    #[test]
    fn fenced_code_block_keeps_its_language_and_body_separate_from_prose() {
        let blocks = parse_blocks("```rust\nlet x = 1;\n```");
        assert!(matches!(
            &blocks[0],
            Block::CodeBlock { language, code }
            if language == "rust" && code == "let x = 1;\n"
        ));
    }

    #[test]
    fn unordered_and_ordered_list_items_are_distinguished() {
        let blocks = parse_blocks("- one\n- two\n\n1. first\n2. second");
        assert!(
            matches!(&blocks[0], Block::ListItem { ordered: None, segments } if flatten(segments) == "one")
        );
        assert!(
            matches!(&blocks[1], Block::ListItem { ordered: None, segments } if flatten(segments) == "two")
        );
        assert!(
            matches!(&blocks[2], Block::ListItem { ordered: Some(1), segments } if flatten(segments) == "first")
        );
        assert!(
            matches!(&blocks[3], Block::ListItem { ordered: Some(2), segments } if flatten(segments) == "second")
        );
    }

    #[test]
    fn block_quote_and_rule_parse() {
        let blocks = parse_blocks("> a note\n\n---\n\nafter");
        assert!(matches!(&blocks[0], Block::BlockQuote(s) if flatten(s) == "a note"));
        assert!(matches!(&blocks[1], Block::Rule));
        assert!(matches!(&blocks[2], Block::Paragraph(s) if flatten(s) == "after"));
    }

    #[test]
    fn build_rich_text_offsets_each_segment_correctly() {
        let segments = vec![
            Segment::Plain("Call ".to_string()),
            Segment::Code("foo()".to_string()),
            Segment::Plain(" to start.".to_string()),
        ];
        let rich = build_rich_text(&segments, Color::WHITE, Color::BLACK);
        assert_eq!(rich.text, "Call foo() to start.");
        assert_eq!(rich.highlighted.color_text[0].range, 0..5);
        assert_eq!(rich.highlighted.color_text[1].range, 5..10);
        assert_eq!(rich.highlighted.color_text[2].range, 10..20);
    }
}
