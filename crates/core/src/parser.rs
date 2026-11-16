#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub value: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub words: Vec<String>,
    pub prefix: String,
    pub start: usize,
    pub end: usize,
}

/// Parse only literal shell words. Never expand or execute input.
pub fn context(buffer: &str, cursor: usize) -> Option<Context> {
    let chars: Vec<char> = buffer.chars().collect();
    if cursor > chars.len() || chars.len() > 16_384 {
        return None;
    }
    let mut tokens = Vec::new();
    let mut value = String::new();
    let mut start = None;
    let mut quote = None;
    let mut escaped = false;
    let mut segment = 0;
    let mut scanned_end = chars.len();
    for (i, &c) in chars.iter().enumerate() {
        if escaped {
            value.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' && quote != Some('\'') {
            start.get_or_insert(i);
            escaped = true;
            continue;
        }
        if c == '$' && quote != Some('\'') || c == '`' && quote != Some('\'') {
            return None;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else {
                value.push(c);
            }
            continue;
        }
        match c {
            '\'' | '"' => {
                start.get_or_insert(i);
                quote = Some(c);
            }
            '(' | ')' | '{' | '}' | '<' | '>' => return None,
            '#' if start.is_none() => return None,
            ';' | '|' | '&' => {
                if i < cursor {
                    tokens.clear();
                    value.clear();
                    start = None;
                    segment = i + 1;
                } else {
                    scanned_end = i;
                    break;
                }
            }
            c if c.is_whitespace() => {
                if let Some(s) = start.take() {
                    tokens.push(Token {
                        value: std::mem::take(&mut value),
                        start: s,
                        end: i,
                    });
                }
            }
            _ => {
                start.get_or_insert(i);
                value.push(c);
            }
        }
    }
    if escaped {
        return None;
    }
    if let Some(s) = start {
        tokens.push(Token {
            value,
            start: s,
            end: scanned_end,
        });
    }
    let current = tokens.iter().find(|t| t.start <= cursor && cursor <= t.end);
    let (replace_start, replace_end) = current
        .map(|t| (t.start, t.end))
        .unwrap_or((cursor, cursor));
    let raw_prefix: String = chars[replace_start..cursor].iter().collect();
    let prefix = unquote(&raw_prefix);
    let words = tokens
        .iter()
        .filter(|t| t.end < replace_start && t.start >= segment)
        .map(|t| t.value.clone())
        .collect();
    Some(Context {
        words,
        prefix,
        start: replace_start,
        end: replace_end,
    })
}

fn unquote(s: &str) -> String {
    let mut result = String::new();
    let mut quote = None;
    let mut escaped = false;
    for c in s.chars() {
        if escaped {
            result.push(c);
            escaped = false;
        } else if c == '\\' && quote != Some('\'') {
            escaped = true;
        } else if Some(c) == quote {
            quote = None;
        } else if quote.is_none() && matches!(c, '\'' | '"') {
            quote = Some(c);
        } else {
            result.push(c);
        }
    }
    result
}

pub fn shell_quote(value: &str) -> String {
    if !value.is_empty()
        && value.chars().all(|c| {
            c.is_alphanumeric() || matches!(c, '/' | '.' | '_' | '-' | ':' | '@' | '%' | '+' | ',')
        })
    {
        value.into()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_quotes_and_unicode_offsets() {
        let c = context("cat '日本 語", 9).unwrap();
        assert_eq!(c.words, ["cat"]);
        assert_eq!(c.prefix, "日本 語");
        assert_eq!((c.start, c.end), (4, 9));
    }

    #[test]
    fn replaces_whole_token_when_editing_in_the_middle() {
        let c = context("git checkout", 6).unwrap();
        assert_eq!((c.start, c.end), (4, 12));
        assert_eq!(c.prefix, "ch");
        let before_pipe = context("git ch|cat", 6).unwrap();
        assert_eq!((before_pipe.start, before_pipe.end), (4, 6));
    }

    #[test]
    fn parses_last_pipeline_command() {
        let c = context("echo ok | git st", 16).unwrap();
        assert_eq!(c.words, ["git"]);
        assert_eq!(c.prefix, "st");
    }

    #[test]
    fn rejects_expansions_and_redirects() {
        for s in [
            "echo $(touch x)",
            "echo `date`",
            "cat > file",
            "echo ${HOME}",
        ] {
            assert!(context(s, s.chars().count()).is_none(), "{s}");
        }
    }

    #[test]
    fn quotes_shell_metacharacters_as_literal_text() {
        assert_eq!(shell_quote("a'b $(x)"), "'a'\\''b $(x)'");
        assert_eq!(shell_quote("日本語/file"), "日本語/file");
    }
}
