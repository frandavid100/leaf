use super::*;

// Header/footer contents are engine stories edited on the same document canvas.
use wordcraft_gnome::header_footer::{self,Region,NumberPosition};

pub(super) fn header_change(ui:&Rc<Ui>,f:impl FnOnce(&mut Session)->Result<bool,String>) {
    let result={let mut value=ui.current.borrow_mut();let Some(d)=value.as_mut()else{return;};f(&mut d.session)};
    match result {Ok(true)=>after_content_change(ui),Ok(false)=>refresh_selection(ui),Err(e)=>ui.toasts.add_toast(Toast::new(&e))}
    ui.document_box.grab_focus();caret_ensure_visible(ui);
}
pub(super) fn enter_header(ui:&Rc<Ui>,region:Region) {
    let page=ui.current.borrow().as_ref().map(|d|d.active).unwrap_or(0);
    header_change(ui,|s|header_footer::enter(s,page,region));
}
pub(super) fn refresh_header_context(ui:&Rc<Ui>) {
    let region=ui.current.borrow().as_ref().and_then(|d|header_footer::active(&d.session));
    ui.header_context.set_visible(region.is_some());
    if let Some(region)=region {ui.header_label.set_text(if region==Region::Header {wordcraft_gnome::t!("Editing Header")}else{wordcraft_gnome::t!("Editing Footer")});}
}

pub(super) fn header_context_bar()->(gtk::Box,gtk::Label,gtk::Button) {
    let bar=gtk::Box::new(gtk::Orientation::Horizontal,12);bar.add_css_class("toolbar");bar.set_visible(false);
    let label=gtk::Label::new(None);label.set_xalign(0.0);label.set_hexpand(true);label.set_margin_start(12);bar.append(&label);
    let close=gtk::Button::with_label(wordcraft_gnome::t!("Return to Document"));close.set_widget_name("close-header-footer");bar.append(&close);
    (bar,label,close)
}

// Double-clicking a previously empty page margin creates its editable story.
pub(super) fn enter_empty_header_at(ui:&Rc<Ui>,x:f64,y:f64)->bool {
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

pub(super) struct PageNumberDialog {pub(super) window:adw::Window,pub(super) position:adw::ComboRow,pub(super) align:adw::ComboRow,pub(super) total:gtk::Switch,pub(super) insert:gtk::Button}
pub(super) fn show_page_number(ui:&Rc<Ui>,parent:&ApplicationWindow)->PageNumberDialog {
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


