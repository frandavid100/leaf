//! Explicit downloads for the system font viewer; never a private installed font store.
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use wordcraft_engine::Session;
use wordcraft_fonts::FontDb;

pub const CATALOG_REV: &str = "5bb28733cec313b63fbd415b574b5ae35c8aa8b8";
pub const STYLES: &[&str] = &["Regular", "Bold", "Italic", "BoldItalic"];
const LIMIT: usize = 16 * 1024 * 1024;

pub fn available(name: &str) -> bool {
    FontDb::global().has_family(name)
}
pub fn families() -> Vec<String> {
    FontDb::global().families()
}
pub fn rescan() {
    FontDb::global().load_system_fonts();
}
/// Reload only faces actually present in the system catalog. This also handles a
/// newly installed Bold face when an older Regular face is already loaded.
pub fn refresh_installed(names: &[String]) {
    rescan();
    let mut dirs = wordcraft_fonts::system_font_dirs();
    if let Some(data) = std::env::var_os("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(data).join("fonts"));
    }
    let installed = FontDb::with_font_dirs(dirs);
    for name in names {
        if !installed.has_family(name) {
            continue;
        }
        for style in ["Regular", "Bold", "Italic", "Bold Italic"] {
            let face = installed.face(name, style);
            if face.named(name) {
                // The public bulk loader also invalidates cached font choices.
                FontDb::global().add_document_fonts(vec![(face.data().to_vec(), name.clone())]);
            }
        }
    }
}
#[derive(Clone, Debug)]
pub struct MissingFont {
    pub font: String,
    pub shown_as: Vec<String>,
    pub faces: Vec<&'static str>,
}
pub fn missing(s: &mut Session) -> Vec<MissingFont> {
    let layout = s.layout();
    let mut missing = wordcraft_engine::cmd::font::missing_fonts(&s.doc, &layout)
        .into_iter()
        .map(|f| MissingFont {
            font: f.font,
            shown_as: f.shown_as,
            faces: Vec::new(),
        })
        .collect::<Vec<_>>();
    // The engine's missing-family report already handles embedded faces, actual
    // drawn scripts, fields and list labels. Supplement it for authored B/I faces.
    for story in s.doc.shown_stories() {
        for path in s.doc.para_paths(story) {
            let Some(p) = s.doc.para(story, &path) else {
                continue;
            };
            for (range, props) in p.run_ranges() {
                if range.is_empty() {
                    continue;
                }
                let chr = s.doc.styles.resolve_char(p.props.style.as_deref(), props);
                if !available(&chr.font) {
                    continue;
                }
                let style = match (chr.bold, chr.italic) {
                    (true, true) => "Bold Italic",
                    (true, false) => "Bold",
                    (false, true) => "Italic",
                    _ => "Regular",
                };
                let face = FontDb::global().face(&chr.font, style);
                if face.italic == chr.italic && (face.weight >= 600.) == chr.bold {
                    continue;
                }
                let index = missing
                    .iter()
                    .position(|f| f.font.eq_ignore_ascii_case(&chr.font))
                    .unwrap_or_else(|| {
                        missing.push(MissingFont {
                            font: chr.font.clone(),
                            shown_as: vec![format!("{} {}", face.family, face.style)],
                            faces: Vec::new(),
                        });
                        missing.len() - 1
                    });
                if !missing[index].faces.contains(&style) {
                    missing[index].faces.push(style);
                }
            }
        }
    }
    missing
}
/// A small audited catalog: unlisted/proprietary fonts always use the local-file route.
pub fn catalog_file(family: &str, style: &str) -> Option<String> {
    (family.eq_ignore_ascii_case("Poppins") && STYLES.contains(&style))
        .then(|| format!("Poppins-{style}.ttf"))
}
/// Parse using an isolated public database, without adding anything to the process database.
pub fn validate(bytes: &[u8], expected: Option<&str>) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > LIMIT {
        return Err(wordcraft_gnome::t!("The font file is empty or too large.").into());
    }
    if !bytes.starts_with(b"\0\x01\0\0") && !bytes.starts_with(b"OTTO") {
        return Err(wordcraft_gnome::t!("Choose a TrueType or OpenType font file.").into());
    }
    let probe = FontDb::with_font_dirs(Vec::new());
    let families = match probe.install_font("font.ttf", bytes) {
        Ok(added) => added.font.families(),
        Err(wordcraft_fonts::UserFontError::AlreadyAvailable(families)) => families,
        Err(_) => return Err(wordcraft_gnome::t!("This is not a usable font file.").into()),
    };
    if expected.is_some_and(|name| !families.iter().any(|f| f.eq_ignore_ascii_case(name))) {
        return Err(wordcraft_gnome::t!(
            "The downloaded file does not contain the requested font."
        )
        .into());
    }
    Ok(())
}
pub fn read_local(path: &Path) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take((LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    validate(&bytes, None)?;
    Ok(bytes)
}
fn fetch(file: &str, cap: usize, cancel: &AtomicBool) -> Result<Vec<u8>, String> {
    if cancel.load(Ordering::Acquire) {
        return Err(wordcraft_gnome::t!("Cancelled").into());
    }
    let mut config = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(45)))
        .timeout_per_read(Some(std::time::Duration::from_secs(8)))
        .max_redirects(0);
    // Respect an explicitly selected trust bundle (for example a managed
    // desktop's proxy CA). Verification and host-name checks stay enabled.
    if let Some(path) = std::env::var_os("SSL_CERT_FILE") {
        let mut pem = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(4 * 1024 * 1024)
            .read_to_end(&mut pem)
            .map_err(|e| e.to_string())?;
        let roots = ureq::tls::parse_pem(&pem)
            .filter_map(|p| match p {
                Ok(ureq::tls::PemItem::Certificate(cert)) => Some(cert),
                _ => None,
            })
            .collect::<Vec<_>>();
        if roots.is_empty() {
            return Err(wordcraft_gnome::t!(
                "The certificate trust file contains no certificates."
            )
            .into());
        }
        config = config.tls_config(
            ureq::tls::TlsConfig::builder()
                .root_certs(roots.into())
                .build(),
        );
    }
    if let Some(proxy) = std::env::var_os("HTTPS_PROXY").or_else(|| std::env::var_os("https_proxy"))
    {
        config = config.proxy(Some(
            ureq::Proxy::new(&proxy.to_string_lossy()).map_err(|e| e.to_string())?,
        ));
    }
    let agent = ureq::Agent::new_with_config(config.build());
    let url =
        format!("https://raw.githubusercontent.com/google/fonts/{CATALOG_REV}/ofl/poppins/{file}");
    let mut response = agent.get(&url).call().map_err(|e| e.to_string())?;
    let mut reader = response.body_mut().as_reader();
    let mut out = Vec::new();
    let mut buf = [0; 16384];
    loop {
        if cancel.load(Ordering::Acquire) {
            return Err(wordcraft_gnome::t!("Cancelled").into());
        }
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        if out.len() + n > cap {
            return Err(wordcraft_gnome::t!("The font download is too large.").into());
        }
        out.extend_from_slice(&buf[..n]);
    }
    Ok(out)
}
pub fn download(
    family: &str,
    style: &str,
    folder: &Path,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    let file = catalog_file(family, style).ok_or(wordcraft_gnome::t!(
        "This font is not in the download catalog. Choose a local font file."
    ))?;
    let bytes = fetch(&file, LIMIT, cancel)?;
    validate(&bytes, Some(family))?;
    let license = fetch("OFL.txt", 32768, cancel)?;
    if !license.starts_with(b"Copyright")
        || !String::from_utf8_lossy(&license).contains("SIL OPEN FONT LICENSE")
    {
        return Err(wordcraft_gnome::t!("The font license could not be verified.").into());
    }
    if cancel.load(Ordering::Acquire) {
        return Err(wordcraft_gnome::t!("Cancelled").into());
    }
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    std::fs::write(folder.join("OFL.txt"), license).map_err(|e| e.to_string())?;
    let path = folder.join(file);
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}
