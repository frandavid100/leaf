//! WordCraft proofing and language adapters; no second spell-checking engine.
use serde_json::{Value, json};
use wordcraft_doc::{Pos, StoryRef};
use wordcraft_engine::Session;

#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub start: Pos,
    pub end: Pos,
    pub text: String,
    pub message: String,
    pub spelling: bool,
}

/// Explain built-in findings in the interface language. Keep unknown engine
/// messages intact rather than guessing a translation of document content.
pub fn description(issue: &Issue) -> String {
    if issue.spelling {
        return wordcraft_gnome::t!("Possible spelling mistake").into();
    }
    match issue.message.as_str() {
        "Use \"an\" before a vowel sound" => {
            wordcraft_gnome::t!("Use \"an\" before a vowel sound").into()
        }
        "Use \"a\" before a consonant sound" => {
            wordcraft_gnome::t!("Use \"a\" before a consonant sound").into()
        }
        "Remove the space before the punctuation" => {
            wordcraft_gnome::t!("Remove the space before the punctuation").into()
        }
        "Extra space" => wordcraft_gnome::t!("Extra space").into(),
        "Capitalize the first word of a sentence" => {
            wordcraft_gnome::t!("Capitalize the first word of a sentence").into()
        }
        text => {
            if let Some(word) = text
                .strip_prefix("Repeated word: \"")
                .and_then(|s| s.strip_suffix('"'))
            {
                crate::i18n::interpolate(
                    wordcraft_gnome::t!("Repeated word: {word}"),
                    &[("word", word.to_owned())],
                )
            } else {
                text.into()
            }
        }
    }
}

/// Findings for one canonical paragraph, also used to revalidate retained UI entries.
pub fn paragraph_issues(s: &Session, pos: &Pos) -> Vec<Issue> {
    let Some(p) = s.doc.para_at(pos) else {
        return Vec::new();
    };
    let runs = wordcraft_engine::layout::para::proof_runs(&s.doc, p, None);
    wordcraft_engine::layout::para::paragraph_issues(
        p,
        &runs,
        &s.prefs.spelling(),
        true,
        wordcraft_engine::layout::para::is_list_item(&s.doc, p),
    )
    .into_iter()
    .map(|i| Issue {
        start: Pos {
            off: i.start,
            ..pos.clone()
        },
        end: Pos {
            off: i.end,
            ..pos.clone()
        },
        text: p.text.get(i.start..i.end).unwrap_or("").into(),
        message: i.message,
        spelling: i.kind == wordcraft_engine::proof::IssueKind::Spelling,
    })
    .collect()
}
pub fn issues(s: &Session) -> Vec<Issue> {
    let mut out = Vec::new();
    for story in s.doc.shown_stories() {
        for path in s.doc.para_paths(story) {
            out.extend(paragraph_issues(
                s,
                &Pos {
                    story,
                    path,
                    off: 0,
                },
            ));
            if out.len() >= 500 {
                out.truncate(500);
                return out;
            }
        }
    }
    out
}
/// Remap only unchanged text outside a single changed span. Ambiguous/overlapping
/// findings are discarded; paragraph proofing must still confirm the result.
pub fn remap_issue(
    old: &wordcraft_doc::Document,
    new: &wordcraft_doc::Document,
    i: &Issue,
) -> Option<Issue> {
    let before = &old.para_at(&i.start)?.text;
    let after = &new.para_at(&i.start)?.text;
    if before == after {
        return Some(i.clone());
    }
    let prefix = before
        .chars()
        .zip(after.chars())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum::<usize>();
    let suffix = before[prefix..]
        .chars()
        .rev()
        .zip(after[prefix..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum::<usize>();
    let mut next = i.clone();
    if i.end.off <= prefix {
        return Some(next);
    }
    if i.start.off < before.len() - suffix {
        return None;
    }
    next.start.off = after.len().checked_sub(before.len() - i.start.off)?;
    next.end.off = after.len().checked_sub(before.len() - i.end.off)?;
    Some(next)
}
pub fn at(s: &mut Session, pos: &Pos) -> Option<Issue> {
    let v = s.run("review.suggestions", &json!({"pos":pos})).ok()?;
    Some(Issue {
        start: serde_json::from_value(v.get("start")?.clone()).ok()?,
        end: serde_json::from_value(v.get("end")?.clone()).ok()?,
        text: v["text"].as_str()?.into(),
        message: v["message"].as_str().unwrap_or("").into(),
        spelling: v["kind"] == "spelling",
    })
}
pub fn suggestions(s: &mut Session, i: &Issue) -> Vec<String> {
    s.run("review.suggestions", &json!({"pos":i.start}))
        .ok()
        .and_then(|v| v["suggestions"].as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}
fn valid(s: &Session, i: &Issue) -> bool {
    s.doc
        .para_at(&i.start)
        .and_then(|p| p.text.get(i.start.off..i.end.off))
        .is_some_and(|text| text == i.text)
}
pub fn select(s: &mut Session, i: &Issue) -> Result<(), String> {
    if !valid(s, i) {
        return Err(wordcraft_gnome::t!("The text has changed. Check it again.").into());
    }
    crate::selection::select_range(s, &i.start, &i.end)
}
pub fn replace(s: &mut Session, i: &Issue, text: &str) -> Result<bool, String> {
    select(s, i)?;
    crate::editing::apply(s, crate::editing::Edit::Insert(text.into()))
}
pub fn ignore(s: &mut Session, i: &Issue) -> Result<(), String> {
    if !valid(s, i) {
        return Err(wordcraft_gnome::t!("The text has changed. Check it again.").into());
    }
    s.run("review.ignoreAll", &json!({"word":i.text}))
        .map(|_| ())
        .map_err(|e| e.to_string())
}
pub fn add(s: &mut Session, i: &Issue) -> Result<(), String> {
    if !valid(s, i) {
        return Err(wordcraft_gnome::t!("The text has changed. Check it again.").into());
    }
    s.run("review.addToDictionary", &json!({"word":i.text}))
        .map(|_| ())
        .map_err(|e| e.to_string())
}
/// Apply to the selection/typing position or every visible story in one undo step.
pub fn language(s: &mut Session, tag: &str, no_proof: bool, whole: bool) -> Result<bool, String> {
    let tag = wordcraft_engine::proof::lang::normalize_tag(tag).ok_or(wordcraft_gnome::t!(
        "Use a language tag such as es-ES or en-GB."
    ))?;
    let doc = s.doc.clone();
    let snapshot = s.edit_snapshot();
    let selection = s.sel.clone();
    let pending = s.pending.clone();
    let typing = s.typing_props();
    let default = s.prefs.default_language.clone();
    let mark = s.undo_mark();
    let back = s.hf_return.clone();
    let hint = s.page_hint;
    let result = (|| {
        if whole {
            s.checkpoint(wordcraft_gnome::t!("Text Language"));
            for story in s.doc.shown_stories() {
                let a = s.doc.start_of(story);
                let b = s.doc.end_of(story);
                crate::selection::select_range(s, &a, &b)?;
                s.join_next_undo();
                s.run(
                    "review.language",
                    &json!({"lang":tag,"noProof":no_proof,"default":story==StoryRef::Body}),
                )
                .map_err(|e| e.to_string())?;
            }
            crate::selection::select_range(s, &selection.anchor, &selection.focus)?;
            let mut next = typing.clone();
            next.lang = Some(tag.clone());
            next.no_proof = Some(no_proof);
            s.pending = Some(next);
            s.merge_undo_since(mark);
        } else {
            s.run("review.language", &json!({"lang":tag,"noProof":no_proof}))
                .map_err(|e| e.to_string())?;
        }
        Ok::<_, String>(s.doc != doc)
    })();
    s.prefs.default_language = default;
    s.hf_return = back;
    s.page_hint = hint;
    match result {
        Ok(changed) => Ok(changed),
        Err(e) => {
            s.restore(snapshot);
            s.pending = pending;
            Err(e)
        }
    }
}
pub fn state(s: &mut Session) -> Value {
    s.run("format.state", &json!({})).unwrap_or(Value::Null)
}

fn dictionary_directory() -> std::path::PathBuf {
    crate::dictionaries::imports()
}

pub fn install_dictionary(files: &[std::path::PathBuf]) -> Result<(), String> {
    let invalid = || {
        wordcraft_gnome::t!("Select the matching .aff and .dic files for one Hunspell dictionary.")
            .to_string()
    };
    let [first, second] = files else {
        return Err(invalid());
    };
    let (aff, dic) = if first.extension().is_some_and(|ext| ext == "aff") {
        (first, second)
    } else {
        (second, first)
    };
    let stem = aff
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(invalid)?;
    if !aff.extension().is_some_and(|ext| ext == "aff")
        || !dic.extension().is_some_and(|ext| ext == "dic")
        || dic.file_stem() != aff.file_stem()
        || !stem
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(invalid());
    }
    wordcraft_engine::proof::hunspell::load(aff, dic)?;
    let dir = dictionary_directory();
    let io_error = |error: std::io::Error| error.to_string();
    std::fs::create_dir_all(&dir).map_err(io_error)?;
    let aff_target = dir.join(format!("{stem}.aff"));
    let dic_target = dir.join(format!("{stem}.dic"));
    if aff_target.exists() || dic_target.exists() {
        return Err(
            wordcraft_gnome::t!("A dictionary with this name is already installed.").into(),
        );
    }
    std::fs::copy(aff, &aff_target).map_err(io_error)?;
    if let Err(error) = std::fs::copy(dic, &dic_target) {
        let _ = std::fs::remove_file(aff_target);
        let _ = std::fs::remove_file(dic_target);
        return Err(error.to_string());
    }
    let loaded = crate::dictionaries::refresh(true).and_then(|_| {
        let tag = stem.replace('_', "-");
        let recognised = crate::dictionaries::dictionary_path(&tag)
            .and_then(|p| p.canonicalize().ok())
            .zip(dic_target.canonicalize().ok())
            .is_some_and(|(actual, expected)| actual == expected);
        if recognised {
            Ok(())
        } else {
            Err(wordcraft_gnome::t!(
                "WordCraft does not use a dictionary with this language or filename."
            )
            .into())
        }
    });
    if let Err(error) = loaded {
        let _ = std::fs::remove_file(&aff_target);
        let _ = std::fs::remove_file(&dic_target);
        let _ = crate::dictionaries::refresh(true);
        return Err(error);
    }
    Ok(())
}

pub fn initialize() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if crate::dictionaries::refresh(false).is_err() {
            wordcraft_engine::proof::set_dictionaries_folder(Some(crate::dictionaries::bundled()));
        }
        wordcraft_engine::proof::load_user_dictionary(
            &crate::preferences::config_root().join("custom-dictionary.txt"),
        );
    });
}
pub fn configure(s: &mut Session) {
    initialize();
    let name = crate::preferences::user_name();
    if !name.is_empty() {
        let _ = s.run("file.setAuthor", &json!({"name":name}));
    }
    let dir = crate::preferences::config_root();
    if std::fs::create_dir_all(&dir).is_ok() {
        s.user_dict_path = Some(dir.join("custom-dictionary.txt"));
    }
    crate::proofing_defaults::register(s);
    s.prefs.default_language = Some(crate::proofing_defaults::language(s));
    s.view.proofing = true;
    s.prefs.check_spelling = true;
}
