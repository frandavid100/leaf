// Header/footer contents are engine stories edited on the same document canvas.
use wordcraft_gnome::header_footer::{self,Region,NumberPosition};

fn header_change(ui:&Rc<Ui>,f:impl FnOnce(&mut Session)->Result<bool,String>) {
    let result={let mut value=ui.current.borrow_mut();let Some(d)=value.as_mut()else{return;};f(&mut d.session)};
    match result {Ok(true)=>after_content_change(ui),Ok(false)=>refresh_selection(ui),Err(e)=>ui.toasts.add_toast(Toast::new(&e))}
    ui.document_box.grab_focus();caret_ensure_visible(ui);
}
fn enter_header(ui:&Rc<Ui>,region:Region) {
    let page=ui.current.borrow().as_ref().map(|d|d.active).unwrap_or(0);
    header_change(ui,|s|header_footer::enter(s,page,region));
}
fn refresh_header_context(ui:&Rc<Ui>) {
    let region=ui.current.borrow().as_ref().and_then(|d|header_footer::active(&d.session));
    ui.header_context.set_visible(region.is_some());
    if let Some(region)=region {ui.header_label.set_text(if region==Region::Header {wordcraft_gnome::t!("Editing Header")}else{wordcraft_gnome::t!("Editing Footer")});}
}

fn header_context_bar()->(gtk::Box,gtk::Label,gtk::Button) {
    let bar=gtk::Box::new(gtk::Orientation::Horizontal,12);bar.add_css_class("toolbar");bar.set_visible(false);
    let label=gtk::Label::new(None);label.set_xalign(0.0);label.set_hexpand(true);label.set_margin_start(12);bar.append(&label);
    let close=gtk::Button::with_label(wordcraft_gnome::t!("Return to Document"));close.set_widget_name("close-header-footer");bar.append(&close);
    (bar,label,close)
}

// Double-clicking a previously empty page margin creates its editable story.
fn enter_empty_header_at(ui:&Rc<Ui>,x:f64,y:f64)->bool {
    let target={let mut value=ui.current.borrow_mut();let Some(d)=value.as_mut()else{return false;};
        d.page_frames.iter().enumerate().find_map(|(i,frame)|{
            let r=frame.compute_bounds(&ui.document_box)?;
            if x<r.x()as f64||x>(r.x()+r.width())as f64||y<r.y()as f64||y>(r.y()+r.height())as f64{return None;}
            let py=((y-r.y()as f64)/d.zoom as f64)as f32;
            let (story,header)=d.session.layout().header_footer_zone(i,py)?;
            if story.is_some(){return None;}Some((i,if header{Region::Header}else{Region::Footer}))
        })};
    if let Some((page,region))=target{header_change(ui,|s|header_footer::enter(s,page,region));true}else{false}
}

struct PageNumberDialog {window:adw::Window,position:adw::ComboRow,align:adw::ComboRow,total:gtk::Switch,insert:gtk::Button}
fn show_page_number(ui:&Rc<Ui>,parent:&ApplicationWindow)->PageNumberDialog {
    let identity=document_identity(ui);
    let (page,in_header)={let value=ui.current.borrow();let d=value.as_ref().unwrap();(d.active,d.session.in_header_footer())};
    let window=adw::Window::builder().title(wordcraft_gnome::t!("Page Number")).transient_for(parent).modal(true).default_width(430).default_height(350).build();
    let toolbar=ToolbarView::new();let header=HeaderBar::new();header.set_show_start_title_buttons(false);header.set_show_end_title_buttons(false);
    let cancel=gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));header.pack_start(&cancel);
    let insert=gtk::Button::with_label(wordcraft_gnome::t!("Insert"));insert.add_css_class("suggested-action");header.pack_end(&insert);toolbar.add_top_bar(&header);
    let body=adw::PreferencesPage::new();let group=adw::PreferencesGroup::new();group.set_description(Some(wordcraft_gnome::t!("Page numbers update automatically. Existing header and footer content is kept.")));body.add(&group);
    let position=adw::ComboRow::new();position.set_title(wordcraft_gnome::t!("Position"));position.set_model(Some(&gtk::StringList::new(&[wordcraft_gnome::t!("At the Cursor"),wordcraft_gnome::t!("In the Header"),wordcraft_gnome::t!("In the Footer")])));position.set_selected(if in_header{0}else{2});group.add(&position);
    let align=adw::ComboRow::new();align.set_title(wordcraft_gnome::t!("Alignment"));align.set_model(Some(&gtk::StringList::new(&[wordcraft_gnome::t!("Left"),wordcraft_gnome::t!("Centered"),wordcraft_gnome::t!("Right")])));align.set_selected(1);align.set_sensitive(!in_header);group.add(&align);
    let row=adw::ActionRow::new();row.set_title(wordcraft_gnome::t!("Include Total Pages"));let total=gtk::Switch::new();total.set_valign(gtk::Align::Center);row.add_suffix(&total);row.set_activatable_widget(Some(&total));group.add(&row);
    {let a=align.downgrade();position.connect_selected_notify(move|p|{if let Some(a)=a.upgrade(){a.set_sensitive(p.selected()!=0);}});}
    let error=gtk::Label::new(None);error.set_wrap(true);error.add_css_class("error");body.add(&{let g=adw::PreferencesGroup::new();g.add(&error);g});
    toolbar.set_content(Some(&body));window.set_content(Some(&toolbar));
    {let w=window.downgrade();cancel.connect_clicked(move|_|{if let Some(w)=w.upgrade(){w.close();}});}
    {let weak=Rc::downgrade(ui);let w=window.downgrade();let p=position.clone();let a=align.clone();let total=total.clone();
        insert.connect_clicked(move|_|{
            let(Some(ui),Some(w))=(weak.upgrade(),w.upgrade())else{return;};
            if ui.closed.get()||document_identity(&ui)!=identity{error.set_text(wordcraft_gnome::t!("The document has changed; open this dialog again."));return;}
            let result={let mut value=ui.current.borrow_mut();let Some(d)=value.as_mut()else{return;};
                header_footer::insert_number(&mut d.session,page,match p.selected(){1=>NumberPosition::Header,2=>NumberPosition::Footer,_=>NumberPosition::Current},match a.selected(){0=>"left",2=>"right",_=>"center"},total.is_active())};
            match result{Ok(changed)=>{w.close();if changed{after_content_change(&ui);}else{refresh_selection(&ui);}ui.document_box.grab_focus();caret_ensure_visible(&ui);},Err(e)=>error.set_text(&e)}
        });
    }
    window.present();PageNumberDialog{window,position,align,total,insert}
}

fn header_footer_ui_test(ui:&Rc<Ui>,parent:&ApplicationWindow,app:&Application,pointer:bool) {
    let text=(0..150).map(|i|format!("Paragraph {i}: test body text.\n")).collect::<String>();
    opened(ui,Session::new(wordcraft_doc::Document::from_text(&text)),"Page stories");
    let body_text=ui.current.borrow().as_ref().unwrap().session.doc.plain_text(wordcraft_doc::StoryRef::Body);
    let back={let mut v=ui.current.borrow_mut();let d=v.as_mut().unwrap();
        selection::set_caret(&mut d.session,&Pos::body(100,5),false).unwrap();d.session.sel.clone()};
    header_change(ui,|s|header_footer::enter(s,1,Region::Header));
    apply_user_edit(ui,Edit::Insert("Editable page header".into()));assert!(ui.header_context.is_visible());
    header_change(ui,header_footer::close);assert!(!ui.header_context.is_visible());
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel,back);
    let dialog=show_page_number(ui,parent);dialog.position.set_selected(2);dialog.align.set_selected(2);dialog.total.set_active(true);dialog.insert.emit_clicked();
    assert!(!dialog.window.is_visible());assert_eq!(ui.current.borrow().as_ref().and_then(|d|header_footer::active(&d.session)),Some(Region::Footer));
    header_change(ui,|s|header_footer::enter(s,1,Region::Header));
    ui.dirty.set(false);ui.current.borrow_mut().as_mut().unwrap().session.dirty=false;
    let u=ui.clone();let w=parent.clone();let a=app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(400),move||{
        let (expected,x,y)={let mut value=u.current.borrow_mut();let d=value.as_mut().unwrap();
            let mut pos=d.session.doc.start_of(d.session.sel.focus.story);pos.off=1;
            let c=d.session.layout().caret_on(&pos,1).unwrap();
            let frame=d.page_frames[1].compute_bounds(&u.document_box).unwrap();
            let canvas=u.document_box.compute_bounds(&w).unwrap();
            let x=canvas.x()as f64+frame.x()as f64+c.x as f64*d.zoom as f64;
            let y=canvas.y()as f64+frame.y()as f64+(c.top+c.height/2.0)as f64*d.zoom as f64;
            (pos,x,y)};
        if pointer {eprintln!("HEADER_POINTER_READY {} {}",x as i32,y as i32);}
        else {let mut v=u.current.borrow_mut();selection::set_caret(&mut v.as_mut().unwrap().session,&expected,false).unwrap();}
        glib::timeout_add_local_once(std::time::Duration::from_secs(if pointer{2}else{0}),move||{
            {let value=u.current.borrow();let d=value.as_ref().unwrap();assert_eq!(d.session.sel.focus.story,expected.story);assert_eq!(d.session.page_hint,1);assert!(d.session.sel.focus.off.abs_diff(expected.off)<=1);assert!(!d.session.dirty);}
            assert!(!u.dirty.get());
            if pointer {eprintln!("HEADER_RETURN_READY");}else{header_change(&u,header_footer::close);}
            glib::timeout_add_local_once(std::time::Duration::from_secs(if pointer{2}else{0}),move||{
                assert!(!u.header_context.is_visible());let v=u.current.borrow();let d=v.as_ref().unwrap();assert_eq!(d.session.sel,back);
                assert_eq!(d.session.doc.plain_text(wordcraft_doc::StoryRef::Body),body_text);
                eprintln!("HEADER_FOOTER_UI_OK pointer={pointer}");a.quit();
            });
        });
    });
}
