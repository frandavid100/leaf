//! A private, disposable PDF snapshot. Printing never saves the live session.
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use wordcraft_doc::Document;
use wordcraft_engine::Session;

static NEXT: AtomicU64 = AtomicU64::new(0);

pub struct PrintFile {
    directory: PathBuf,
}
impl PrintFile {
    pub fn prepare(document: Document) -> Result<Self, String> {
        Self::prepare_with_path(document, None)
    }
    pub fn prepare_with_path(
        document: Document,
        document_path: Option<PathBuf>,
    ) -> Result<Self, String> {
        let tick = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "leaf-print-{}-{tick}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&directory).map_err(|e| {
            wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Could Not Prepare Printing: {e}"),
                &[("e", format!("{}", e))],
            )
        })?;
        let file = Self { directory };
        // WordCraft 0.14 separates printer jobs from exported PDFs: links stay
        // clickable in exports, but their annotations can break printer fitting.
        let mut session = Session::new(document);
        session.path = document_path;
        crate::references::refresh_derived(&mut session)?;
        let (bytes, _) = session
            .print_job_pdf(&serde_json::json!({"markup": false}))
            .map_err(|e| e.to_string())?;
        std::fs::write(file.path(), bytes).map_err(|e| e.to_string())?;
        Ok(file)
    }
    pub fn path(&self) -> PathBuf {
        self.directory.join("document.pdf")
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}
impl Drop for PrintFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
