// Diagnostic only: compare public placed-line geometry with engine PNG/PDF.
use serde_json::json;
use wordcraft_engine::layout::Placed;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let out = std::path::Path::new(&args[2]);
    std::fs::create_dir_all(out)?;
    let mut s = wordcraft_gnome::open_document(std::path::Path::new(&args[1]))?;
    let before = s.doc.clone();
    for (page, p) in s.layout().pages.iter().enumerate() {
        for item in &p.items {
            if let Placed::Lines {
                path,
                para,
                l0,
                l1,
                y,
                ..
            } = item
            {
                println!("page={page} path={path:?} y={y} lines={l0}..{l1}");
                for line in &para.lines[*l0..*l1] {
                    println!("{line:?}");
                }
            }
        }
    }
    s.run(
        "file.exportPng",
        &json!({"path":out.join("page.png"),"page":1,"scale":1.5}),
    )?;
    s.run("file.exportPdf", &json!({"path":out.join("document.pdf")}))?;
    assert_eq!(s.doc, before);
    Ok(())
}
