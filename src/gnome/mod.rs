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

const APP_VERSION: &str = "0.1.0-alpha.45";
const APP_ID: &str = "io.github.frandavid100.leaf";
const STEPS: &[f32] = &[0.5, 0.67, 0.75, 0.85, 1.0, 1.25, 1.5, 1.75, 2.0];

// Native state, controllers and panels. Siblings are private to this frontend.
mod accessibility;
use accessibility::*;
mod adaptive;
mod dictionaries;
use adaptive::*;
use dictionaries::*;
mod debounce;
use debounce::Debounce;
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
mod font_install;
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
mod diagnostics;
use diagnostics::*;

mod application;
pub(crate) fn run() {
    application::run();
}

mod format_painter;

mod templates;

mod document_fonts;

mod image_drop;
