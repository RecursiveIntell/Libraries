use crate::types::ScopeTag;

pub(crate) fn infer_scope(context: &[String]) -> ScopeTag {
    let sanitized = strip_comments_and_strings(context);
    for line in sanitized.lines().rev() {
        let tokens = tokenize(line);
        if tokens.is_empty() {
            continue;
        }

        if tokens.contains(&"macro_rules") {
            return ScopeTag::Macro;
        }
        if tokens.contains(&"fn") {
            return ScopeTag::Function;
        }
        if tokens.contains(&"impl") {
            return ScopeTag::Implementation;
        }
        if tokens.contains(&"struct") {
            return ScopeTag::Struct;
        }
        if tokens.contains(&"enum") {
            return ScopeTag::Enum;
        }
        if tokens.contains(&"trait") {
            return ScopeTag::Trait;
        }
        if tokens.contains(&"mod") {
            return ScopeTag::Module;
        }
    }

    ScopeTag::Unknown
}

fn tokenize(line: &str) -> Vec<&str> {
    line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|token| !token.is_empty())
        .collect()
}

fn strip_comments_and_strings(context: &[String]) -> String {
    let mut output = String::new();
    let mut in_block_comment = 0_u32;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    for line in context {
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if in_block_comment > 0 {
                if ch == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    in_block_comment -= 1;
                } else if ch == '/' && chars.peek() == Some(&'*') {
                    chars.next();
                    in_block_comment += 1;
                }
                continue;
            }

            if in_string {
                if escaped {
                    escaped = false;
                    continue;
                }
                match ch {
                    '\\' => escaped = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }

            if in_char {
                if escaped {
                    escaped = false;
                    continue;
                }
                match ch {
                    '\\' => escaped = true,
                    '\'' => in_char = false,
                    _ => {}
                }
                continue;
            }

            if ch == '/' && chars.peek() == Some(&'/') {
                break;
            }
            if ch == '/' && chars.peek() == Some(&'*') {
                chars.next();
                in_block_comment += 1;
                continue;
            }
            if ch == '"' {
                in_string = true;
                continue;
            }
            if ch == '\'' {
                in_char = true;
                continue;
            }

            output.push(ch);
        }
        output.push('\n');
    }

    output
}
