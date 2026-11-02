use super::*;

pub(super) struct PinTest {
    pub(super) ui: Rc<Ui>,
    pub(super) sm: gtk::MenuButton,
    pub(super) s: Rc<StylePopover>,
    pub(super) pm: gtk::MenuButton,
    pub(super) p: Rc<ParagraphPopover>,
    pub(super) tm: gtk::MenuButton,
    pub(super) tp: gtk::Popover,
    pub(super) tb: gtk::MenuButton,
    pub(super) app: Application,
}

pub(super) fn pin_ui_test(
    ui: &Rc<Ui>,
    sm: &gtk::MenuButton,
    s: &Rc<StylePopover>,
    pm: &gtk::MenuButton,
    p: &Rc<ParagraphPopover>,
    tm: &gtk::MenuButton,
    tp: &gtk::Popover,
    tb: &gtk::MenuButton,
    app: &Application,
) {
    pin_test_step(
        Rc::new(PinTest {
            ui: ui.clone(),
            sm: sm.clone(),
            s: s.clone(),
            pm: pm.clone(),
            p: p.clone(),
            tm: tm.clone(),
            tp: tp.clone(),
            tb: tb.clone(),
            app: app.clone(),
        }),
        0,
    );
}

pub(super) fn pin_test_step(t: Rc<PinTest>, step: u8) {
    let ui = &t.ui;
    match step {
        0 => {
            t.sm.popup();
            let sp = t.sm.popover().unwrap();
            let rev = ui.current.borrow().as_ref().unwrap().session.rev();
            let surface = sp.surface();
            let hides = Rc::new(Cell::new(0));
            let n = hides.clone();
            sp.connect_hide(move |_| n.set(n.get() + 1));
            popover_pin(&sp).set_active(true);
            popover_pin(&sp).set_active(false);
            popover_pin(&sp).set_active(true);
            assert_eq!(hides.get(), 0, "pinning must never hide the popup");
            assert_eq!(surface, sp.surface());
            assert!(!sp.is_autohide());
            assert_eq!(rev, ui.current.borrow().as_ref().unwrap().session.rev());
            style_click(&t.s, "style-apply-Heading1");
        }
        1 => {
            let sp = t.sm.popover().unwrap();
            assert!(
                sp.is_mapped(),
                "pinned style must stay drawn after apply and focus"
            );
            t.pm.popup();
            let pp = t.pm.popover().unwrap();
            assert!(!sp.is_visible());
            assert!(
                popover_pin(&pp).is_active(),
                "pin must carry to replacement panel"
            );
            t.sm.popup();
            assert!(!pp.is_visible());
            assert!(popover_pin(&sp).is_active());
            style_click(&t.s, "style-manage-Heading1");
            popover_pin(&sp).set_active(false);
            assert_eq!(
                t.s.stack.visible_child_name().as_deref(),
                Some("detail"),
                "unpin must preserve subpage"
            );
        }
        2 => {
            assert!(
                t.sm.popover().unwrap().is_mapped(),
                "unpin must remap the popup"
            );
            t.s.main();
            style_click(&t.s, "style-apply-Normal");
            assert!(!t.sm.popover().unwrap().is_visible());
            t.pm.popup();
            let pp = t.pm.popover().unwrap();
            popover_pin(&pp).set_active(true);
            t.p.execute("para.align", serde_json::json!({"value":"center"}));
            assert_eq!(selection_paragraph(ui)["align"], "center");
        }
        3 => {
            let pp = t.pm.popover().unwrap();
            assert!(pp.is_mapped(), "pinned paragraph must stay drawn");
            t.p.page("spacing");
            popover_pin(&pp).set_active(false);
            assert_eq!(t.p.stack.visible_child_name().as_deref(), Some("detail"));
            t.p.execute("para.align", serde_json::json!({"value":"left"}));
            assert!(!pp.is_visible());
            t.tm.popup();
            popover_pin(&t.tp).set_active(true);
            ui.format_bold.set_active(!ui.format_bold.is_active());
        }
        4 => {
            assert!(
                t.tp.is_mapped(),
                "pinned text must stay drawn after toggling bold"
            );
            popover_pin(&t.tp).set_active(false);
            ui.size_input.set_value(18.0);
            assert!(t.tp.is_visible(), "editing a value keeps its popover open");
            assert_eq!(
                wordcraft_engine::cmd::format::state(
                    &ui.current.borrow().as_ref().unwrap().session
                )["size"],
                18.0
            );
            t.tp.popdown();
            format_execute(ui, "insert.table", serde_json::json!({"rows":2,"cols":2})).unwrap();
            t.tb.popup();
            let table = t.tb.popover().unwrap();
            popover_pin(&table).set_active(true);
            find_widget(&table.child().unwrap(), "table-action-table.insertRowBelow")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
        }
        5 => {
            let table = t.tb.popover().unwrap();
            assert!(table.is_mapped(), "pinned table must stay drawn");
            find_widget(&table.child().unwrap(), "table-delete")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            assert!(
                !find_widget(&table.child().unwrap(), "table-context")
                    .unwrap()
                    .is_visible()
            );
        }
        _ => {
            let table = t.tb.popover().unwrap();
            assert!(
                table.is_mapped(),
                "deleting table must not hide pinned popup"
            );
            popover_pin(&table).set_active(false);
            close_format_popover(&table);
            assert!(!table.is_visible());
            assert_eq!(t.app.windows().len(), 1);
            eprintln!("POPOVER_PIN_OK");
            t.app.quit();
            return;
        }
    }
    glib::timeout_add_local_once(std::time::Duration::from_millis(220), move || {
        pin_test_step(t, step + 1)
    });
}
