//! Read-only selection state for the existing engine-owned table property editors.
use super::*;
use serde_json::{Value, json};
pub(super) fn state(ui: &Rc<Ui>) -> Option<Value> {
    let c = ui.current.borrow();
    let s = &c.as_ref()?.session;
    let (path, row, col) = s.sel.focus.path.cell()?;
    let table = s.doc.table(s.sel.focus.story, &path)?;
    let mut out = serde_json::to_value(&table.props).ok()?;
    let cells = wordcraft_engine::cmd::table::selected_cells(s);
    let cells = if cells.is_empty() {
        vec![(row, col)]
    } else {
        cells
    };
    let common = |values: Vec<Value>| -> Value {
        values
            .first()
            .filter(|first| values.iter().all(|v| v == *first))
            .cloned()
            .unwrap_or(Value::Null)
    };
    let mut widths = Vec::new();
    let mut heights = Vec::new();
    let mut rules = Vec::new();
    let mut headers = Vec::new();
    let mut split = Vec::new();
    let mut valign = Vec::new();
    let mut directions = Vec::new();
    let mut cellwidth = Vec::new();
    for (r, c) in &cells {
        let row = table.rows.get(*r)?;
        let cell = row.cells.get(*c)?;
        let first = table.grid_col(*r, *c);
        for col in first..first + cell.span() {
            widths.push(json!(table.grid.get(col)));
        }
        heights.push(json!(row.props.height));
        rules.push(json!(row.props.height_rule));
        headers.push(json!(row.props.header));
        split.push(json!(row.props.cant_split));
        valign.push(json!(cell.props.valign));
        directions.push(json!(cell.props.text_direction));
        cellwidth.push(json!(cell.props.width));
    }
    out["_hasAutomaticHeight"] = json!(heights.iter().any(|v| v.is_null()));
    out["columnWidth"] = common(widths);
    out["row"] = json!({"height":common(heights),"heightRule":common(rules),"header":common(headers),"cantSplit":common(split)});
    out["cell"] = json!({"valign":common(valign),"textDirection":common(directions),"width":common(cellwidth)});
    out["selectedCount"] = json!(cells.len());
    Some(out)
}
pub(super) fn select_current(
    dialog: &adw::PreferencesDialog,
    name: &str,
    values: &[&str],
    value: &Value,
) {
    let Some(row) = find_widget(dialog, name).and_downcast::<adw::ComboRow>() else {
        return;
    };
    let index = value
        .as_str()
        .and_then(|v| values.iter().position(|candidate| *candidate == v))
        .map_or(0, |i| i as u32 + 1);
    row.set_selected(index);
}
pub(super) fn bind_refresh(ui: &Rc<Ui>, dialog: &adw::PreferencesDialog) {
    let weak = Rc::downgrade(ui);
    let window = dialog.downgrade();
    let previous = RefCell::new(None::<(u64, u64, String)>);
    dialog.add_tick_callback(move |_, _| {
        let (Some(ui), Some(w)) = (weak.upgrade(), window.upgrade()) else {
            return glib::ControlFlow::Break;
        };
        if ui.closed.get() || !w.is_mapped() {
            return glib::ControlFlow::Break;
        }
        let current = ui.current.borrow().as_ref().map(|d| {
            (
                d.session.document_id(),
                d.session.rev(),
                format!("{:?}", d.session.sel),
            )
        });
        if *previous.borrow() == current {
            return glib::ControlFlow::Continue;
        }
        previous.replace(current);
        let Some(state) = state(&ui) else {
            return glib::ControlFlow::Continue;
        };
        let old = ui.format_syncing.replace(true);
        for (name, value, factor) in [
            (
                "tool-control-table.properties-width",
                &state["width"],
                72. / 2.54,
            ),
            (
                "tool-control-table.properties-indent",
                &state["indent"],
                72. / 2.54,
            ),
            (
                "tool-control-table.columnWidth-width",
                &state["columnWidth"],
                72. / 2.54,
            ),
            (
                "tool-control-table.properties-rowHeight",
                &state["row"]["height"],
                72. / 2.54,
            ),
            (
                "tool-control-table.properties-cellWidth",
                &state["cell"]["width"],
                72. / 2.54,
            ),
        ] {
            if let Some(n) = find_widget(&w, name)
                .and_downcast::<gtk::SpinButton>()
                .filter(|n| !control_has_focus(n))
            {
                if name == "tool-control-table.properties-rowHeight" {
                    n.set_sensitive(!state["row"]["heightRule"].is_null());
                }
                if let Some(v) = value.as_f64() {
                    n.set_value(v / factor);
                } else {
                    n.set_text("");
                    n.set_tooltip_text(Some(wordcraft_gnome::t!("Mixed or automatic")));
                }
            }
        }
        select_current(
            &w,
            "tool-control-table.properties-align",
            &["left", "center", "right"],
            &state["align"],
        );
        select_current(
            &w,
            "tool-control-table.properties-valign",
            &["top", "center", "bottom"],
            &state["cell"]["valign"],
        );
        select_current(
            &w,
            "tool-control-table.textDirection-value",
            &["horizontal", "down", "up"],
            &state["cell"]["textDirection"],
        );
        select_current(
            &w,
            "tool-control-table.properties-rowHeightRule",
            &["auto", "atLeast", "exact"],
            &state["row"]["heightRule"],
        );
        for (name, value, invert) in [
            (
                "tool-control-table.properties-headerRow",
                &state["row"]["header"],
                false,
            ),
            (
                "tool-control-table.properties-allowBreak",
                &state["row"]["cantSplit"],
                true,
            ),
        ] {
            if let Some(check) = find_widget(&w, name).and_downcast::<gtk::CheckButton>() {
                check.set_inconsistent(value.is_null());
                check.set_active(value.as_bool().is_some_and(|v| v != invert));
            }
        }
        ui.format_syncing.set(old);
        glib::ControlFlow::Continue
    });
}
