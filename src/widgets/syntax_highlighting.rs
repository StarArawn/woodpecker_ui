use crate::prelude::{ColorText, Highlighted};
use autumnus::{themes, FormatterOption, Options};

/// A function to generate syntax highlighting for parley.
pub fn highlight(language: &str, code: &str, theme: &str) -> Highlighted {
    let ansi = autumnus::highlight(
        code,
        Options {
            lang_or_file: Some(language),
            formatter: FormatterOption::Terminal {
                theme: themes::get(theme).ok(),
            },
        },
    );
    use ansi_parser::{AnsiParser, AnsiSequence, Output};
    let parsed: Vec<Output> = ansi.ansi_parse().collect();

    let mut highlighted = Highlighted::default();
    let mut current_color_text = ColorText::default();
    let mut start = 0;
    let mut end = 0;
    for item in parsed {
        match item {
            Output::TextBlock(text) => {
                end += text.len();
                current_color_text.range = start..end;
                highlighted.color_text.push(current_color_text);
                current_color_text = ColorText::default();
                start = end;
            }
            Output::Escape(ansi_sequence) => {
                if let AnsiSequence::SetGraphicsMode(items) = ansi_sequence {
                    let code = items[0];
                    if code == 38 {
                        let code2 = items[1];
                        if code2 == 2 {
                            let red = items[2];
                            let green = items[3];
                            let blue = items[4];
                            current_color_text.color =
                                bevy::prelude::Srgba::rgb_u8(red, green, blue).into();
                        }
                    }
                }
            }
        }
    }

    highlighted
}

#[test]
fn test_highlighting() {
    let code_to_highlight = r#"
use autumnus::{themes, FormatterOption, Options};

pub fn highlight(language: &str, code: &str) {
    let ansi = autumnus::highlight(
        code,
        Options {
            lang_or_file: Some(language),
            formatter: FormatterOption::Terminal {
                theme: themes::get("dracula").ok(),
            },
            ..Options::default()
        },
    );

    dbg!(ansi);
}
"#;

    highlight("rust", code_to_highlight, "dracula");
}
