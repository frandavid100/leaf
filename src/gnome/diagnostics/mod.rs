//! Opt-in diagnostics: GUI regression checks, synthetic fixtures and demo
//! setup driven by command-line flags. None of this runs during normal use;
//! every entry point is reached only through a test flag. Leaf ships these
//! checks so CI can run them against the installed Flatpak it publishes.
use super::*;

/// True when Leaf was started with the given command-line flag. Diagnostic
/// checks and demo modes use flags; normal use never passes one. Production
/// code that needs a test hook asks here, so every hook can be found by
/// searching for `diagnostics::flag` and `diagnostics::any_flag`.
pub(super) fn flag(name: &str) -> bool {
    std::env::args().any(|arg| arg == name)
}

/// True when any command-line flag satisfies `test`.
pub(super) fn any_flag(test: impl Fn(&str) -> bool) -> bool {
    std::env::args().any(|arg| test(&arg))
}

pub(super) mod startup_commands;

#[path = "dictionaries84.rs"]
pub(super) mod dictionaries84_checks;
pub(super) use dictionaries84_checks::*;
#[path = "pending_issues.rs"]
pub(super) mod pending_issue_checks;
pub(super) use pending_issue_checks::*;
#[path = "clipboard.rs"]
pub(super) mod clipboard_checks;
pub(super) use clipboard_checks::*;
#[path = "captions.rs"]
pub(super) mod caption_checks;
pub(super) use caption_checks::*;
#[path = "alpha31.rs"]
pub(super) mod alpha31_checks;
pub(super) use alpha31_checks::*;
#[path = "format136.rs"]
pub(super) mod format136_checks;
pub(super) use format136_checks::*;
#[path = "accessibility.rs"]
pub(super) mod accessibility_checks;
pub(super) use accessibility_checks::*;
#[path = "adaptive.rs"]
pub(super) mod adaptive_checks;
pub(super) use adaptive_checks::*;
#[path = "annotations.rs"]
pub(super) mod annotation_checks;
pub(super) use annotation_checks::*;
#[path = "canvas.rs"]
pub(super) mod canvas_checks;
pub(super) use canvas_checks::*;
#[path = "columns.rs"]
pub(super) mod columns_checks;
pub(super) use columns_checks::*;
#[path = "context.rs"]
pub(super) mod context_checks;
pub(super) use context_checks::*;
#[path = "navigation131.rs"]
pub(super) mod navigation131_checks;
pub(super) use navigation131_checks::*;
#[path = "alpha30.rs"]
pub(super) mod alpha30_checks;
pub(super) use alpha30_checks::*;
#[path = "review29.rs"]
pub(super) mod review29_checks;
pub(super) use review29_checks::*;
#[path = "alpha29.rs"]
pub(super) mod alpha29_checks;
pub(super) use alpha29_checks::*;
#[path = "alpha28.rs"]
pub(super) mod alpha28_checks;
pub(super) use alpha28_checks::*;
#[path = "register.rs"]
pub(super) mod diagnostic_registration;
pub(super) use diagnostic_registration::*;
#[path = "sweep18.rs"]
pub(super) mod diagnostic_sweep18;
pub(super) use diagnostic_sweep18::*;
#[path = "sweep19.rs"]
pub(super) mod diagnostic_sweep19;
pub(super) use diagnostic_sweep19::*;
#[path = "sweep20.rs"]
pub(super) mod diagnostic_sweep20;
pub(super) use diagnostic_sweep20::*;
#[path = "document_tools.rs"]
pub(super) mod document_tools_checks;
pub(super) use document_tools_checks::*;
#[path = "header_options.rs"]
pub(super) mod header_options_checks;
pub(super) use header_options_checks::*;
#[path = "headers.rs"]
pub(super) mod headers_checks;
pub(super) use headers_checks::*;
#[path = "interactions.rs"]
pub(super) mod interaction_checks;
pub(super) use interaction_checks::*;
#[path = "issues.rs"]
pub(super) mod issue_checks;
pub(super) use issue_checks::*;
#[path = "lifecycle.rs"]
pub(super) mod lifecycle_checks;
pub(super) use lifecycle_checks::*;
#[path = "localization.rs"]
pub(super) mod localization_checks;
pub(super) use localization_checks::*;
#[path = "motion.rs"]
pub(super) mod motion_checks;
pub(super) use motion_checks::*;
#[path = "navigation.rs"]
pub(super) mod navigation_checks;
pub(super) use navigation_checks::*;
#[path = "paintable.rs"]
pub(super) mod paintable_checks;
pub(super) use paintable_checks::*;
#[path = "paragraph.rs"]
pub(super) mod paragraph_checks;
pub(super) use paragraph_checks::*;
#[path = "performance.rs"]
pub(super) mod performance_checks;
pub(super) use performance_checks::*;
#[path = "performance_checks.rs"]
pub(super) mod performance_checks_checks;
pub(super) use performance_checks_checks::*;
#[path = "popovers.rs"]
pub(super) mod popovers_checks;
pub(super) use popovers_checks::*;
#[path = "printing.rs"]
pub(super) mod printing_checks;
pub(super) use printing_checks::*;
#[path = "search.rs"]
pub(super) mod search_checks;
pub(super) use search_checks::*;
#[path = "spellcheck.rs"]
pub(super) mod spellcheck_checks;
pub(super) use spellcheck_checks::*;
#[path = "startup.rs"]
pub(super) mod startup_demos;
pub(super) use startup_demos::*;
#[path = "styles.rs"]
pub(super) mod styles_checks;
pub(super) use styles_checks::*;
#[path = "view_tests.rs"]
pub(super) mod view_tests_checks;
pub(super) use view_tests_checks::*;
#[path = "dialogs.rs"]
pub(super) mod dialog_checks;
#[path = "workflows35.rs"]
pub(super) mod workflows35_checks;
#[path = "workflows36.rs"]
pub(super) mod workflows36_checks;
pub(super) use dialog_checks::*;
#[path = "alpha42.rs"]
pub(super) mod alpha42_checks;
#[path = "alpha43.rs"]
pub(super) mod alpha43_checks;
#[path = "capture_ready.rs"]
pub(super) mod capture_ready;
#[path = "maintenance45.rs"]
pub(super) mod maintenance45_checks;
#[path = "toolbar41.rs"]
pub(super) mod toolbar41_checks;
#[path = "windows.rs"]
pub(super) mod windows_checks;
#[path = "workflows37.rs"]
pub(super) mod workflows37_checks;
#[path = "workflows38.rs"]
pub(super) mod workflows38_checks;
#[path = "workflows40.rs"]
pub(super) mod workflows40_checks;
