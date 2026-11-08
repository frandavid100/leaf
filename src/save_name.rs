//! Read-only, bounded filename suggestions for the native save chooser.
use std::path::Path;
use wordcraft_engine::Session;

/// Keep complete UTF-8 characters and leave room for extensions on common filesystems.
fn stem(text: &str) -> Option<String> {
    let mut result = String::new();
    let mut space = false;
    for c in text.chars() {
        if c.is_whitespace() || c.is_control() || "/\\:*?\"<>|".contains(c) {
            space = !result.is_empty();
            continue;
        }
        if result.len() + c.len_utf8() + usize::from(space) > 180 {
            break;
        }
        if space {
            result.push(' ');
            space = false;
        }
        result.push(c);
    }
    let result = result.trim_matches([' ', '.']);
    (!result.is_empty()).then(|| result.to_owned())
}

/// Existing files use a copy suffix; unnamed documents use metadata, then a heading.
/// This never changes document metadata, selection or history.
pub fn suggestion(session: &Session, source: Option<&Path>, requires_docx: bool) -> String {
    let title = stem(&session.doc.core.title);
    let name = source
        .and_then(Path::file_stem)
        .and_then(|s| s.to_str())
        .and_then(stem)
        .or(title)
        .or_else(|| {
            crate::navigation::headings(&session.doc)
                .iter()
                .find_map(|h| stem(&h.text))
        })
        .unwrap_or_else(|| crate::t!("Document").into());
    let ext = if !requires_docx
        && source
            .and_then(Path::extension)
            .is_some_and(|e| e.eq_ignore_ascii_case("odt"))
    {
        "odt"
    } else {
        "docx"
    };
    let suffix = if source.is_some() {
        format!(" — {}", crate::t!("Copy"))
    } else {
        String::new()
    };
    format!("{name}{suffix}.{ext}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_is_safe_and_read_only() {
        let mut s = crate::sample_session();
        s.doc.core.title = " . / Plan:\n日本語\\2026? . ".into();
        let before = (
            s.rev(),
            s.undo_depth(),
            s.sel.clone(),
            s.doc.core.title.clone(),
        );
        assert_eq!(suggestion(&s, None, false), "Plan 日本語 2026.docx");
        assert_eq!(
            before,
            (
                s.rev(),
                s.undo_depth(),
                s.sel.clone(),
                s.doc.core.title.clone()
            )
        );
    }

    #[test]
    fn first_heading_is_used_when_title_is_empty() {
        let mut s = Session::new(wordcraft_doc::Document::from_text("Lesson plan\nBody"));
        s.run("para.style", &serde_json::json!({"style":"Heading1"}))
            .unwrap();
        let before = (s.rev(), s.undo_depth(), s.sel.clone());
        assert_eq!(suggestion(&s, None, false), "Lesson plan.docx");
        s.doc.core.title = "Title takes precedence".into();
        assert_eq!(suggestion(&s, None, false), "Title takes precedence.docx");
        assert_eq!(before, (s.rev(), s.undo_depth(), s.sel.clone()));
    }

    #[test]
    fn bounded_unicode_and_copy_format() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        s.doc.core.title = "界".repeat(300);
        let name = suggestion(&s, None, false);
        assert!(name.len() <= 185);
        assert_eq!(name.chars().filter(|c| *c == '界').count(), 60);
        assert!(suggestion(&s, Some(Path::new("/tmp/Original.odt")), false).ends_with(".odt"));
        assert!(suggestion(&s, Some(Path::new("/tmp/Original.odt")), true).ends_with(".docx"));
        s.doc.core.title = "../\\\n..".into();
        assert_eq!(
            suggestion(&s, None, false),
            format!("{}.docx", crate::t!("Document"))
        );
    }
}
