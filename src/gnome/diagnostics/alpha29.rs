use super::*;
use serde_json::json;

fn apply_reference_dialog(window: &adw::Dialog) {
    wait_dialog_presented(window);
    find_widget(window, "reference-apply")
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    assert_dialog_closed(window);
}

pub(super) fn alpha29_ui_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    window.set_default_size(1250, 950);
    let text = (0..300)
        .map(|i| format!("Paragraph {i}: a long document for sidebar navigation.\n"))
        .collect::<String>();
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(&text)),
        "Thumbnail navigation",
    );
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
        let last = u.current.borrow().as_ref().unwrap().sizes.len() - 1;
        jump_to(&u, last);
        // Page navigation dismisses the collapsed overlay. Reopening Pages
        // must reveal the active page too, at both desktop raster scales.
        u.sidebar.set_show_sidebar(true);
        refresh_thumbnails(&u);
        reveal_active_thumbnail(&u);
        let (u, w, a) = (u.clone(), w.clone(), a.clone());
        glib::timeout_add_local_once(std::time::Duration::from_millis(450), move || {
            {
                let c = u.current.borrow();
                let d = c.as_ref().unwrap();
                let r = d.thumb_buttons[d.active]
                    .compute_bounds(&u.thumbnails_scroll)
                    .unwrap();
                let v = u.thumbnails_scroll.vadjustment();
                assert!(
                    r.y() >= -1. && r.y() + r.height() <= u.thumbnails_scroll.height() as f32 + 1.,
                    "active={} bounds={r:?} scroll={} page={} upper={} sidebar={} collapsed={}",
                    d.active,
                    v.value(),
                    v.page_size(),
                    v.upper(),
                    u.sidebar.shows_sidebar(),
                    u.sidebar.is_collapsed()
                );
                assert!(
                    d.thumbs[d.active].paintable().is_some(),
                    "the revealed thumbnail must be rasterized"
                );
                let label = d.thumb_buttons[d.active]
                    .child()
                    .unwrap()
                    .last_child()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap();
                assert!(label.text().contains(&d.sizes.len().to_string()));
            }
            u.thumbnails_scroll.vadjustment().set_value(0.);
            refresh_thumbnails(&u);
            update_status(&u);
            assert!(
                u.thumbnails_scroll.vadjustment().value() < 1.,
                "redraw must not override manual sidebar browsing"
            );
            alpha29_auto_test(&u, &w, &a);
        });
    });
}

fn alpha29_controls(ui: &Rc<Ui>, window: &ApplicationWindow) {
    let host = ui.host.upgrade().unwrap();
    let mut s = Session::new(wordcraft_doc::Document::from_text("Heading\nA paragraph."));
    s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
    opened(ui, s, "References");
    let link = show_link(ui).unwrap();
    wait_dialog_presented(&link.window);
    find_widget(&link.window, "link-kind")
        .unwrap()
        .downcast::<gtk::DropDown>()
        .unwrap()
        .set_selected(1);
    assert!(link.apply.is_sensitive());
    link.apply.emit_clicked();
    assert_dialog_closed(&link.window);
    assert!(
        serde_json::to_string(&ui.current.borrow().as_ref().unwrap().session.doc)
            .unwrap()
            .contains("REF _Ref")
    );
    show_contents_dialog(ui, window);
    let toc = window
        .visible_dialog()
        .filter(|w| w.title() == wordcraft_gnome::t!("Table of Contents"))
        .unwrap();
    apply_reference_dialog(&toc);
    {
        let mut c = ui.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        let at = wordcraft_gnome::references::toc_field(s).unwrap().0;
        selection::set_caret(s, &at, false).unwrap();
    }
    let pop = show_document_context(ui, window, 100, 100, DocumentContext::Text);
    assert!(pop.activate_action("context.edit-toc", None).is_ok());
    let elements = ui.elements.borrow().as_ref().unwrap().clone();
    find_widget(&elements.pop, "elements-toc-apply")
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    elements.pop.popdown();
    pop.popdown();
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text("A table caption")),
        "Captions",
    );
    refresh_selection(ui);
    assert!(
        !host
            .window
            .lookup_action("insert-caption")
            .unwrap()
            .is_enabled()
    );
    {
        let mut c = ui.current.borrow_mut();
        c.as_mut()
            .unwrap()
            .session
            .run("insert.table", &json!({"rows":2,"cols":2}))
            .unwrap();
    }
    after_content_change(ui);
    assert!(
        host.window
            .lookup_action("insert-caption")
            .unwrap()
            .is_enabled()
    );
    host.window
        .lookup_action("insert-caption")
        .unwrap()
        .activate(None);
    {
        let c = ui.current.borrow();
        let s = &c.as_ref().unwrap().session;
        assert!(
            s.doc
                .para_at(&s.sel.focus)
                .unwrap()
                .plain_text()
                .ends_with(": ")
        );
    }
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text("An equation")),
        "Equations",
    );
    show_equation(ui, window);
    let eq = window
        .visible_dialog()
        .filter(|w| w.title() == wordcraft_gnome::t!("Equation"))
        .unwrap();
    find_widget(&eq, "equation-input")
        .unwrap()
        .downcast::<gtk::Entry>()
        .unwrap()
        .set_text("(a+b)/(c+d)+√(x)");
    apply_reference_dialog(&eq);
    assert!(
        wordcraft_gnome::equations::target(&ui.current.borrow().as_ref().unwrap().session)
            .is_some()
    );
    let mut s = wordcraft_gnome::proofing_defaults::new_document();
    s.run("text.insert", &json!({"text":"Hola"})).unwrap();
    wordcraft_gnome::references::insert_footnote(&mut s).unwrap();
    s.run(
        "text.insert",
        &json!({"text":"Probando los pies de página."}),
    )
    .unwrap();
    let layout = s.layout();
    let c = layout.caret_on(&s.sel.focus, 0).unwrap();
    assert!(c.top + c.height < layout.pages[c.page].h);
    let draws = wordcraft_engine::layout::display::page_display(
        &s.doc,
        &layout.pages[0],
        &Default::default(),
    );
    assert!(draws.iter().filter(|d|matches!(d,wordcraft_engine::layout::display::Draw::Glyphs{text,..} if text=="1")).count()>=2,"both note numbers must render");
    opened(ui, s, "Footnotes");
    let props = show_document_properties(ui, window);
    wait_dialog_presented(&props.window);
    find_widget(&props.window, "properties-footnote-start")
        .unwrap()
        .downcast::<gtk::SpinButton>()
        .unwrap()
        .set_value(4.);
    props.apply.emit_clicked();
    assert_dialog_closed(&props.window);
    {
        let c = ui.current.borrow();
        let s = &c.as_ref().unwrap().session;
        assert_eq!(
            s.doc.note_options(s.doc.sections()[0].1, false).num_start,
            4
        );
    }
    let mut s = Session::new(wordcraft_doc::Document::from_text("Current filename: "));
    wordcraft_gnome::references::insert_field(&mut s, "FILENAME").unwrap();
    opened(ui, s, "File name field");
    let old_raster = ui.current.borrow().as_ref().unwrap().pages[0]
        .paintable()
        .unwrap();
    let path = std::env::temp_dir().join("leaf29-filename-display.docx");
    assert!(save_to(ui, &path));
    assert!(!ui.dirty.get());
    assert!(!ui.current.borrow().as_ref().unwrap().session.dirty);
    assert_ne!(
        ui.current.borrow().as_ref().unwrap().pages[0]
            .paintable()
            .as_ref(),
        Some(&old_raster),
        "Save As must repaint the changed filename field"
    );
    // Section Properties must target their section, independently of the caret.
    let mut s = Session::new(wordcraft_doc::Document::from_text(
        "First section\nSecond section",
    ));
    selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
    s.run("layout.break", &json!({"kind":"nextPage"})).unwrap();
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    let caret = s.sel.clone();
    let before = s.doc.clone();
    opened(ui, s, "Section note settings");
    let properties = show_section_properties(ui, window, 1);
    wait_dialog_presented(&properties.window);
    find_widget(&properties.window, "properties-footnote-start")
        .unwrap()
        .downcast::<gtk::SpinButton>()
        .unwrap()
        .set_value(9.);
    properties.apply.emit_clicked();
    assert_dialog_closed(&properties.window);
    {
        let mut c = ui.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        assert_eq!(s.sel, caret);
        let sections = s.doc.sections();
        assert_eq!(s.doc.note_options(sections[0].1, false).num_start, 1);
        assert_eq!(s.doc.note_options(sections[1].1, false).num_start, 9);
        assert!(
            sections[1].1.footnote_pr.num_fmt.is_none(),
            "unchanged format remains inherited"
        );
        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(s.doc, before);
        s.run("edit.redo", &json!({})).unwrap();
        let path = std::env::temp_dir().join("leaf29-section-notes.docx");
        wordcraft_gnome::saving::save(s, &path).unwrap();
        let copy = wordcraft_gnome::open_document(&path).unwrap();
        assert_eq!(
            copy.doc
                .note_options(copy.doc.sections()[1].1, false)
                .num_start,
            9
        );
    }
}

fn alpha29_image_stage(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application, stage: usize) {
    let mut s = Session::new(wordcraft_doc::Document::from_text(
        "Source paragraph for a picture.\nDestination paragraph for a picture.",
    ));
    selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
    s.run("layout.break", &json!({"kind":"nextPage"})).unwrap();
    let mut setup = PageSetup::current(&s);
    std::mem::swap(&mut setup.width, &mut setup.height);
    document_properties::apply(&mut s, setup, false).unwrap();
    let reverse = stage == 18;
    let at = if reverse {
        s.doc
            .para_paths(wordcraft_doc::StoryRef::Body)
            .last()
            .unwrap()
            .clone()
    } else {
        wordcraft_doc::Path::top(0)
    };
    selection::set_caret(
        &mut s,
        &Pos {
            story: wordcraft_doc::StoryRef::Body,
            path: at,
            off: 0,
        },
        false,
    )
    .unwrap();
    let file = std::env::temp_dir().join("leaf29-image.png");
    std::fs::write(
        &file,
        wordcraft_engine::cmd::insert::base64_decode(include_str!(
            "../../../tests/fixtures/picture.b64"
        ))
        .unwrap(),
    )
    .unwrap();
    pictures::insert(&mut s, &file).unwrap();
    pictures::resize(&mut s, 120., 80., false, true).unwrap();
    let inline = (8..16).contains(&stage) || stage == 17;
    if !inline {
        pictures::placement(&mut s, Some("left"), 6.).unwrap();
    }
    opened(ui, s, "Image gestures");
    window.set_title(Some("Leaf Alpha29 Regression"));
    zoom_to(ui, if stage < 16 { 0.9 } else { 0.4 });
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
        let (ax, ay, bx, by, before, depth, old_w, old_h) = {
            let mut c = u.current.borrow_mut();
            let d = c.as_mut().unwrap();
            let (pos, _, _) = pictures::selected(&d.session).unwrap();
            let hit = d
                .session
                .layout()
                .object(&pos, usize::from(reverse))
                .unwrap();
            let r = d.page_frames[hit.page].compute_bounds(&w).unwrap();
            let z = d.page_zoom(hit.page);
            let scale = u.document_box.scale_factor() as f32;
            let (tx, ty) = w.surface_transform();
            let (ex, ey) = if stage < 16 {
                [
                    (-1, -1),
                    (0, -1),
                    (1, -1),
                    (-1, 0),
                    (1, 0),
                    (-1, 1),
                    (0, 1),
                    (1, 1),
                ][stage % 8]
            } else {
                (0, 0)
            };
            let px = match ex {
                -1 => hit.rect.x + 1.,
                1 => hit.rect.right() - 1.,
                _ => hit.rect.x + hit.rect.w / 2.,
            };
            let py = match ey {
                -1 => hit.rect.y + 1.,
                1 => hit.rect.bottom() - 1.,
                _ => hit.rect.y + hit.rect.h / 2.,
            };
            let ax = ((r.x() + px * z + tx as f32) * scale).round() as i32;
            let ay = ((r.y() + py * z + ty as f32) * scale).round() as i32;
            let (bx, by) = if stage < 16 {
                (ax + ex * 20 * scale as i32, ay + ey * 20 * scale as i32)
            } else if stage == 19 {
                (ax + 35 * scale as i32, ay + 25 * scale as i32)
            } else if stage == 20 {
                let top = d.page_frames[0].compute_bounds(&w).unwrap();
                (
                    ax,
                    ((top.y() + top.height() + 12. + ty as f32) * scale) as i32,
                )
            } else {
                let target = usize::from(!reverse);
                let r = d.page_frames[target].compute_bounds(&w).unwrap();
                let z = d.page_zoom(target);
                (
                    ((r.x() + 150. * z + tx as f32) * scale) as i32,
                    ((r.y() + 160. * z + ty as f32) * scale) as i32,
                )
            };
            (
                ax,
                ay,
                bx,
                by,
                d.session.doc.clone(),
                d.session.undo_depth(),
                hit.rect.w,
                hit.rect.h,
            )
        };
        eprintln!(
            "ALPHA29_POINTER {stage} {} {ax} {ay} {bx} {by}",
            if stage == 19 { "cancel" } else { "drag" }
        );
        let dir = std::env::var("LEAF29_ACK_DIR").unwrap();
        let tries = Cell::new(0);
        glib::timeout_add_local(std::time::Duration::from_millis(80), move || {
            tries.set(tries.get() + 1);
            assert!(tries.get() < 400, "alpha29 pointer stalled at {stage}");
            if !Path::new(&dir).join(stage.to_string()).exists() {
                return glib::ControlFlow::Continue;
            }
            {
                let mut c = u.current.borrow_mut();
                let d = c.as_mut().unwrap();
                if stage >= 19 {
                    assert_eq!(
                        d.session.doc, before,
                        "canceled/invalid drop must preserve the document"
                    );
                    assert_eq!(d.session.undo_depth(), depth);
                } else {
                    assert_eq!(
                        d.session.undo_depth(),
                        depth + 1,
                        "one undo per gesture at {stage}"
                    );
                    let (pos, _, _) = pictures::selected(&d.session).unwrap();
                    let hit = d
                        .session
                        .layout()
                        .object(&pos, usize::from(!reverse))
                        .unwrap();
                    if stage < 16 {
                        let (ex, ey) = [
                            (-1, -1),
                            (0, -1),
                            (1, -1),
                            (-1, 0),
                            (1, 0),
                            (-1, 1),
                            (0, 1),
                            (1, 1),
                        ][stage % 8];
                        if ex != 0 {
                            assert!(hit.rect.w > old_w + 8., "resize width at {stage}")
                        }
                        if ey != 0 {
                            assert!(hit.rect.h > old_h + 8., "resize height at {stage}")
                        }
                    } else {
                        assert_eq!(
                            hit.page,
                            usize::from(!reverse),
                            "destination page at {stage}"
                        );
                    }
                    d.session.run("edit.undo", &json!({})).unwrap();
                    assert_eq!(d.session.doc, before);
                    d.session.run("edit.redo", &json!({})).unwrap();
                }
            }
            if stage == 20 {
                alpha29_note_pointer(&u, &w, &a, 21);
            } else {
                alpha29_image_stage(&u, &w, &a, stage + 1);
            }
            glib::ControlFlow::Break
        });
    });
}

fn alpha29_note_pointer(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application, stage: usize) {
    if stage == 21 {
        let mut s = wordcraft_gnome::proofing_defaults::new_document();
        s.run("text.insert", &json!({"text":"Hola"})).unwrap();
        wordcraft_gnome::references::insert_footnote(&mut s).unwrap();
        s.run("text.insert", &json!({"text":"A directly navigable note."}))
            .unwrap();
        opened(ui, s, "Footnote number navigation");
        window.set_title(Some("Leaf Alpha29 Regression"));
        zoom_to(ui, 0.85);
    }
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
        let (x, y, depth) = {
            let mut c = u.current.borrow_mut();
            let d = c.as_mut().unwrap();
            let (reference, id) = wordcraft_gnome::references::footnote_refs(&d.session)[0].clone();
            let pos = if stage == 21 {
                reference
            } else {
                d.session.doc.start_of(wordcraft_doc::StoryRef::Part(id))
            };
            let end = Pos {
                off: pos.off + wordcraft_doc::para::OBJ.len_utf8(),
                ..pos.clone()
            };
            let l = d.session.layout();
            let (page, r) = l
                .selection_rects(&d.session.doc, &pos, &end, 0)
                .into_iter()
                .next()
                .expect("visible note-number geometry");
            let frame = d.page_frames[page].compute_bounds(&w).unwrap();
            let z = d.page_zoom(page);
            let scale = u.document_box.scale_factor() as f32;
            let (tx, ty) = w.surface_transform();
            (
                ((frame.x() + (r.x + r.w / 2.) * z + tx as f32) * scale) as i32,
                ((frame.y() + (r.y + r.h / 2.) * z + ty as f32) * scale) as i32,
                d.session.undo_depth(),
            )
        };
        eprintln!("ALPHA29_POINTER {stage} click {x} {y} {x} {y}");
        let dir = std::env::var("LEAF29_ACK_DIR").unwrap();
        let tries = Cell::new(0);
        glib::timeout_add_local(std::time::Duration::from_millis(80), move || {
            tries.set(tries.get() + 1);
            assert!(tries.get() < 400);
            if !Path::new(&dir).join(stage.to_string()).exists() {
                return glib::ControlFlow::Continue;
            }
            {
                let c = u.current.borrow();
                let s = &c.as_ref().unwrap().session;
                assert_eq!(s.undo_depth(), depth);
                assert_eq!(
                    matches!(s.sel.focus.story, wordcraft_doc::StoryRef::Part(_)),
                    stage == 21,
                    "number click must navigate in the right direction"
                );
            }
            if stage == 21 {
                alpha29_note_pointer(&u, &w, &a, 22)
            } else {
                alpha29_autoscroll(&u, &w, &a)
            }
            glib::ControlFlow::Break
        });
    });
}

fn alpha29_autoscroll(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let mut s = Session::new(wordcraft_doc::Document::from_text(
        &"A paragraph with enough text to form a long document.\n".repeat(200),
    ));
    pictures::insert(&mut s, &std::env::temp_dir().join("leaf29-image.png")).unwrap();
    pictures::resize(&mut s, 120., 80., false, true).unwrap();
    pictures::placement(&mut s, Some("left"), 6.).unwrap();
    opened(ui, s, "Picture edge scrolling");
    fit_width(ui);
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    glib::timeout_add_local_once(std::time::Duration::from_millis(450), move || {
        let (x, y, before, depth) = {
            let mut c = u.current.borrow_mut();
            let d = c.as_mut().unwrap();
            let (pos, _, _) = pictures::selected(&d.session).unwrap();
            let hit = d.session.layout().object(&pos, 0).unwrap();
            let r = d.page_frames[0].compute_bounds(&u.document_box).unwrap();
            let z = d.page_zoom(0);
            (
                (r.x() + (hit.rect.x + hit.rect.w / 2.) * z) as f64,
                (r.y() + (hit.rect.y + hit.rect.h / 2.) * z) as f64,
                d.session.doc.clone(),
                d.session.undo_depth(),
            )
        };
        assert!(picture_drag_begin(&u, x, y));
        let r = u.document_box.compute_bounds(&u.scroll).unwrap();
        let edge = u.scroll.height() as f64 - 3. - r.y() as f64;
        let initial = u.scroll.vadjustment().value();
        picture_drag_update(&u, 0., edge - y);
        glib::timeout_add_local_once(std::time::Duration::from_millis(450), move || {
            assert!(
                u.scroll.vadjustment().value() > initial + 10.,
                "picture edge drag must autoscroll"
            );
            assert!(picture_drag_cancel(&u));
            let c = u.current.borrow();
            let d = c.as_ref().unwrap();
            assert_eq!(d.session.doc, before);
            assert_eq!(d.session.undo_depth(), depth);
            let _ = w;
            eprintln!(
                "ALPHA29_POINTER_OK eight handles, inline/floating, cross-page, cancellation, invalid gaps, note clicks, autoscroll, Undo/Redo"
            );
            a.quit();
        });
    });
}

fn alpha29_auto_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let mut s = Session::new(wordcraft_doc::Document::from_text(
        "Heading\nA paragraph.\nSee: ",
    ));
    s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    selection::set_caret(&mut s, &Pos::body(2, 5), false).unwrap();
    s.run(
        "insert.crossReference",
        &json!({"to":"heading","target":"Heading","show":"text"}),
    )
    .unwrap();
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    s.run("references.toc", &json!({"title":"Contents","levels":3}))
        .unwrap();
    let mut heading = wordcraft_engine::cmd::citations::ref_targets(&s, "heading")
        .into_iter()
        .find(|(text, _)| text == "Heading")
        .unwrap()
        .1;
    heading.off = s
        .doc
        .para_at(&heading)
        .unwrap()
        .text
        .find("Heading")
        .unwrap();
    selection::set_caret(&mut s, &heading, false).unwrap();
    opened(ui, s, "Automatic references");
    {
        let mut c = ui.current.borrow_mut();
        c.as_mut()
            .unwrap()
            .session
            .run("text.insert", &json!({"text":"New "}))
            .unwrap();
    }
    after_content_change(ui);
    let caret = ui.current.borrow().as_ref().unwrap().session.sel.clone();
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
        let has_new = |s: &Session| {
            let mut found = false;
            s.doc.objects_in_reading_order(&mut |o| {
                if let wordcraft_doc::InlineObject::Field { instr, result, .. } = o {
                    if instr.starts_with("REF ") {
                        found |= result.contains("New Heading");
                    }
                }
            });
            found
        };
        {
            let c = u.current.borrow();
            let d = c.as_ref().unwrap();
            assert_eq!(d.session.sel, caret);
            assert!(
                has_new(&d.session),
                "coalesced reference refresh must run automatically"
            );
        }
        apply_user_edit(&u, Edit::Undo);
        {
            let c = u.current.borrow();
            let s = &c.as_ref().unwrap().session;
            assert!(!has_new(s));
            assert!(s.can_redo());
        }
        apply_user_edit(&u, Edit::Redo);
        assert!(has_new(&u.current.borrow().as_ref().unwrap().session));
        alpha29_controls(&u, &w);
        eprintln!(
            "ALPHA29_CONTROLS_OK thumbnails, automatic references/Undo/Redo, captions, fields, links, TOC settings, footnotes, equations"
        );
        alpha29_image_stage(&u, &w, &a, 0);
    });
}
