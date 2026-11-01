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
use wordcraft_gnome::viewer_layout::{self, THUMB_SCALE};
use wordcraft_gnome::{open_document, render_page_screen as render_page, sample_session};

use wordcraft_gnome::document_properties::ColumnScope;
use wordcraft_gnome::header_footer::{self, Region};
use wordcraft_gnome::page_options;
use wordcraft_gnome::proofing;
use wordcraft_gnome::{
    document_properties::{self, PageSetup},
    pdf_export, pictures,
};

const APP_VERSION: &str = "0.1.0-alpha.14";
const APP_ID: &str = "io.github.frandavid100.leaf";
const STEPS: &[f32] = &[0.5, 0.67, 0.75, 0.85, 1.0, 1.25, 1.5, 1.75, 2.0];

mod state;
use state::*;
mod accessibility;
use accessibility::*;
mod adaptive;
use adaptive::*;
mod canvas;
use canvas::*;
mod columns;
use columns::*;
mod context;
use context::*;
mod document_tools;
use document_tools::*;
mod format_controls;
use format_controls::*;
mod headers;
use headers::*;
mod paintable;
use paintable::*;
mod workspace;
use workspace::*;
mod menus;
use menus::*;
mod navigation;
use navigation::*;
mod annotations;
use annotations::*;
mod annotation_dialog;
use annotation_dialog::*;
mod header_options;
use header_options::*;
mod paragraph;
use paragraph::*;
mod performance;
use performance::*;
mod popovers;
use popovers::*;
mod printing;
use printing::*;
mod spellcheck;
use spellcheck::*;
mod search;
use search::*;
mod styles;
use styles::*;
mod tables;
use tables::*;
mod selection_view;
use selection_view::*;
mod commands;
use commands::*;
mod files;
use files::*;
mod view;
use view::*;
mod appearance;
use appearance::*;
mod input;
use input::*;
mod tab;
use tab::*;
#[path = "diagnostics/accessibility.rs"]
mod accessibility_checks;
use accessibility_checks::*;
#[path = "diagnostics/adaptive.rs"]
mod adaptive_checks;
use adaptive_checks::*;
#[path = "diagnostics/canvas.rs"]
mod canvas_checks;
use canvas_checks::*;
#[path = "diagnostics/columns.rs"]
mod columns_checks;
use columns_checks::*;
#[path = "diagnostics/context.rs"]
mod context_checks;
use context_checks::*;
#[path = "diagnostics/document_tools.rs"]
mod document_tools_checks;
use document_tools_checks::*;
#[path = "diagnostics/headers.rs"]
mod headers_checks;
use headers_checks::*;
#[path = "diagnostics/paintable.rs"]
mod paintable_checks;
use paintable_checks::*;
#[path = "diagnostics/lifecycle.rs"]
mod lifecycle_checks;
use lifecycle_checks::*;
#[path = "diagnostics/localization.rs"]
mod localization_checks;
use localization_checks::*;
#[path = "diagnostics/navigation.rs"]
mod navigation_checks;
use navigation_checks::*;
#[path = "diagnostics/header_options.rs"]
mod header_options_checks;
use header_options_checks::*;
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
#[path = "diagnostics/spellcheck.rs"]
mod spellcheck_checks;
use spellcheck_checks::*;
#[path = "diagnostics/search.rs"]
mod search_checks;
use search_checks::*;
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

#[path = "diagnostics/register.rs"]
mod diagnostic_registration;
use diagnostic_registration::*;

#[path = "diagnostics/annotations.rs"]
mod annotation_checks;
use annotation_checks::*;

#[path = "diagnostics/motion.rs"]
mod motion_checks;
use motion_checks::*;
#[path = "diagnostics/startup.rs"]
mod startup_demos;
use startup_demos::*;
