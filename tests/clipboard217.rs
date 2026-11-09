use serde_json::json;
use wordcraft_doc::{Document, StoryRef, props::Rgb};
use wordcraft_engine::Session;
use wordcraft_gnome::clipboard_html::for_paste;

const CAPTURES: &[(&str, &str)] = &[
    (
        "chrome light",
        include_str!("fixtures/clipboard217/chrome-light-release.html"),
    ),
    (
        "chrome dark",
        include_str!("fixtures/clipboard217/chrome-dark-release.html"),
    ),
    (
        "firefox light",
        include_str!("fixtures/clipboard217/firefox-light-release.html"),
    ),
    (
        "firefox dark",
        include_str!("fixtures/clipboard217/firefox-dark-release.html"),
    ),
];
fn rgb(c: wordcraft_doc::props::TextColor) -> Rgb {
    match c {
        wordcraft_doc::props::TextColor::Auto => Rgb::BLACK,
        wordcraft_doc::props::TextColor::Rgb(c) => c,
    }
}
fn pasted(html: &str) -> Session {
    let mut s = Session::new(Document::new());
    s.run("edit.paste", &json!({"html":for_paste(html)}))
        .unwrap();
    s
}
fn properties(s: &Session, text: &str) -> wordcraft_doc::resolve::ResolvedChar {
    for path in s.doc.para_paths(StoryRef::Body) {
        let p = s.doc.para(StoryRef::Body, &path).unwrap();
        if let Some(off) = p.text.find(text) {
            return s
                .doc
                .styles
                .resolve_char(p.props.style.as_deref(), p.props_of_char(off));
        }
    }
    panic!("missing text {text}")
}
#[test]
fn real_browser_release_copies_adapt_without_losing_structure_or_history() {
    for (name, html) in CAPTURES {
        let mut s = pasted(html);
        let doc = s.doc.clone();
        let p = properties(&s, "WordCraft remains");
        assert_eq!(rgb(p.color), Rgb::BLACK, "{name}");
        assert!(
            p.shading.is_none(),
            "{name}: theme background must not be authored"
        );
        assert!(
            s.doc.para_paths(StoryRef::Body).iter().any(|path| {
                let p = s.doc.para(StoryRef::Body, path).unwrap();
                p.props.numbering.is_some()
            }),
            "{name}: lists"
        );
        assert!(
            s.doc.para_paths(StoryRef::Body).iter().any(|path| {
                let p = s.doc.para(StoryRef::Body, path).unwrap();
                p.run_ranges().any(|(_, c)| c.link.is_some())
            }),
            "{name}: links"
        );
        assert_eq!(s.undo_depth(), 1);
        s.run("edit.undo", &json!({})).unwrap();
        assert!(s.doc.plain_text(StoryRef::Body).trim().is_empty());
        s.run("edit.redo", &json!({})).unwrap();
        assert_eq!(s.doc, doc);
        for ext in ["docx", "odt"] {
            let bytes = wordcraft_engine::io::save_bytes(&format!("copy.{ext}"), &s.doc).unwrap();
            let reopened =
                wordcraft_engine::io::open_bytes(&format!("copy.{ext}"), &bytes).unwrap();
            let r = Session::new(reopened);
            assert_eq!(
                r.doc.plain_text(StoryRef::Body),
                s.doc.plain_text(StoryRef::Body)
            );
            assert_eq!(rgb(properties(&r, "WordCraft remains").color), Rgb::BLACK);
        }
        // This is the same public screen renderer used by the native canvas.
        for dark in [false, true] {
            let before = s.doc.clone();
            let r = wordcraft_gnome::render_page_screen(&mut s, 0, 1.0, dark).unwrap();
            assert!(!r.rgba.is_empty());
            assert_eq!(
                s.doc, before,
                "screen rendering must not rewrite document colors"
            );
        }
    }
}
#[test]
fn authored_red_highlights_bold_italic_and_links_survive_both_browsers() {
    for html in [
        include_str!("fixtures/clipboard217/chrome-light-authored.html"),
        include_str!("fixtures/clipboard217/chrome-dark-authored.html"),
        include_str!("fixtures/clipboard217/firefox-light-authored.html"),
        include_str!("fixtures/clipboard217/firefox-dark-authored.html"),
    ] {
        let s = pasted(html);
        assert_eq!(rgb(properties(&s, "Ordinary").color), Rgb::BLACK);
        assert_eq!(rgb(properties(&s, "Intentional red").color), Rgb(255, 0, 0));
        assert_eq!(
            properties(&s, "Intentional highlight").shading,
            Some(Rgb(255, 255, 0))
        );
        assert!(properties(&s, "bold").bold);
        assert!(properties(&s, "italic").italic);
        assert_eq!(
            properties(&s, "A link").link.as_deref(),
            Some("https://example.org/")
        );
    }
}
#[test]
fn preserves_ambiguous_non_chrome_html_and_colored_local_backgrounds() {
    let authored = "<p style=\"color:rgb(240, 246, 252);background-color:#0d1117\">Intentional</p>";
    assert_eq!(for_paste(authored), authored);
    let html = "<p style=\"color:rgb(240, 246, 252);background-color:#0d1117;-webkit-text-stroke-width:0;white-space:normal\">Base <span style=\"background-color:rgba(255,0,0,0.5);color:rgb(240,246,252)\">Local</span></p>";
    let result = for_paste(html);
    assert!(result.contains("rgba(255,0,0,0.5)"));
    assert!(result.contains("color:rgb(240,246,252)"));
    assert!(!result.contains("background-color:#0d1117"));
    assert_eq!(for_paste(&result), result, "normalization is idempotent");
}
#[test]
fn rgb_background_spelling_is_preserved_semantically_for_the_public_importer() {
    let s =
        pasted("<p><span style=\"background-color:rgb(255, 255, 0);color:red\">Marker</span></p>");
    assert_eq!(properties(&s, "Marker").shading, Some(Rgb(255, 255, 0)));
    assert_eq!(rgb(properties(&s, "Marker").color), Rgb(255, 0, 0));
}
