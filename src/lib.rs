//! Integración headless entre WordCraft y nuestra interfaz GNOME.
//! La lógica de apertura, paginación y renderizado no utiliza GTK ni egui.

extern crate self as wordcraft_gnome;
pub mod i18n;

use std::path::Path;
use wordcraft_doc::Document;
use wordcraft_engine::Session;

/// La vista contiene pixels RGBA en vez de widgets egui.
pub struct PageImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub total_pages: usize,
}

/// Documento de demostración de WordCraft: incluye texto, títulos y tabla.
pub fn sample_session() -> Session {
    Session::new(wordcraft_engine::sample::sample_document())
}

/// Utiliza el comando file.open, el mismo que emplea WordCraft.
pub fn open_document(path: &Path) -> Result<Session, String> {
    if !path.is_file() {
        return Err(wordcraft_gnome::t!("The file does not exist or is not a regular file").into());
    }
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !["docx", "odt"]
        .iter()
        .any(|ext| extension.eq_ignore_ascii_case(ext))
    {
        return Err(wordcraft_gnome::t!("Only DOCX and ODT files are supported").into());
    }
    let name = path.to_str().ok_or(wordcraft_gnome::t!(
        "The file path cannot be represented as UTF-8"
    ))?;
    let mut session = Session::new(Document::new());
    session
        .run("file.open", &serde_json::json!({"path": name}))
        .map_err(|e| {
            wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("WordCraft Could Not Open the File: {e}"),
                &[("e", format!("{}", e))],
            )
        })?;
    Ok(session)
}

/// Rasterización fiel de una página, con el propio maquetador de WordCraft.
/// Pixel allocation is bounded below; display density is independent of document zoom.
pub fn render_page(
    session: &mut Session,
    page_index: usize,
    scale: f32,
) -> Result<PageImage, String> {
    render_page_pixels(session, page_index, scale, false, false)
}

/// Native GTK texture data. Retains the renderer's premultiplied RGBA buffer
/// instead of copying and unpremultiplying every pixel on the UI thread.
pub fn render_page_premultiplied(
    session: &mut Session,
    page_index: usize,
    scale: f32,
) -> Result<PageImage, String> {
    render_page_pixels(session, page_index, scale, true, false)
}

/// Screen-only appearance; document data, printing and export are unchanged.
pub fn render_page_screen(
    session: &mut Session,
    page_index: usize,
    scale: f32,
    dark: bool,
) -> Result<PageImage, String> {
    render_page_pixels(session, page_index, scale, true, dark)
}

fn render_page_pixels(
    session: &mut Session,
    page_index: usize,
    scale: f32,
    premultiplied: bool,
    dark: bool,
) -> Result<PageImage, String> {
    let scale = if scale.is_finite() {
        scale.max(0.15)
    } else {
        1.0
    };
    let mut options = wordcraft_render::RenderOptions::default();
    options.display.marks = session.view.marks;
    options.dark = dark;
    let layout = session.layout();
    let count = layout.pages.len();
    let page = layout
        .pages
        .get(page_index)
        .ok_or(wordcraft_gnome::t!("The requested page does not exist"))?;
    // Es una prueba funcional, no un visor para archivos gigantes.
    let width = (page.w * scale).ceil();
    let height = (page.h * scale).ceil();
    if !width.is_finite() || !height.is_finite() || width * height > 12_000_000.0 {
        return Err(wordcraft_gnome::t!("The page exceeds the preview size limit").into());
    }
    let output = wordcraft_render::render_page(&session.doc, page, scale, &options);
    let rgba = if premultiplied {
        output.pixels
    } else {
        output.to_straight()
    };
    if output.width == 0
        || output.height == 0
        || rgba.len() != output.width as usize * output.height as usize * 4
    {
        return Err(wordcraft_gnome::t!("WordCraft returned an incomplete image").into());
    }
    Ok(PageImage {
        width: output.width,
        height: output.height,
        rgba,
        total_pages: count,
    })
}

pub mod viewer_layout;

pub mod selection;

pub mod editing;

pub mod formatting;

pub mod saving;

pub mod preferences;

pub mod tables;

pub mod styles;

pub mod caret_motion;

pub mod search;

pub mod navigation;

pub mod document_properties;
pub mod header_footer;
pub mod page_options;
pub mod pdf_export;
pub mod pictures;
pub mod proofing;

pub mod accessible_text;
pub mod annotations;
pub mod printing;
