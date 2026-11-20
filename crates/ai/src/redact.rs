const MARK: &str = "[redacted]";

/// Replace saved secrets and key-shaped tokens before a line is logged or shown.
pub fn redact(text: &str, secrets: &[&str]) -> String {
    let mut cleaned = text.to_string();
    for secret in secrets {
        if secret.len() >= 8 {
            cleaned = cleaned.replace(secret, MARK);
        }
    }
    scrub_keys(&cleaned)
}

fn scrub_keys(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(length) = key_len(rest) {
            out.push_str(MARK);
            rest = &rest[length..];
            continue;
        }
        let ch = rest.chars().next().expect("rest is not empty");
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

fn key_len(text: &str) -> Option<usize> {
    let (prefix, tail) = if let Some(tail) = text.strip_prefix("sk-ant-") {
        ("sk-ant-", tail)
    } else {
        ("sk-", text.strip_prefix("sk-")?)
    };
    let extra = tail
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
        .count();
    if extra < 4 {
        return None;
    }
    let byte_len: usize = tail.chars().take(extra).map(char::len_utf8).sum();
    Some(prefix.len() + byte_len)
}

#[cfg(test)]
mod tests {
    use super::redact;

    #[test]
    fn log_line_loses_the_key() {
        let key = "sk-ant-api03-example";
        let line = format!("provider error body includes {key} and sk-live-abcdef");
        let logged = redact(&line, &[key]);
        assert!(!logged.contains(key));
        assert!(!logged.contains("sk-live-abcdef"));
        assert!(logged.contains("[redacted]"));
    }

    #[test]
    fn exact_secret_is_removed_even_without_a_key_prefix() {
        let secret = "super-secret-value";
        let logged = redact(&format!("saved {secret}"), &[secret]);
        assert!(!logged.contains(secret));
        assert!(logged.contains("[redacted]"));
    }
}
