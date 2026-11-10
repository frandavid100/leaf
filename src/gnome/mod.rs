//! Native GTK frontend. Shared tab state lives here; WordCraft remains the model.
use adw::prelude::*;
use adw::{
    Application, ApplicationWindow, HeaderBar, StatusPage, Toast, ToastOverlay, ToolbarView,
    WindowTitle,
};
use glib::value::ToValue;
use glib::variant::ToVariant;
use gtk::{gdk, gio, glib};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    path::{Path, PathBuf},
    rc::Rc,
};
use wordcraft_doc::Pos;
use wordcraft_engine::Session;
use wordcraft_gnome::editing::{self, Edit};
use wordcraft_gnome::formatting::{self, Format};
use wordcraft_gnome::selection::{self, SelectionGeometry};
use wordcraft_gnome::viewer_layout;
use wordcraft_gnome::{open_document, render_page_screen as render_page, sample_session};

use wordcraft_gnome::document_properties::ColumnScope;
use wordcraft_gnome::header_footer::{self, Region};
use wordcraft_gnome::page_options;
use wordcraft_gnome::proofing;
use wordcraft_gnome::{
    document_properties::{self, PageSetup},
    pdf_export, pictures,
};

const APP_VERSION: &str = "0.1.0-alpha.43";
const APP_ID: &str = "io.github.frandavid100.leaf";
const STEPS: &[f32] = &[0.5, 0.67, 0.75, 0.85, 1.0, 1.25, 1.5, 1.75, 2.0];

// Native state, controllers and panels. Siblings are private to this frontend.
mod accessibility;
use accessibility::*;
mod adaptive;
mod dictionaries;
use adaptive::*;
use dictionaries::*;
#[path = "diagnostics/dictionaries84.rs"]
mod dictionaries84_checks;
use dictionaries84_checks::*;
mod annotation_dialog;
use annotation_dialog::*;
mod annotations;
use annotations::*;
mod appearance;
use appearance::*;
mod canvas;
use canvas::*;
mod caption_cues;
use caption_cues::*;
mod clipboard;
use clipboard::*;
mod color_chooser;
use color_chooser::*;
mod columns;
use columns::*;
mod comment_composer;
use comment_composer::*;
mod commands;
use commands::*;
mod context;
mod context_clipboard;
use context::*;
mod document_properties_dialog;
use document_properties_dialog::*;
mod document_tools;
use document_tools::*;
mod close_group;
mod files;
use files::*;
mod format_controls;
use format_controls::*;
mod format_bar;
mod header_options;
mod tool_actions;
mod toolbar_preferences;
use header_options::*;
mod headers;
use headers::*;
mod history;
use history::*;
mod keyboard_input;
use keyboard_input::*;
mod links;
use links::*;
mod menus;
use menus::*;
mod navigation;
use navigation::*;
mod paintable;
use paintable::*;
mod list_options;
mod paragraph;
mod paragraph_options;
use paragraph::*;
mod paragraph_icons;
use paragraph_icons::*;
mod pdf_dialog;
use pdf_dialog::*;
mod performance;
use performance::*;
mod elements;
mod equations;
use elements::*;
use equations::*;
mod reference_geometry;
use reference_geometry::*;
mod references;
use references::*;
mod picture_pointer;
use picture_pointer::*;
mod image_options;
mod picture_controls;
use picture_controls::*;
mod pointer_input;
use pointer_input::*;
mod popovers;
use popovers::*;
mod printing;
use printing::*;
mod review;
use review::*;
mod ruler;
use ruler::*;
mod search;
use search::*;
mod selection_drag;
use selection_drag::*;
mod selection_style;
use selection_style::*;
mod selection_view;
use selection_view::*;
mod spellcheck;
use spellcheck::*;
mod state;
use state::*;
mod styles;
use styles::*;
mod tab;
use tab::*;
mod table_pointer;
use table_pointer::*;
mod table_properties;
mod tables;
use tables::*;
mod text_controls;
use text_controls::*;
mod text_palette;
use text_palette::*;
mod view;
use view::*;
mod widgets;
use widgets::*;
mod workspace;
use workspace::*;

// Opt-in diagnostics and synthetic fixtures; no normal startup work.
#[path = "diagnostics/pending_issues.rs"]
mod pending_issue_checks;
use pending_issue_checks::*;
#[path = "diagnostics/clipboard.rs"]
mod clipboard_checks;
use clipboard_checks::*;
#[path = "diagnostics/captions.rs"]
mod caption_checks;
use caption_checks::*;
#[path = "diagnostics/alpha31.rs"]
mod alpha31_checks;
use alpha31_checks::*;
#[path = "diagnostics/format136.rs"]
mod format136_checks;
use format136_checks::*;
#[path = "diagnostics/accessibility.rs"]
mod accessibility_checks;
use accessibility_checks::*;
#[path = "diagnostics/adaptive.rs"]
mod adaptive_checks;
use adaptive_checks::*;
#[path = "diagnostics/annotations.rs"]
mod annotation_checks;
use annotation_checks::*;
#[path = "diagnostics/canvas.rs"]
mod canvas_checks;
use canvas_checks::*;
#[path = "diagnostics/columns.rs"]
mod columns_checks;
use columns_checks::*;
#[path = "diagnostics/context.rs"]
mod context_checks;
use context_checks::*;
#[path = "diagnostics/navigation131.rs"]
mod navigation131_checks;
use navigation131_checks::*;
#[path = "diagnostics/alpha30.rs"]
mod alpha30_checks;
use alpha30_checks::*;
#[path = "diagnostics/review29.rs"]
mod review29_checks;
use review29_checks::*;
#[path = "diagnostics/alpha29.rs"]
mod alpha29_checks;
use alpha29_checks::*;
#[path = "diagnostics/alpha28.rs"]
mod alpha28_checks;
use alpha28_checks::*;
#[path = "diagnostics/register.rs"]
mod diagnostic_registration;
use diagnostic_registration::*;
#[path = "diagnostics/sweep18.rs"]
mod diagnostic_sweep18;
use diagnostic_sweep18::*;
#[path = "diagnostics/sweep19.rs"]
mod diagnostic_sweep19;
use diagnostic_sweep19::*;
#[path = "diagnostics/sweep20.rs"]
mod diagnostic_sweep20;
use diagnostic_sweep20::*;
#[path = "diagnostics/document_tools.rs"]
mod document_tools_checks;
use document_tools_checks::*;
#[path = "diagnostics/header_options.rs"]
mod header_options_checks;
use header_options_checks::*;
#[path = "diagnostics/headers.rs"]
mod headers_checks;
use headers_checks::*;
#[path = "diagnostics/interactions.rs"]
mod interaction_checks;
use interaction_checks::*;
#[path = "diagnostics/issues.rs"]
mod issue_checks;
use issue_checks::*;
#[path = "diagnostics/lifecycle.rs"]
mod lifecycle_checks;
use lifecycle_checks::*;
#[path = "diagnostics/localization.rs"]
mod localization_checks;
use localization_checks::*;
#[path = "diagnostics/motion.rs"]
mod motion_checks;
use motion_checks::*;
#[path = "diagnostics/navigation.rs"]
mod navigation_checks;
use navigation_checks::*;
#[path = "diagnostics/paintable.rs"]
mod paintable_checks;
use paintable_checks::*;
#[path = "diagnostics/paragraph.rs"]
mod paragraph_checks;
use paragraph_checks::*;
#[path = "diagnostics/performance.rs"]
mod performance_checks;
use performance_checks::*;
#[path = "diagnostics/performance_checks.rs"]
mod performance_checks_checks;
use performance_checks_checks::*;
#[path = "diagnostics/popovers.rs"]
mod popovers_checks;
use popovers_checks::*;
#[path = "diagnostics/printing.rs"]
mod printing_checks;
use printing_checks::*;
#[path = "diagnostics/search.rs"]
mod search_checks;
use search_checks::*;
#[path = "diagnostics/spellcheck.rs"]
mod spellcheck_checks;
use spellcheck_checks::*;
#[path = "diagnostics/startup.rs"]
mod startup_demos;
use startup_demos::*;
#[path = "diagnostics/styles.rs"]
mod styles_checks;
use styles_checks::*;
#[path = "diagnostics/view_tests.rs"]
mod view_tests_checks;
use view_tests_checks::*;

mod application;
pub(crate) fn run() {
    application::run();
}

mod format_painter;

mod templates;

mod document_fonts;

mod image_drop;

#[path = "diagnostics/workflows35.rs"]
mod workflows35_checks;

#[path = "diagnostics/workflows36.rs"]
mod workflows36_checks;

#[path = "diagnostics/dialogs.rs"]
mod dialog_checks;
#[path = "diagnostics/windows.rs"]
mod windows_checks;
use dialog_checks::*;

#[path = "diagnostics/workflows37.rs"]
mod workflows37_checks;

#[path = "diagnostics/alpha42.rs"]
mod alpha42_checks;
#[path = "diagnostics/alpha43.rs"]
mod alpha43_checks;
#[path = "diagnostics/toolbar41.rs"]
mod toolbar41_checks;
#[path = "diagnostics/workflows38.rs"]
mod workflows38_checks;
#[path = "diagnostics/workflows40.rs"]
mod workflows40_checks;

#[path = "diagnostics/capture_ready.rs"]
mod capture_ready;
