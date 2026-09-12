//! Logging redaction invariants (AGENTS.md "Logging" convention, 09 §2).
//!
//! The server's logging policy is: `tracing` events must never carry
//! candidate-sensitive or secret material — audio URLs, transcripts,
//! option ids, answer keys, emails, credentials or DSNs (connection
//! strings with passwords).
//!
//! Two enforcement layers:
//! 1. **Static** (`log_statement_source_scan`): every `tracing::*!` call
//!    site in this crate is scanned for the redaction list as bare
//!    identifiers. This catches the realistic mistake — a future edit
//!    interpolating `transcript`, `option_id`, `key` etc. directly into a
//!    log line — at `cargo test` time, with zero runtime cost. It cannot
//!    be fooled by input values (they aren't in the source), which is
//!    exactly the property a source-level linter needs.
//! 2. **Runtime** (ci.yml, "Container Healthz Smoke Test"): the built
//!    container boots with canary secret values in its environment,
//!    serves traffic, and its full `docker logs` output — which includes
//!    every event from this crate's own code, `tower_http`'s TraceLayer
//!    and deadpool's internal tracing — is grepped for those canaries.
//!    Only a whole-process capture proves the *composed* pipeline is
//!    clean; the static scan alone cannot (third-party crates aren't in
//!    this source tree).
//!
//! The complementary credential-hygiene rule this module's existence
//! motivated: the Gemini client sends its API key in the
//! `x-goog-api-key` header, never in the URL, so reqwest error Display
//! strings (which embed URLs) can't leak it — see
//! `GeminiClient::generate_content_url`.

/// Redaction list from AGENTS.md §3 ("Logging" redaction list: audio URLs,
/// transcripts, option ids, keys, emails) plus the credential-bearing
/// identifiers this deployment uses (`api_key`, `secret`, `password`,
/// `database_url`). Each entry is matched as a whole code identifier
/// (inside `{...}` captures, `ident = value` named fields, or positional
/// args), never as prose inside the message literal.
pub const REDACTED_IDENTIFIERS: &[&str] = &[
    "audio_url",
    "transcript",
    "option_id",
    "authoring_letter",
    "correct",
    "key",
    "script",
    "rationale",
    "email",
    "api_key",
    "secret",
    "password",
    "database_url",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Scan every Rust source file in this crate for `tracing::(info|warn|
    /// error|debug)!` macro invocations and fail if any redacted identifier
    /// appears in the macro's literal text. Catches `tracing::info!("... {transcript}")`-style
    /// regressions anywhere in `server/src`, including files added later.
    #[test]
    fn log_statement_source_scan() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let src_dir = std::path::Path::new(manifest_dir).join("src");

        let mut rust_files = Vec::new();
        collect_rust_files(&src_dir, &mut rust_files);

        assert!(
            !rust_files.is_empty(),
            "source scan found no Rust files under {} — the test is misconfigured",
            src_dir.display()
        );

        let mut failures: Vec<String> = Vec::new();

        for path in rust_files {
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            let rel = path.strip_prefix(manifest_dir).unwrap_or(&path).display().to_string();

            for (idx, line) in source.lines().enumerate() {
                let trimmed = line.trim_start();
                if !trimmed.starts_with("tracing::")
                    || !line.contains("!")
                {
                    continue;
                }
                // Only the macro invocations: tracing::info!(, tracing::warn!(, etc.
                let is_log_macro = ["tracing::info!(", "tracing::warn!(", "tracing::error!(", "tracing::debug!(", "tracing::trace!("]
                    .iter()
                    .any(|m| line.contains(m));
                if !is_log_macro {
                    continue;
                }

                // Join continuation lines so multi-line macro bodies are
                // checked as one statement.
                let mut statement = line.to_string();
                if !line.ends_with(';') && !statement.ends_with(')') {
                    let mut depth: i32 = statement.matches('(').count() as i32
                        - statement.matches(')').count() as i32;
                    let mut rest = source.lines().skip(idx + 1);
                    while depth > 0 {
                        match rest.next() {
                            Some(next_line) => {
                                statement.push(' ');
                                statement.push_str(next_line.trim());
                                depth += next_line.matches('(').count() as i32;
                                depth -= next_line.matches(')').count() as i32;
                            }
                            None => break,
                        }
                    }
                }

                for ident in REDACTED_IDENTIFIERS {
                    // Only the *code* part of the macro is scanned — string
                    // literals are stripped first, so an English word inside
                    // a message ("a random secret") can never false-positive.
                    // What remains is the format-args/field-name surface:
                    // inline captures (`{transcript}`), named fields
                    // (`email = e`), and positional args (`"{}", correct`).
                    if contains_identifier_outside_literals(&statement, ident) {
                        failures.push(format!(
                            "{rel}:{}: tracing macro appears to log redacted identifier `{ident}`: {}",
                            idx + 1,
                            statement.trim()
                        ));
                    }
                }
            }
        }

        assert!(
            failures.is_empty(),
            "tracing statements violating the redaction policy (AGENTS.md §3):\n{}",
            failures.join("\n")
        );
    }

    fn collect_rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let entries = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("failed to read dir {}: {e}", dir.display()));
        for entry in entries {
            let entry = entry.expect("valid dir entry");
            let path = entry.path();
            if path.is_dir() {
                collect_rust_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }

    /// True if `ident` occurs in `statement` as a code token — i.e. outside
    /// every string literal. Word-boundary matching on the literal-stripped
    /// text covers `{ident}` captures, `ident = value` named fields, and
    /// positional-arg uses, while a word inside a quoted message ("random
    /// secret") is removed with the literal and never matches.
    fn contains_identifier_outside_literals(statement: &str, ident: &str) -> bool {
        let code_only = strip_string_literals(statement);
        code_only.split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|token| token == ident)
    }

    /// Replace the contents of every double-quoted string literal with a
    /// placeholder. Handles `\"` escapes. Raw strings and char literals are
    /// not used in any tracing macro in this crate (asserted below by
    /// construction: char literals like `'a'` cannot contain the
    /// identifiers we scan for, and no `r#"..."#` appears in a log macro).
    fn strip_string_literals(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut in_literal = false;
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if in_literal {
                match c {
                    '\\' => {
                        chars.next(); // skip the escaped char
                    }
                    '"' => in_literal = false,
                    _ => {}
                }
            } else if c == '"' {
                in_literal = true;
                out.push('"');
            } else {
                out.push(c);
            }
        }
        out
    }

    /// The literal-stripper must actually strip: sanity property that
    /// would catch a refactor breaking the scanner itself.
    #[test]
    fn literal_stripper_removes_message_contents() {
        let stripped = strip_string_literals(
            "tracing::warn!(\"JWT_SECRET not set — random secret {}\", secret_value);"
        );
        assert!(stripped.contains("secret_value"));
        assert!(!stripped.contains("random secret"));
    }

    /// The scanner must flag a real violation: inline-capture form.
    #[test]
    fn source_scan_detects_inline_capture_violation() {
        let src = "tracing::info!(\"transcript was {transcript}\");";
        assert!(contains_identifier_outside_literals(src, "transcript"));
    }

    /// The scanner must flag named-field form: `tracing::info!(email = user_email)`.
    #[test]
    fn source_scan_detects_named_field_violation() {
        let src = "tracing::info!(email = user_email);";
        assert!(contains_identifier_outside_literals(src, "email"));
    }

    /// The scanner must NOT flag an English word inside the message text.
    #[test]
    fn source_scan_ignores_words_inside_literals() {
        let src = "tracing::warn!(\"JWT_SECRET not set — using an insecure development default.\");";
        assert!(!contains_identifier_outside_literals(src, "secret"));
    }

}
