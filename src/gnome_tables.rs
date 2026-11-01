// Native table controls. Document changes remain upstream commands.
fn table_command(ui:&Rc<Ui>,command:&str,args:serde_json::Value) {
    if let Err(e)=format_execute(ui,command,args) {ui.toasts.add_toast(Toast::new(&e));}
    caret_ensure_visible(ui);
    ui.document_box.grab_focus();
    if let Some(menu)=style_widget(&ui.format_toolbar,"table-menu").and_then(|w|w.downcast::<gtk::MenuButton>().ok()){if let Some(pop)=menu.popover(){refresh_table_controls(ui,&pop);}}
}

fn table_navigate(ui:&Rc<Ui>,backwards:bool,literal:bool) {
    let result={let mut state=ui.current.borrow_mut();state.as_mut().map(|d|wordcraft_gnome::tables::navigate(&mut d.session,backwards,literal))};
    match result {Some(Ok(true))=>after_content_change(ui),Some(Err(e))=>ui.toasts.add_toast(Toast::new(&e)),_=>refresh_selection(ui)}
    caret_ensure_visible(ui);
}

fn show_insert_table(ui:&Rc<Ui>,parent:&ApplicationWindow) {
    let w=panel_window(parent,wordcraft_gnome::t!("Insert Table"));w.set_default_size(440,640);w.set_search_enabled(false);
    let p=panel_page(&w,wordcraft_gnome::t!("Table"));let g=panel_group(&p,wordcraft_gnome::t!("Table Size"));
    let grid=gtk::Grid::new();grid.set_row_spacing(3);grid.set_column_spacing(3);grid.set_halign(gtk::Align::Center);
    let label=gtk::Label::new(Some(wordcraft_gnome::t!("Choose Columns × Rows")));g.add(&label);
    for row in 1..=5 {for col in 1..=6 {
        let b=gtk::Button::with_label("□");b.set_tooltip_text(Some(&format!("{col} columnas × {row} filas")));accessible_name(&b,&wordcraft_gnome::i18n::interpolate(wordcraft_gnome::t!("Insert a table with {col} columns and {row} rows"), &[("col", format!("{}", col)), ("row", format!("{}", row))]));
        b.set_size_request(30,28);grid.attach(&b,col-1,row-1,1,1);
        let motion=gtk::EventControllerMotion::new();let l=label.clone();motion.connect_enter(move|_,_,_|l.set_text(&format!("{col} columnas × {row} filas")));b.add_controller(motion);
        let ui=ui.clone();let win=w.clone();b.connect_clicked(glib::clone!(#[weak] ui, #[weak] win, move|_|{
            if let Err(e)=format_execute(&ui,"insert.table",serde_json::json!({"rows":row,"cols":col})) {win.add_toast(Toast::new(&e));}
            else {win.close();ui.document_box.grab_focus();caret_ensure_visible(&ui);}
        }));
    }}g.add(&grid);
    let g=panel_group(&p,wordcraft_gnome::t!("Custom Size"));
    let rows=panel_number(&g,wordcraft_gnome::t!("Rows"),3.0,1.0,1000.0);rows.set_digits(0);rows.set_increments(1.0,5.0);
    let cols=panel_number(&g,wordcraft_gnome::t!("Columns"),3.0,1.0,63.0);cols.set_digits(0);cols.set_increments(1.0,5.0);
    let b=gtk::Button::with_label(wordcraft_gnome::t!("Insert Table"));b.add_css_class("suggested-action");g.add(&b);
    let ui=ui.clone();let win=w.clone();b.connect_clicked(glib::clone!(#[weak] ui, #[weak] win, move|_|{
        rows.update();cols.update();
        if let Err(e)=format_execute(&ui,"insert.table",serde_json::json!({"rows":rows.value_as_int(),"cols":cols.value_as_int()})){win.add_toast(Toast::new(&e));}
        else {win.close();ui.document_box.grab_focus();caret_ensure_visible(&ui);}
    }));w.present();
}

fn table_property_toggle(g:&adw::PreferencesGroup,label:&str,on:bool,key:&'static str,ui:&Rc<Ui>,w:&adw::PreferencesWindow) {
    let row=adw::SwitchRow::new();row.set_title(label);row.set_active(on);g.add(&row);
    let ui=ui.clone();let w=w.clone();row.connect_active_notify(glib::clone!(#[weak] ui, #[weak] w, move|r|{
        if let Err(e)=format_execute(&ui,"table.properties",serde_json::json!({key:r.is_active()})){w.add_toast(Toast::new(&e));}
    }));
}

fn table_measurement(g:&adw::PreferencesGroup,label:&str,value:f64,min:f64,ui:&Rc<Ui>,w:&adw::PreferencesWindow,command:&'static str,key:&'static str,extra:serde_json::Value) {
    let row=adw::ActionRow::new();row.set_title(label);
    let n=gtk::SpinButton::with_range(min,1584.0,0.5);n.set_digits(1);n.set_value(value);n.set_valign(gtk::Align::Center);
    let b=gtk::Button::with_label(wordcraft_gnome::t!("Apply"));b.set_valign(gtk::Align::Center);accessible_name(&n,label);accessible_name(&b,&wordcraft_gnome::i18n::interpolate(wordcraft_gnome::t!("Apply {label}"), &[("label", format!("{}", label))]));row.set_activatable_widget(Some(&n));row.add_suffix(&n);row.add_suffix(&b);g.add(&row);
    let ui=ui.clone();let w=w.clone();b.connect_clicked(glib::clone!(#[weak] ui, #[weak] w, move|_|{n.update();let mut args=extra.clone();args[key]=serde_json::json!(n.value());
        if let Err(e)=format_execute(&ui,command,args){w.add_toast(Toast::new(&e));}
    }));
}

fn show_table_properties(ui:&Rc<Ui>,parent:&ApplicationWindow) {
    let state={let mut state=ui.current.borrow_mut();let Some(doc)=state.as_mut() else{return};
        match doc.session.run("table.properties",&serde_json::json!({})){Ok(v)=>v,Err(e)=>{ui.toasts.add_toast(Toast::new(&e.to_string()));return;}}};
    let w=panel_window(parent,wordcraft_gnome::t!("Table Properties"));w.set_search_enabled(false);
    let p=panel_page(&w,wordcraft_gnome::t!("Size"));let g=panel_group(&p,wordcraft_gnome::t!("Table"));
    panel_choice(&g,wordcraft_gnome::t!("Table Alignment"),&[(wordcraft_gnome::t!("Left"),"left"),(wordcraft_gnome::t!("Centered"),"center"),(wordcraft_gnome::t!("Right"),"right")],ui,&w,"table.properties","align");
    panel_choice(&g,wordcraft_gnome::t!("Fit Width"),&[(wordcraft_gnome::t!("Fit Contents"),"contents"),(wordcraft_gnome::t!("Page Width"),"window"),(wordcraft_gnome::t!("Fixed Width"),"fixed")],ui,&w,"table.autofit","mode");
    panel_action(&g,wordcraft_gnome::t!("Distribute Columns Evenly"),ui,&w,"table.distributeColumns",serde_json::json!({}));
    panel_action(&g,wordcraft_gnome::t!("Distribute Rows Evenly"),ui,&w,"table.distributeRows",serde_json::json!({}));
    let g=panel_group(&p,wordcraft_gnome::t!("Selected Column and Row (Points)"));
    table_measurement(&g,wordcraft_gnome::t!("Column Width"),state["columnWidth"].as_f64().unwrap_or(72.0),6.0,ui,&w,"table.columnWidth","width",serde_json::json!({}));
    table_measurement(&g,wordcraft_gnome::t!("Minimum Row Height"),state["row"]["height"].as_f64().unwrap_or(18.0),1.0,ui,&w,"table.rowHeight","height",serde_json::json!({"rule":"atLeast"}));
    panel_action(&g,wordcraft_gnome::t!("Restore Automatic Height for Current Row"),ui,&w,"table.properties",serde_json::json!({"rowHeight":null}));
    let g=panel_group(&p,wordcraft_gnome::t!("Current Row Pagination"));
    table_property_toggle(&g,wordcraft_gnome::t!("Repeat as Header"),state["row"]["header"].as_bool().unwrap_or(false),"headerRow",ui,&w);
    table_property_toggle(&g,wordcraft_gnome::t!("Allow Row to Split Across Pages"),!state["row"]["cantSplit"].as_bool().unwrap_or(false),"allowBreak",ui,&w);
    let p=panel_page(&w,wordcraft_gnome::t!("Cells"));let g=panel_group(&p,wordcraft_gnome::t!("Content Alignment"));
    panel_choice(&g,wordcraft_gnome::t!("Horizontal and Vertical Alignment"),&[(wordcraft_gnome::t!("Top · Left"),"topLeft"),(wordcraft_gnome::t!("Top · Center"),"topCenter"),(wordcraft_gnome::t!("Top · Right"),"topRight"),(wordcraft_gnome::t!("Center · Left"),"centerLeft"),(wordcraft_gnome::t!("Center"),"center"),(wordcraft_gnome::t!("Center · Right"),"centerRight"),(wordcraft_gnome::t!("Bottom · Left"),"bottomLeft"),(wordcraft_gnome::t!("Bottom · Center"),"bottomCenter"),(wordcraft_gnome::t!("Bottom · Right"),"bottomRight")],ui,&w,"table.cellAlign","value");
    panel_choice(&g,wordcraft_gnome::t!("Text Direction"),&[(wordcraft_gnome::t!("Landscape"),"horizontal"),(wordcraft_gnome::t!("Vertical Downwards"),"down"),(wordcraft_gnome::t!("Vertical Upwards"),"up")],ui,&w,"table.textDirection","value");
    let g=panel_group(&p,wordcraft_gnome::t!("Split Cells"));let cols=panel_number(&g,wordcraft_gnome::t!("Columns"),2.0,1.0,63.0);cols.set_digits(0);cols.set_increments(1.0,1.0);
    let rows=panel_number(&g,wordcraft_gnome::t!("Rows"),1.0,1.0,63.0);rows.set_digits(0);rows.set_increments(1.0,1.0);
    let hint=gtk::Label::new(Some(wordcraft_gnome::t!("Multiple selected cells are merged before splitting.")));hint.set_wrap(true);hint.add_css_class("dim-label");g.add(&hint);
    let b=gtk::Button::with_label(wordcraft_gnome::t!("Split Selected Cells"));g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(glib::clone!(#[weak] ui, #[weak] w, move|_|{cols.update();rows.update();if let Err(e)=format_execute(&ui,"table.split",serde_json::json!({"columns":cols.value_as_int(),"rows":rows.value_as_int()})){w.add_toast(Toast::new(&e));}}));}
    let p=panel_page(&w,wordcraft_gnome::t!("Appearance"));let g=panel_group(&p,wordcraft_gnome::t!("Selection Borders and Shading"));
    panel_color(&g,wordcraft_gnome::t!("Background Color"),ui,&w,"table.shading");panel_action(&g,wordcraft_gnome::t!("No Background Color"),ui,&w,"table.shading",serde_json::json!({"color":null}));
    panel_choice(&g,wordcraft_gnome::t!("Borders"),&[(wordcraft_gnome::t!("All"),"all"),(wordcraft_gnome::t!("Outside"),"outside"),(wordcraft_gnome::t!("Inside"),"inside"),(wordcraft_gnome::t!("None"),"none"),(wordcraft_gnome::t!("Top"),"top"),(wordcraft_gnome::t!("Bottom"),"bottom"),(wordcraft_gnome::t!("Left"),"left"),(wordcraft_gnome::t!("Right"),"right")],ui,&w,"table.borders","kind");
    let width=panel_number(&g,wordcraft_gnome::t!("Border Width (pt)"),0.5,0.25,12.0);let color=panel_entry(&g,wordcraft_gnome::t!("Border Color (RRGGBB)"),"808080");
    let b=gtk::Button::with_label(wordcraft_gnome::t!("Apply to All Borders"));g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(glib::clone!(#[weak] ui, #[weak] w, move|_|{width.update();let c=color.text().trim().to_string();if c.len()!=6||!c.bytes().all(|b|b.is_ascii_hexdigit()){w.add_toast(Toast::new(wordcraft_gnome::t!("Enter Six Hexadecimal Characters, for Example 808080")));return;}
        if let Err(e)=format_execute(&ui,"table.borders",serde_json::json!({"kind":"all","width":width.value(),"color":c})){w.add_toast(Toast::new(&e));}}));}
    w.present();
}

fn install_table_controls(ui:&Rc<Ui>,parent:&ApplicationWindow,bar:&gtk::Box)->gtk::MenuButton {
    let menu=gtk::MenuButton::new();menu.set_widget_name("table-menu");format_menu_label(&menu,wordcraft_gnome::t!("Table"));menu.set_direction(gtk::ArrowType::Up);menu.set_tooltip_text(Some(wordcraft_gnome::t!("Insert and Edit Tables")));
    let pop=gtk::Popover::new();pop.add_css_class("format-popover");let content=gtk::Box::new(gtk::Orientation::Vertical,6);
    content.set_margin_top(8);content.set_margin_bottom(8);content.set_margin_start(8);content.set_margin_end(8);
    format_popover_header(&pop,&content,wordcraft_gnome::t!("Table"),&ui.format_group);
    let insert=gtk::Button::with_label(wordcraft_gnome::t!("Insert Table…"));insert.add_css_class("flat");insert.set_visible(tool_diagnostics());content.append(&insert);
    {let ui=ui.clone();let p=parent.clone();let pop=pop.clone();insert.connect_clicked(glib::clone!(#[weak] ui, #[weak] p, #[weak] pop, move|_|{close_format_popover(&pop);show_insert_table(&ui,&p);}));}
    let context=gtk::Box::new(gtk::Orientation::Vertical,6);context.set_widget_name("table-context");content.append(&context);
    let heading=gtk::Label::new(Some(wordcraft_gnome::t!("Current Table")));heading.add_css_class("heading");context.append(&heading);
    for items in [
        vec![(wordcraft_gnome::t!("Select Row"),"table.selectRow"),(wordcraft_gnome::t!("Select Column"),"table.selectColumn")],
        vec![(wordcraft_gnome::t!("Select Cell"),"table.selectCell"),(wordcraft_gnome::t!("Select Table"),"table.selectTable")],
        vec![(wordcraft_gnome::t!("Row Above"),"table.insertRowAbove"),(wordcraft_gnome::t!("Row Below"),"table.insertRowBelow")],
        vec![(wordcraft_gnome::t!("Left Column"),"table.insertColumnLeft"),(wordcraft_gnome::t!("Right Column"),"table.insertColumnRight")],
        vec![(wordcraft_gnome::t!("Merge Cells"),"table.merge"),(wordcraft_gnome::t!("Split Table"),"table.splitTable")],
        vec![(wordcraft_gnome::t!("Delete Row"),"table.deleteRow"),(wordcraft_gnome::t!("Delete Column"),"table.deleteColumn")],
    ] {
        let row=gtk::Box::new(gtk::Orientation::Horizontal,4);
        for(label,command)in items {let b=gtk::Button::with_label(label);b.add_css_class("flat");b.set_hexpand(true);row.append(&b);b.set_widget_name(&format!("table-action-{command}"));if command=="table.merge"{b.set_widget_name("table-merge");}
            let ui=ui.clone();let pop=pop.clone();b.connect_clicked(glib::clone!(#[weak] ui, #[weak] pop, move|_|{close_format_popover(&pop);table_command(&ui,command,serde_json::json!({}));}));}
        context.append(&row);
    }
    let properties=gtk::Button::with_label(wordcraft_gnome::t!("Properties and Split Cells…"));properties.add_css_class("flat");context.append(&properties);
    {let ui=ui.clone();let parent=parent.clone();let pop=pop.clone();properties.connect_clicked(glib::clone!(#[weak] ui, #[weak] parent, #[weak] pop, move|_|{close_format_popover(&pop);show_table_properties(&ui,&parent);}));}
    let remove=gtk::Button::with_label(wordcraft_gnome::t!("Delete Table"));remove.set_widget_name("table-delete");remove.add_css_class("destructive-action");context.append(&remove);
    {let ui=ui.clone();let pop=pop.clone();remove.connect_clicked(glib::clone!(#[weak] ui, #[weak] pop, move|_|{close_format_popover(&pop);table_command(&ui,"table.deleteTable",serde_json::json!({}));}));}
    {let ui=ui.clone();pop.connect_show(glib::clone!(#[weak] ui, move|p|refresh_table_controls(&ui,p)));}
    pop.set_child(Some(&content));menu.set_popover(Some(&pop));bar.append(&menu);menu
}

fn refresh_table_controls(ui:&Rc<Ui>,pop:&gtk::Popover){
    let Some(root)=pop.child()else{return;};let value=ui.current.borrow();let doc=value.as_ref();
    if let Some(w)=style_widget(&root,"table-context"){w.set_visible(doc.is_some_and(|d|wordcraft_gnome::tables::in_table(&d.session)));}
    if let Some(w)=style_widget(&root,"table-merge"){w.set_sensitive(doc.is_some_and(|d|wordcraft_gnome::tables::selected_cells(&d.session)>1));}
}

