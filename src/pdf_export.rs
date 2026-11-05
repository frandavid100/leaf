//! PDF is an export: it never becomes the editable document's save destination.
use std::path::Path;
use wordcraft_engine::Session;
pub fn export(s: &mut Session, path: &Path) -> Result<(), String> {
    if !path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("pdf"))
    {
        return Err("Incluye .pdf en el nombre del archivo".into());
    }
    let path = path
        .to_str()
        .ok_or(wordcraft_gnome::t!("The path is not UTF-8"))?;
    let mut copy = Session::new(s.doc.clone());
    copy.sel = s.sel.clone();
    copy.page_hint = s.page_hint;
    copy.path = s.path.clone();
    copy.view = s.view.clone();
    crate::references::refresh_derived(&mut copy)?;
    copy.run(
        "file.exportPdf",
        &serde_json::json!({"path": path, "markup": false}),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
