use crate::theme;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use scraper::{ElementRef, Html, node::Node};

const RENDER_WIDTH: usize = 70;

pub fn render_html(html: &str) -> Vec<Line<'static>> {
    if html.trim().is_empty() {
        return vec![Line::from(Span::styled(
            "(no content)",
            Style::default().fg(theme::TX4),
        ))];
    }

    let doc = Html::parse_fragment(html);
    let mut out: Vec<Line<'static>> = Vec::new();
    walk_children(doc.root_element(), &mut out, false, 0);

    // strip leading/trailing blank lines
    while out.first().map(|l| l.spans.is_empty()).unwrap_or(false) {
        out.remove(0);
    }
    while out.last().map(|l| l.spans.is_empty()).unwrap_or(false) {
        out.pop();
    }

    out
}

// ── Tree walker ────────────────────────────────────────────────────

fn walk_children(el: ElementRef, out: &mut Vec<Line<'static>>, in_bq: bool, list_depth: usize) {
    for child in el.children() {
        match child.value() {
            Node::Text(t) => {
                let text = normalize_ws(&t.text);
                if !text.is_empty() {
                    let spans = vec![(text, Style::default().fg(theme::TX2))];
                    emit_wrapped(out, spans, RENDER_WIDTH, in_bq);
                    out.push(Line::default());
                }
            }
            Node::Element(_) => {
                if let Some(el) = ElementRef::wrap(child) {
                    render_block(el, out, in_bq, list_depth);
                }
            }
            _ => {}
        }
    }
}

fn render_block(el: ElementRef, out: &mut Vec<Line<'static>>, in_bq: bool, list_depth: usize) {
    let tag = el.value().name();

    match tag {
        "p" => {
            let spans = collect_inline(el);
            if !spans.is_empty() {
                emit_wrapped(out, spans, RENDER_WIDTH, in_bq);
                out.push(Line::default());
            }
        }

        "div" | "article" | "section" | "main" | "header" | "footer" | "aside" => {
            if has_block_children(el) {
                walk_children(el, out, in_bq, list_depth);
            } else {
                let spans = collect_inline(el);
                if !spans.is_empty() {
                    emit_wrapped(out, spans, RENDER_WIDTH, in_bq);
                    out.push(Line::default());
                }
            }
        }

        "h1" => emit_heading(el, out, 1),
        "h2" => emit_heading(el, out, 2),
        "h3" => emit_heading(el, out, 3),
        "h4" | "h5" | "h6" => emit_heading(el, out, 4),

        "blockquote" => {
            out.push(Line::default());
            walk_children(el, out, true, list_depth);
            // last blank line already added by paragraph emitter
            out.push(Line::default());
        }

        "pre" => {
            emit_code_block(el, out, in_bq);
            out.push(Line::default());
        }

        "ul" => {
            emit_list(el, out, in_bq, list_depth, false);
            out.push(Line::default());
        }
        "ol" => {
            emit_list(el, out, in_bq, list_depth, true);
            out.push(Line::default());
        }

        "hr" => {
            let line = "─".repeat(48);
            out.push(Line::from(Span::styled(line, Style::default().fg(theme::BORDER2))));
            out.push(Line::default());
        }

        "br" => out.push(Line::default()),

        "figure" => {
            walk_children(el, out, in_bq, list_depth);
        }
        "figcaption" => {
            let text = plain_text(el);
            if !text.is_empty() {
                let prefix = if in_bq { bq_prefix() } else { vec![] };
                let mut spans = prefix;
                spans.push(Span::styled(
                    text,
                    Style::default().fg(theme::TX4).add_modifier(Modifier::ITALIC),
                ));
                out.push(Line::from(spans));
                out.push(Line::default());
            }
        }

        "img" => {
            let alt = el.value().attr("alt").unwrap_or("image");
            let prefix = if in_bq { bq_prefix() } else { vec![] };
            let mut spans = prefix;
            spans.push(Span::styled(
                format!("[{}]", if alt.is_empty() { "image" } else { alt }),
                Style::default().fg(theme::TX4),
            ));
            out.push(Line::from(spans));
        }

        "table" => {
            emit_table(el, out, in_bq);
            out.push(Line::default());
        }

        // inline elements appearing at block level
        "span" | "strong" | "em" | "b" | "i" | "a" | "code" | "s" | "del" => {
            let spans = collect_inline(el);
            if !spans.is_empty() {
                emit_wrapped(out, spans, RENDER_WIDTH, in_bq);
                out.push(Line::default());
            }
        }

        // ignore
        "script" | "style" | "noscript" | "iframe" | "form" | "input"
        | "button" | "meta" | "link" | "select" | "textarea" => {}

        // recurse into unknown containers
        _ => walk_children(el, out, in_bq, list_depth),
    }
}

// ── Headings ───────────────────────────────────────────────────────

fn emit_heading(el: ElementRef, out: &mut Vec<Line<'static>>, level: u8) {
    let text = plain_text(el);
    if text.is_empty() {
        return;
    }
    out.push(Line::default());
    match level {
        1 => {
            let bar = "━".repeat(text.chars().count().min(RENDER_WIDTH));
            out.push(Line::from(Span::styled(
                text,
                Style::default().fg(theme::TX).add_modifier(Modifier::BOLD),
            )));
            out.push(Line::from(Span::styled(bar, Style::default().fg(theme::ACCENT))));
        }
        2 => {
            out.push(Line::from(vec![
                Span::styled("▍ ", Style::default().fg(theme::ACCENT)),
                Span::styled(text, Style::default().fg(theme::TX).add_modifier(Modifier::BOLD)),
            ]));
        }
        3 => {
            out.push(Line::from(vec![
                Span::styled("  ▸ ", Style::default().fg(theme::TX3)),
                Span::styled(text, Style::default().fg(theme::TX).add_modifier(Modifier::BOLD)),
            ]));
        }
        _ => {
            out.push(Line::from(Span::styled(
                text,
                Style::default().fg(theme::TX2).add_modifier(Modifier::BOLD),
            )));
        }
    }
    out.push(Line::default());
}

// ── Code blocks ────────────────────────────────────────────────────

fn emit_code_block(el: ElementRef, out: &mut Vec<Line<'static>>, in_bq: bool) {
    // content may be inside a nested <code> element
    let text: String = el.text().collect::<Vec<_>>().join("");
    let text = text.trim_matches('\n');

    let bar: String = "─".repeat(50);
    let bq: Vec<Span<'static>> = if in_bq { bq_prefix() } else { vec![] };

    let mut top = bq.clone();
    top.push(Span::styled(bar.clone(), Style::default().fg(theme::BORDER)));
    out.push(Line::from(top));

    for line in text.lines() {
        let mut row = bq.clone();
        row.push(Span::styled(
            format!("  {}", line),
            Style::default().fg(theme::TX2),
        ));
        out.push(Line::from(row));
    }

    let mut bot = bq;
    bot.push(Span::styled(bar, Style::default().fg(theme::BORDER)));
    out.push(Line::from(bot));
}

// ── Lists ──────────────────────────────────────────────────────────

fn emit_list(el: ElementRef, out: &mut Vec<Line<'static>>, in_bq: bool, depth: usize, ordered: bool) {
    let indent_width = (depth + 1) * 2;
    let indent: String = " ".repeat(indent_width);
    let mut counter: usize = 1;

    for child in el.children() {
        let Some(item) = ElementRef::wrap(child) else { continue };
        if item.value().name() != "li" {
            continue;
        }

        let bullet_markers = ["•", "◦", "▸"];
        let marker = if ordered {
            format!("{}{}. ", indent, counter)
        } else {
            format!("{}{} ", indent, bullet_markers[depth.min(2)])
        };
        counter += 1;

        let marker_len = marker.chars().count();
        let cont_indent: String = " ".repeat(marker_len);
        let item_width = RENDER_WIDTH.saturating_sub(marker_len);

        // collect only inline content of <li> (skip nested lists)
        let spans = collect_inline_skip_block(item);
        let wrapped = wrap_tokens(spans, item_width);

        for (i, line_spans) in wrapped.into_iter().enumerate() {
            let mut row: Vec<Span<'static>> = if in_bq { bq_prefix() } else { vec![] };
            if i == 0 {
                row.push(Span::styled(marker.clone(), Style::default().fg(theme::ACCENT)));
            } else {
                row.push(Span::raw(cont_indent.clone()));
            }
            row.extend(line_spans);
            out.push(Line::from(row));
        }

        // handle nested lists
        for child in item.children() {
            let Some(nested) = ElementRef::wrap(child) else { continue };
            let nested_tag = nested.value().name();
            if nested_tag == "ul" || nested_tag == "ol" {
                emit_list(nested, out, in_bq, depth + 1, nested_tag == "ol");
            }
        }
    }
}

// ── Tables (simplified) ────────────────────────────────────────────

fn emit_table(el: ElementRef, out: &mut Vec<Line<'static>>, in_bq: bool) {
    // collect header row and data rows as plain text
    let mut rows: Vec<Vec<String>> = Vec::new();

    for row_el in el.select(&scraper::Selector::parse("tr").unwrap()) {
        let cells: Vec<String> = row_el
            .select(&scraper::Selector::parse("th, td").unwrap())
            .map(|c| plain_text(c))
            .collect();
        if !cells.is_empty() {
            rows.push(cells);
        }
    }

    if rows.is_empty() {
        return;
    }

    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let col_width = (RENDER_WIDTH.saturating_sub(cols + 1)) / cols.max(1);

    let bq: Vec<Span<'static>> = if in_bq { bq_prefix() } else { vec![] };

    for (i, row) in rows.iter().enumerate() {
        let mut line = bq.clone();
        let style = if i == 0 {
            Style::default().fg(theme::TX).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TX2)
        };

        for (j, cell) in row.iter().enumerate() {
            if j > 0 {
                line.push(Span::styled("  ", Style::default().fg(theme::BORDER)));
            }
            let truncated = truncate(cell, col_width);
            line.push(Span::styled(truncated, style));
        }
        out.push(Line::from(line));

        if i == 0 {
            let mut sep = bq.clone();
            sep.push(Span::styled(
                "─".repeat(RENDER_WIDTH.min(60)),
                Style::default().fg(theme::BORDER),
            ));
            out.push(Line::from(sep));
        }
    }
}

// ── Inline content collector ───────────────────────────────────────

fn collect_inline(el: ElementRef) -> Vec<(String, Style)> {
    collect_inline_inner(el, Style::default().fg(theme::TX2))
}

fn collect_inline_skip_block(el: ElementRef) -> Vec<(String, Style)> {
    // like collect_inline but skips nested block elements
    collect_inline_inner_sb(el, Style::default().fg(theme::TX2))
}

fn collect_inline_inner(el: ElementRef, style: Style) -> Vec<(String, Style)> {
    let mut result = Vec::new();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => {
                let text = normalize_ws(&t.text);
                if !text.is_empty() {
                    result.push((text, style));
                }
            }
            Node::Element(e) => {
                let Some(child_el) = ElementRef::wrap(child) else { continue };
                inline_element(e.name(), child_el, style, &mut result);
            }
            _ => {}
        }
    }
    result
}

fn collect_inline_inner_sb(el: ElementRef, style: Style) -> Vec<(String, Style)> {
    let block = ["p", "div", "h1", "h2", "h3", "h4", "h5", "h6",
                 "blockquote", "pre", "ul", "ol", "table", "hr", "figure"];
    let mut result = Vec::new();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => {
                let text = normalize_ws(&t.text);
                if !text.is_empty() {
                    result.push((text, style));
                }
            }
            Node::Element(e) => {
                if block.contains(&e.name()) { continue; }
                let Some(child_el) = ElementRef::wrap(child) else { continue };
                inline_element(e.name(), child_el, style, &mut result);
            }
            _ => {}
        }
    }
    result
}

fn inline_element(tag: &str, el: ElementRef, style: Style, result: &mut Vec<(String, Style)>) {
    match tag {
        "strong" | "b" => result.extend(collect_inline_inner(
            el,
            style.fg(theme::TX).add_modifier(Modifier::BOLD),
        )),
        "em" | "i" => result.extend(collect_inline_inner(
            el,
            style.add_modifier(Modifier::ITALIC),
        )),
        "code" => result.extend(collect_inline_inner(
            el,
            Style::default().fg(theme::ACCENT),
        )),
        "a" => result.extend(collect_inline_inner(
            el,
            style.fg(theme::ACCENT),
        )),
        "s" | "del" => result.extend(collect_inline_inner(
            el,
            style.fg(theme::TX4),
        )),
        "br" => result.push(("\n".into(), style)),
        "img" => {
            let alt = el.value().attr("alt").unwrap_or("");
            if !alt.is_empty() {
                result.push((format!("[{}]", alt), Style::default().fg(theme::TX4)));
            }
        }
        "script" | "style" | "noscript" => {}
        _ => result.extend(collect_inline_inner(el, style)),
    }
}

// ── Word wrap ──────────────────────────────────────────────────────

fn wrap_tokens(tokens: Vec<(String, Style)>, width: usize) -> Vec<Vec<Span<'static>>> {
    let mut lines: Vec<Vec<Span<'static>>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut current_width: usize = 0;

    for (text, style) in tokens {
        if text == "\n" {
            lines.push(std::mem::take(&mut current));
            current_width = 0;
            continue;
        }

        let words: Vec<&str> = text.split_ascii_whitespace().collect();
        for word in words {
            let wlen = word.chars().count();
            let need_space = current_width > 0;
            let total = current_width + if need_space { 1 } else { 0 } + wlen;

            if total > width && current_width > 0 {
                lines.push(std::mem::take(&mut current));
                current_width = 0;
            }

            if current_width > 0 {
                current.push(Span::raw(" "));
                current_width += 1;
            }

            current.push(Span::styled(word.to_string(), style));
            current_width += wlen;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

fn emit_wrapped(out: &mut Vec<Line<'static>>, spans: Vec<(String, Style)>, width: usize, in_bq: bool) {
    let effective = if in_bq { width.saturating_sub(4) } else { width };
    let wrapped = wrap_tokens(spans, effective);

    for line_spans in wrapped {
        if line_spans.is_empty() {
            continue;
        }
        if in_bq {
            let mut row = bq_prefix();
            row.extend(line_spans);
            out.push(Line::from(row));
        } else {
            out.push(Line::from(line_spans));
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn bq_prefix() -> Vec<Span<'static>> {
    vec![
        Span::styled("│", Style::default().fg(theme::ACCENT)),
        Span::styled("  ", Style::default()),
    ]
}

fn plain_text(el: ElementRef) -> String {
    normalize_ws(&el.text().collect::<String>())
}

fn normalize_ws(s: &str) -> String {
    let mut out = String::new();
    let mut last_space = true;
    for c in s.chars() {
        if c.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(c);
            last_space = false;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let mut t: String = chars[..max.saturating_sub(1)].iter().collect();
        t.push('…');
        t
    }
}

fn has_block_children(el: ElementRef) -> bool {
    const BLOCK: &[&str] = &[
        "p", "div", "h1", "h2", "h3", "h4", "h5", "h6",
        "blockquote", "pre", "ul", "ol", "table", "hr", "figure",
    ];
    el.children()
        .filter_map(ElementRef::wrap)
        .any(|c| BLOCK.contains(&c.value().name()))
}
