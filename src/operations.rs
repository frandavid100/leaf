//! Small public-command transactions shared by native document tools.
use wordcraft_engine::Session;

/// Keep WordCraft's document/history authoritative and restore both on failure.
pub fn atomic<T>(
    s: &mut Session,
    label: &str,
    edit: impl FnOnce(&mut Session) -> Result<T, String>,
) -> Result<T, String> {
    let snapshot = s.edit_snapshot();
    let mark = s.undo_mark();
    s.checkpoint(label);
    match edit(s) {
        Ok(value) => {
            s.merge_undo_since(mark);
            Ok(value)
        }
        Err(error) => {
            s.restore(snapshot);
            Err(error)
        }
    }
}
