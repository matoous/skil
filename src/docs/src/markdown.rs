use pulldown_cmark::{
    CodeBlockKind, Event, Options as MarkdownOptions, Parser as MarkdownParser, Tag, TagEnd,
    html as markdown_html,
};

pub fn strip_frontmatter(content: &str) -> &str {
    if !content.starts_with("---") {
        return content;
    }

    let mut lines = content.lines();
    if lines.next().map(str::trim) != Some("---") {
        return content;
    }

    let mut offset = 4;
    for line in lines {
        offset += line.len() + 1;
        if line.trim() == "---" {
            return content
                .get(offset..)
                .unwrap_or(content)
                .trim_start_matches('\n');
        }
    }

    content
}

pub fn markdown_to_html(markdown: &str) -> String {
    let mut out = String::new();
    let mut buffered = Vec::new();
    let mut options = MarkdownOptions::empty();
    options.insert(MarkdownOptions::ENABLE_STRIKETHROUGH);
    options.insert(MarkdownOptions::ENABLE_TABLES);
    options.insert(MarkdownOptions::ENABLE_TASKLISTS);
    options.insert(MarkdownOptions::ENABLE_FOOTNOTES);
    options.insert(MarkdownOptions::ENABLE_HEADING_ATTRIBUTES);

    let parser = MarkdownParser::new_ext(markdown, options);
    let mut it = parser.into_iter();
    while let Some(event) = it.next() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                markdown_html::push_html(&mut out, buffered.drain(..));
                let language = match kind {
                    CodeBlockKind::Fenced(lang) => Some(lang.into_string()),
                    CodeBlockKind::Indented => None,
                };
                let code = collect_code_block_text(&mut it);
                out.push_str(&render_code_block(&code, language.as_deref()));
            }
            other => buffered.push(other),
        }
    }
    markdown_html::push_html(&mut out, buffered.drain(..));
    out
}

fn collect_code_block_text<'a, I>(events: &mut I) -> String
where
    I: Iterator<Item = Event<'a>>,
{
    let mut code = String::new();
    for event in events {
        match event {
            Event::End(TagEnd::CodeBlock) => break,
            Event::Text(text) | Event::Code(text) => code.push_str(&text),
            Event::SoftBreak | Event::HardBreak => code.push('\n'),
            _ => {}
        }
    }
    code
}

fn render_code_block(code: &str, language: Option<&str>) -> String {
    let lang = language
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase());
    let class_attr = lang
        .as_deref()
        .map(|l| format!(" class=\"language-{}\"", escape_html_text(l)))
        .unwrap_or_default();
    let highlighted = highlight_code(code, lang.as_deref());
    format!("<pre><code{class_attr}>{highlighted}</code></pre>")
}

fn highlight_code(code: &str, language: Option<&str>) -> String {
    let keywords = keywords_for(language);
    let mut out = String::new();
    for line in code.split_inclusive('\n') {
        let mut rendered = highlight_line(line, keywords, language);
        if rendered.is_empty() {
            rendered = "\n".to_string();
        }
        out.push_str(&rendered);
    }
    if !code.ends_with('\n') {
        out.push_str(&highlight_line("", keywords, language));
    }
    out
}

fn highlight_line(line: &str, keywords: &[&str], language: Option<&str>) -> String {
    let mut out = String::new();
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let mut in_string: Option<u8> = None;

    let line_comment = if matches!(
        language,
        Some("sh" | "bash" | "zsh" | "yaml" | "yml" | "toml")
    ) {
        Some(b'#')
    } else {
        Some(b'/')
    };

    while i < bytes.len() {
        let b = bytes[i];

        if let Some(quote) = in_string {
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == quote && bytes[i.saturating_sub(1)] != b'\\' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            push_span(&mut out, "tok-str", &line[start..i]);
            continue;
        }

        if line_comment == Some(b'#') && b == b'#' {
            push_span(&mut out, "tok-comment", &line[i..]);
            return out;
        }
        if line_comment == Some(b'/') && b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            push_span(&mut out, "tok-comment", &line[i..]);
            return out;
        }

        if b == b'\'' || b == b'"' {
            in_string = Some(b);
            continue;
        }

        if b.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < bytes.len()
                && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b'_')
            {
                i += 1;
            }
            push_span(&mut out, "tok-num", &line[start..i]);
            continue;
        }

        if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &line[start..i];
            if keywords.iter().any(|kw| kw == &word) {
                push_span(&mut out, "tok-kw", word);
            } else {
                out.push_str(&escape_html_text(word));
            }
            continue;
        }

        if b == b'$'
            && i + 1 < bytes.len()
            && (bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_')
        {
            let start = i;
            i += 2;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            push_span(&mut out, "tok-var", &line[start..i]);
            continue;
        }

        if b == b'-' && i + 1 < bytes.len() && bytes[i + 1] == b'-' {
            let start = i;
            i += 2;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
                i += 1;
            }
            push_span(&mut out, "tok-flag", &line[start..i]);
            continue;
        }

        if let Some(ch) = line[i..].chars().next() {
            let len = ch.len_utf8();
            out.push_str(&escape_html_text(&line[i..i + len]));
            i += len;
        } else {
            break;
        }
    }

    out
}

fn keywords_for(language: Option<&str>) -> &'static [&'static str] {
    match language {
        Some("rust" | "rs") => &[
            "fn", "let", "mut", "pub", "impl", "struct", "enum", "match", "if", "else", "for",
            "while", "loop", "return", "use", "mod", "trait", "where", "self", "crate", "super",
            "const",
        ],
        Some("js" | "javascript" | "ts" | "typescript") => &[
            "function", "const", "let", "var", "if", "else", "return", "class", "new", "import",
            "from", "export", "async", "await", "try", "catch", "throw",
        ],
        Some("go") => &[
            "func",
            "var",
            "const",
            "type",
            "struct",
            "interface",
            "if",
            "else",
            "for",
            "range",
            "return",
            "package",
            "import",
            "go",
            "defer",
            "switch",
            "case",
        ],
        Some("sh" | "bash" | "zsh") => &[
            "if", "then", "fi", "for", "in", "do", "done", "case", "esac", "function", "export",
        ],
        Some("yaml" | "yml" | "toml" | "json") => &["true", "false", "null"],
        _ => &["true", "false", "null"],
    }
}

fn push_span(out: &mut String, class: &str, text: &str) {
    out.push_str("<span class=\"");
    out.push_str(class);
    out.push_str("\">");
    out.push_str(&escape_html_text(text));
    out.push_str("</span>");
}

fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
