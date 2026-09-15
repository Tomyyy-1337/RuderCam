use ansi_to_tui::IntoText;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub fn parse_output_lines(line: String) -> Vec<Line<'static>> {
    let parsed_lines = match line.into_text() {
        Ok(text) => text.lines,
        Err(_) => vec![Line::from(Span::styled(
            line,
            Style::default().fg(Color::Gray),
        ))],
    };

    parsed_lines
        .into_iter()
        .map(|mut parsed| {
            for span in &mut parsed.spans {
                if span.style.fg.is_none() {
                    span.style = span.style.fg(Color::Gray);
                }
            }
            add_left_padding(&mut parsed);
            parsed
        })
        .collect()
}

fn add_left_padding(line: &mut Line<'static>) {
    line.spans.insert(0, Span::raw(" "));
}

fn split_words_and_spaces(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut in_space = false;

    for (i, c) in text.char_indices() {
        let is_space = c.is_whitespace();
        if i == 0 {
            in_space = is_space;
            continue;
        }
        if is_space != in_space {
            result.push(&text[start..i]);
            start = i;
            in_space = is_space;
        }
    }
    if start < text.len() {
        result.push(&text[start..]);
    }
    result
}

pub fn wrap_line(line: Line<'static>, width: usize) -> Vec<Line<'static>> {
    if width == 0 {
        return vec![line];
    }

    let total_width: usize = line.spans.iter().map(|s| s.width()).sum();
    if total_width <= width {
        return vec![line];
    }

    let mut acc_lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_width: usize = 0;

    for span in line.spans {
        let style = span.style;
        let text = span.content;
        let words = split_words_and_spaces(&text);

        for word in words {
            let word_span = Span::styled(word.to_string(), style);
            let word_width = word_span.width();

            if current_width + word_width <= width {
                current_spans.push(word_span);
                current_width += word_width;
            } else if word_width > width {
                for ch in word.chars() {
                    let ch_str = ch.to_string();
                    let ch_span = Span::styled(ch_str, style);
                    let ch_width = ch_span.width();

                    if current_width + ch_width > width && current_width > 0 {
                        acc_lines.push(Line::from(std::mem::take(&mut current_spans)));
                        current_width = 0;
                    }
                    current_spans.push(ch_span);
                    current_width += ch_width;
                }
            } else {
                if current_width > 0 {
                    let mut wrapped = std::mem::take(&mut current_spans);
                    if !acc_lines.is_empty() {
                        wrapped.insert(0, Span::raw("  "));
                    }
                    acc_lines.push(Line::from(wrapped));
                    current_width = 0;
                }
                if word.trim_start().is_empty() {
                    continue;
                }
                let word_span = Span::styled(word.to_string(), style);
                let word_width = word_span.width();
                current_spans.push(word_span);
                current_width += word_width;
            }
        }
    }

    if !current_spans.is_empty() {
        let mut wrapped = current_spans;
        if !acc_lines.is_empty() {
            wrapped.insert(0, Span::raw("  "));
        }
        acc_lines.push(Line::from(wrapped));
    }

    if acc_lines.is_empty() {
        vec![Line::default()]
    } else {
        acc_lines
    }
}
