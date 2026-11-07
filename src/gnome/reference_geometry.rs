//! Disposable TOC ranges, indexed once per canonical document revision.
use super::*;

#[derive(Default)]
pub(super) struct TocCache {
    identity: Option<(u64, u64)>,
    spans: Vec<(Pos, Pos)>,
}

impl TocCache {
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }

    pub(super) fn at(&mut self, session: &Session, position: &Pos) -> Option<(Pos, Pos)> {
        if position.story != wordcraft_doc::StoryRef::Body {
            return None;
        }
        let identity = (session.document_id(), session.rev());
        if self.identity != Some(identity) {
            perf_count("toc_lookup_scans", 1);
            self.spans = wordcraft_gnome::references::toc_spans(session);
            self.identity = Some(identity);
        }
        self.spans
            .iter()
            .find(|(start, end)| start <= position && position <= end)
            .cloned()
    }
}

pub(super) fn cached_toc_span(ui: &Ui, session: &Session, position: &Pos) -> Option<(Pos, Pos)> {
    ui.toc_cache.borrow_mut().at(session, position)
}
