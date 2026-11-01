use wordcraft_gnome::proofing::{self,Issue as ProofIssue};

fn refresh_all_proofing(ui:&Rc<Ui>){
    let mut docs=WORKSPACES.with(|hosts|hosts.borrow().iter().flat_map(|h|h.docs.borrow().iter().map(|(_,u)|u.clone()).collect::<Vec<_>>()).collect::<Vec<_>>());
    if docs.is_empty(){docs.push(ui.clone());}
    for u in docs {if u.closed.get(){continue;}if let Some(d)=u.current.borrow_mut().as_mut(){d.session.relayout();}refresh_document_appearance(&u);}
}
fn proof_replace(ui:&Rc<Ui>,issue:&ProofIssue,text:&str){
    let result={let mut v=ui.current.borrow_mut();let Some(d)=v.as_mut()else{return;};proofing::replace(&mut d.session,issue,text)};
    match result{Ok(true)=>after_content_change(ui),Ok(false)=>{},Err(e)=>ui.toasts.add_toast(Toast::new(&e))}
}
struct SpellReview {
    ui:std::rc::Weak<Ui>,window:adw::Window,word:gtk::Label,message:gtk::Label,replacement:gtk::Entry,
    suggestions:gtk::Box,change:gtk::Button,next:gtk::Button,ignore_all:gtk::Button,add:gtk::Button,
    index:Cell<usize>,issues:RefCell<Vec<ProofIssue>>,identity:Cell<Option<(u64,u64)>>,source:RefCell<Option<glib::SourceId>>,
}
impl SpellReview {
    fn scan(self:&Rc<Self>){
        if let Some(source)=self.source.borrow_mut().take(){source.remove();}
        let Some(ui)=self.ui.upgrade()else{return;};
        let Some((doc,prefs))=ui.current.borrow().as_ref().map(|d|(d.session.doc.clone(),d.session.prefs.clone()))else{return;};
        self.identity.set(document_identity(&ui));self.word.set_text(wordcraft_gnome::t!("Checking…"));self.message.set_text(wordcraft_gnome::t!("Languages without installed dictionaries are skipped."));self.actions(false);
        let (tx,rx)=std::sync::mpsc::channel();
        std::thread::spawn(move||{let mut session=Session::new(doc);session.prefs=prefs;let _=tx.send(proofing::issues(&session));});
        let weak=Rc::downgrade(self);
        let id=glib::timeout_add_local(std::time::Duration::from_millis(25),move||{
            let Some(s)=weak.upgrade()else{return glib::ControlFlow::Break;};
            match rx.try_recv(){
                Ok(issues)=>{s.source.borrow_mut().take();s.issues.replace(issues);s.show();glib::ControlFlow::Break},
                Err(std::sync::mpsc::TryRecvError::Empty)=>glib::ControlFlow::Continue,
                Err(_)=>{s.source.borrow_mut().take();s.word.set_text(wordcraft_gnome::t!("The spelling check could not finish."));glib::ControlFlow::Break},
            }
        });self.source.replace(Some(id));
    }
    fn actions(&self,on:bool){for b in [&self.change,&self.next,&self.ignore_all,&self.add]{b.set_sensitive(on);}self.replacement.set_sensitive(on);}
    fn show(self:&Rc<Self>){
        while let Some(c)=self.suggestions.first_child(){self.suggestions.remove(&c);}
        let Some(ui)=self.ui.upgrade()else{return;};
        if document_identity(&ui)!=self.identity.get(){self.word.set_text(wordcraft_gnome::t!("The text has changed. Check it again."));self.next.set_sensitive(true);return;}
        let items=self.issues.borrow();if items.is_empty(){self.word.set_text(wordcraft_gnome::t!("Check Complete"));self.message.set_text(wordcraft_gnome::t!("No issues found in text with available proofing tools."));self.actions(false);self.next.set_sensitive(true);return;}
        let index=self.index.get()%items.len();self.index.set(index);let issue=items[index].clone();drop(items);
        let suggestions={let mut v=ui.current.borrow_mut();let Some(d)=v.as_mut()else{return;};if let Err(e)=proofing::select(&mut d.session,&issue){self.message.set_text(&e);return;}proofing::suggestions(&mut d.session,&issue)};
        self.word.set_text(&issue.text);self.message.set_text(&issue.message);self.replacement.set_text(suggestions.first().map(String::as_str).unwrap_or(&issue.text));self.actions(true);self.add.set_sensitive(issue.spelling);self.ignore_all.set_sensitive(issue.spelling);
        for text in suggestions {let b=gtk::Button::with_label(&text);let weak=Rc::downgrade(self);b.connect_clicked(move|_|{if let Some(s)=weak.upgrade(){s.replacement.set_text(&text);s.replace();}});self.suggestions.append(&b);}
        if let Some(h)=ui.host.upgrade(){if let Some(page)=ui.tab_page.borrow().as_ref(){h.tabs.set_selected_page(page);}}refresh_selection(&ui);caret_ensure_visible(&ui);
    }
    fn item(&self)->Option<ProofIssue>{self.issues.borrow().get(self.index.get()).cloned()}
    fn replace(self:&Rc<Self>){let Some(ui)=self.ui.upgrade()else{return;};if document_identity(&ui)!=self.identity.get(){self.scan();return;}if let Some(i)=self.item(){proof_replace(&ui,&i,&self.replacement.text());self.scan();}}
}
fn show_spelling(ui:&Rc<Ui>,parent:&ApplicationWindow){
    if let Some(s)=ui.spelling.borrow().as_ref(){s.window.present();s.scan();return;}
    let window=adw::Window::builder().title(wordcraft_gnome::t!("Spelling and Grammar")).transient_for(parent).default_width(460).default_height(390).build();
    let toolbar=ToolbarView::new();toolbar.add_top_bar(&HeaderBar::new());let page=adw::PreferencesPage::new();let group=adw::PreferencesGroup::new();page.add(&group);
    let live=option_switch(&group,wordcraft_gnome::t!("Check Spelling as You Type"));live.set_active(wordcraft_gnome::preferences::check_spelling());
    {let weak=Rc::downgrade(ui);live.connect_active_notify(move|b|{if let Some(ui)=weak.upgrade(){if let Err(e)=wordcraft_gnome::preferences::set_check_spelling(b.is_active()){ui.toasts.add_toast(Toast::new(&e));return;}WORKSPACES.with(|hosts|{for h in hosts.borrow().iter(){for(_,u)in h.docs.borrow().iter(){if let Some(d)=u.current.borrow_mut().as_mut(){let _=d.session.run("review.proofing",&serde_json::json!({"value":b.is_active()}));}}}});refresh_all_proofing(&ui);}});}
    let body=gtk::Box::new(gtk::Orientation::Vertical,12);group.add(&body);
    let word=gtk::Label::new(None);word.add_css_class("title-2");word.set_wrap(true);word.set_xalign(0.0);body.append(&word);
    let message=gtk::Label::new(None);message.set_xalign(0.0);message.set_wrap(true);body.append(&message);
    let suggestions=gtk::Box::new(gtk::Orientation::Vertical,4);body.append(&suggestions);
    let replacement=gtk::Entry::new();replacement.set_placeholder_text(Some(wordcraft_gnome::t!("Replacement")));accessible_name(&replacement,wordcraft_gnome::t!("Replacement"));body.append(&replacement);
    let actions=gtk::Box::new(gtk::Orientation::Horizontal,8);let change=gtk::Button::with_label(wordcraft_gnome::t!("Change"));change.add_css_class("suggested-action");let next=gtk::Button::with_label(wordcraft_gnome::t!("Skip"));actions.append(&next);actions.append(&change);body.append(&actions);
    let ignore_all=gtk::Button::with_label(wordcraft_gnome::t!("Ignore All"));let add=gtk::Button::with_label(wordcraft_gnome::t!("Add to Dictionary"));body.append(&ignore_all);body.append(&add);
    toolbar.set_content(Some(&page));window.set_content(Some(&toolbar));
    let s=Rc::new(SpellReview{ui:Rc::downgrade(ui),window:window.clone(),word,message,replacement,suggestions,change,next,ignore_all,add,index:Cell::new(0),issues:RefCell::new(Vec::new()),identity:Cell::new(None),source:RefCell::new(None)});
    {let weak=Rc::downgrade(&s);s.change.connect_clicked(move|_|{if let Some(s)=weak.upgrade(){s.replace();}});}
    {let weak=Rc::downgrade(&s);s.next.connect_clicked(move|_|{if let Some(s)=weak.upgrade(){s.index.set(s.index.get()+1);s.scan();}});}
    for (button,add_word) in [(&s.ignore_all,false),(&s.add,true)]{let weak=Rc::downgrade(&s);button.connect_clicked(move|_|{let Some(s)=weak.upgrade()else{return;};let Some(ui)=s.ui.upgrade()else{return;};if document_identity(&ui)!=s.identity.get(){s.scan();return;}if let Some(i)=s.item(){let result={let mut v=ui.current.borrow_mut();let d=v.as_mut().unwrap();if add_word{proofing::add(&mut d.session,&i)}else{proofing::ignore(&mut d.session,&i)}};if let Err(e)=result{s.message.set_text(&e);return;}refresh_all_proofing(&ui);s.scan();}});}
    {let weak=Rc::downgrade(ui);window.connect_close_request(move|_|{if let Some(ui)=weak.upgrade(){if let Some(s)=ui.spelling.borrow_mut().take(){if let Some(id)=s.source.borrow_mut().take(){id.remove();}}}glib::Propagation::Proceed});}
    ui.spelling.replace(Some(s.clone()));s.scan();window.present();
}

const TEXT_LANGUAGES:&[(&str,&str)]=&[("en-US","English (United States)"),("en-GB","English (United Kingdom)"),("es-ES","Español (España)"),("es-MX","Español (México)"),("es-AR","Español (Argentina)"),("fr-FR","Français"),("de-DE","Deutsch"),("it-IT","Italiano"),("pt-BR","Português (Brasil)"),("pl-PL","Polski"),("ru-RU","Русский"),("ca-ES","Català"),("eu-ES","Euskara"),("gl-ES","Galego")];
struct LanguageDialog{window:adw::Window,tag:gtk::Entry,no_proof:gtk::Switch,scope:adw::ComboRow,apply:gtk::Button}
fn show_text_language(ui:&Rc<Ui>,parent:&ApplicationWindow)->LanguageDialog{
    let identity=document_identity(ui);let state={let mut v=ui.current.borrow_mut();proofing::state(&mut v.as_mut().unwrap().session)};
    let window=adw::Window::builder().title(wordcraft_gnome::t!("Text Language")).transient_for(parent).modal(true).default_width(460).default_height(460).build();let toolbar=ToolbarView::new();let header=HeaderBar::new();header.set_show_start_title_buttons(false);header.set_show_end_title_buttons(false);
    let cancel=gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));header.pack_start(&cancel);let apply=gtk::Button::with_label(wordcraft_gnome::t!("Apply"));apply.add_css_class("suggested-action");header.pack_end(&apply);toolbar.add_top_bar(&header);
    let page=adw::PreferencesPage::new();let group=adw::PreferencesGroup::new();page.add(&group);
    let lang=option_combo(&group,wordcraft_gnome::t!("Language"),&TEXT_LANGUAGES.iter().map(|(_,name)|*name).chain(std::iter::once(wordcraft_gnome::t!("Other Language"))).collect::<Vec<_>>());
    let current=state["lang"].as_str().unwrap_or("en-US");let i=TEXT_LANGUAGES.iter().position(|(tag,_)|tag.eq_ignore_ascii_case(current)).unwrap_or(TEXT_LANGUAGES.len());lang.set_selected(i as u32);
    let tag=gtk::Entry::new();tag.set_text(current);tag.set_placeholder_text(Some("es-ES"));accessible_name(&tag,wordcraft_gnome::t!("Language Tag"));group.add(&tag);
    let hint=gtk::Label::new(None);hint.set_wrap(true);hint.set_xalign(0.0);group.add(&hint);
    let no_proof=option_switch(&group,wordcraft_gnome::t!("Do Not Check Spelling or Grammar"));no_proof.set_active(state["noProof"].as_bool().unwrap_or(false));
    let scope=option_combo(&group,wordcraft_gnome::t!("Apply To"),&[wordcraft_gnome::t!("Selection or Typing Position"),wordcraft_gnome::t!("Whole Document")]);
    let future=option_switch(&group,wordcraft_gnome::t!("Use for New Documents"));
    let message=gtk::Label::new(None);message.add_css_class("error");message.set_wrap(true);group.add(&message);
    {let tag=tag.downgrade();lang.connect_selected_notify(move|r|{if let(Some(tag),Some((value,_)))=(tag.upgrade(),TEXT_LANGUAGES.get(r.selected()as usize)){tag.set_text(value);}});}
    let update=move|tag:&gtk::Entry|{let value=tag.text();hint.set_text(if wordcraft_engine::proof::has_dictionary(Some(&value)){wordcraft_gnome::t!("Spelling tools are available for this language.")}else{wordcraft_gnome::t!("No dictionary is installed for this language. Its spelling cannot be checked.")});};update(&tag);tag.connect_changed(update);
    {let w=window.downgrade();cancel.connect_clicked(move|_|{if let Some(w)=w.upgrade(){w.close();}});}
    {let weak=Rc::downgrade(ui);let w=window.downgrade();let tag=tag.clone();let no_proof=no_proof.clone();let scope=scope.clone();apply.connect_clicked(move|_|{
        let(Some(ui),Some(w))=(weak.upgrade(),w.upgrade())else{return;};if ui.closed.get()||document_identity(&ui)!=identity{message.set_text(wordcraft_gnome::t!("The text has changed. Check it again."));return;}
        let result={let mut v=ui.current.borrow_mut();proofing::language(&mut v.as_mut().unwrap().session,&tag.text(),no_proof.is_active(),scope.selected()==1)};
        match result{Ok(changed)=>{if changed{after_content_change(&ui);}else{refresh_selection(&ui);}if future.is_active(){if let Err(e)=wordcraft_gnome::preferences::set_default_language(&tag.text()){message.set_text(&e);return;}}w.close();refresh_all_proofing(&ui);ui.document_box.grab_focus();},Err(e)=>message.set_text(&e)}
    });}
    toolbar.set_content(Some(&page));window.set_content(Some(&toolbar));window.present();LanguageDialog{window,tag,no_proof,scope,apply}
}

fn proofing_ui_test(ui:&Rc<Ui>,parent:&ApplicationWindow,app:&Application){
    opened(ui,Session::new(wordcraft_doc::Document::from_text("El qqqgtkwordzz examen.")),"Proofing");
    let language=show_text_language(ui,parent);language.tag.set_text("es-ES");language.scope.set_selected(1);language.no_proof.set_active(false);language.apply.emit_clicked();assert!(!language.window.is_visible());
    show_spelling(ui,parent);let weak=Rc::downgrade(ui);let a=app.clone();let w=parent.clone();let attempts=Rc::new(Cell::new(0));
    glib::timeout_add_local(std::time::Duration::from_millis(100),move||{
        let ui=weak.upgrade().unwrap();attempts.set(attempts.get()+1);assert!(attempts.get()<200,"proofing scan did not finish");
        let s=ui.spelling.borrow().as_ref().unwrap().clone();if s.source.borrow().is_some(){return glib::ControlFlow::Continue;}
        assert_eq!(s.item().unwrap().text,"qqqgtkwordzz");assert!(s.add.is_sensitive());
        let context=show_document_context(&ui,&w,120,180,DocumentContext::Text);assert!(context_model_row(&context,wordcraft_gnome::t!("Add to Dictionary")).is_some());context.popdown();
        s.replacement.set_text("profesor");s.change.emit_clicked();assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc.plain_text(wordcraft_doc::StoryRef::Body),"El profesor examen.");
        let window=s.window.clone();window.close();assert!(ui.spelling.borrow().is_none());eprintln!("PROOFING_UI_OK");a.quit();glib::ControlFlow::Break
    });
}
