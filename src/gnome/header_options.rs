use super::*;

use wordcraft_gnome::page_options::{self,Options as HeaderOptions,Links};
pub(super) struct HeaderControls {
    pub(super) section:adw::ComboRow,pub(super) first:gtk::Switch,pub(super) odd_even:gtk::Switch,pub(super) header:gtk::SpinButton,pub(super) footer:gtk::SpinButton,
    pub(super) format:adw::ComboRow,pub(super) restart:adw::ComboRow,pub(super) start:gtk::SpinButton,pub(super) headers:adw::ComboRow,pub(super) footers:adw::ComboRow,
    pub(super) snapshots:Vec<HeaderOptions>,pub(super) links_state:Vec<(u8,u8)>,
}
pub(super) fn option_combo(group:&adw::PreferencesGroup,title:&str,labels:&[&str])->adw::ComboRow {
    let row=adw::ComboRow::new();row.set_title(title);row.set_model(Some(&gtk::StringList::new(labels)));group.add(&row);row
}
pub(super) fn option_switch(group:&adw::PreferencesGroup,title:&str)->gtk::Switch {
    let row=adw::ActionRow::new();row.set_title(title);let b=gtk::Switch::new();b.set_valign(gtk::Align::Center);row.add_suffix(&b);row.set_activatable_widget(Some(&b));accessible_name(&b,title);group.add(&row);b
}
impl HeaderControls {
    pub(super) fn new(ui:&Rc<Ui>,page:&adw::PreferencesPage,initial:usize)->Rc<Self>{
        let snapshots={let v=ui.current.borrow();let s=&v.as_ref().unwrap().session;(0..s.doc.sections().len()).map(|i|HeaderOptions::current(s,i)).collect::<Vec<_>>()};
        let links_state={let v=ui.current.borrow();let s=&v.as_ref().unwrap().session;let mask=|h:wordcraft_doc::section::HeaderSet|u8::from(h.default.is_none())+u8::from(h.first.is_none())+u8::from(h.even.is_none());s.doc.sections().iter().map(|(_,p)|(mask(p.headers),mask(p.footers))).collect::<Vec<_>>()};
        let target=adw::PreferencesGroup::new();target.set_description(Some(wordcraft_gnome::t!("First-page settings, numbering and distances apply to the selected section. Odd/even headers apply to the whole document.")));page.add(&target);
        let labels=(0..snapshots.len()).map(|i|format!("{} {}",wordcraft_gnome::t!("Section"),i+1)).collect::<Vec<_>>();
        let section=option_combo(&target,wordcraft_gnome::t!("Section"),&labels.iter().map(String::as_str).collect::<Vec<_>>());section.set_selected(initial as u32);
        let variants=adw::PreferencesGroup::new();variants.set_title(wordcraft_gnome::t!("Headers and Footers"));page.add(&variants);
        let first=option_switch(&variants,wordcraft_gnome::t!("Different First Page"));let odd_even=option_switch(&variants,wordcraft_gnome::t!("Different Odd and Even Pages"));odd_even.set_active(snapshots[initial].odd_even);
        let header=properties_number(&variants,wordcraft_gnome::t!("Header from Top (mm)"),snapshots[initial].header);
        let footer=properties_number(&variants,wordcraft_gnome::t!("Footer from Bottom (mm)"),snapshots[initial].footer);
        let numbers=adw::PreferencesGroup::new();numbers.set_title(wordcraft_gnome::t!("Page Numbering"));page.add(&numbers);
        let format=option_combo(&numbers,wordcraft_gnome::t!("Number Format"),&["1, 2, 3","i, ii, iii","I, II, III","a, b, c","A, B, C",wordcraft_gnome::t!("Existing Format")]);
        let restart=option_combo(&numbers,wordcraft_gnome::t!("Numbering"),&[wordcraft_gnome::t!("Continue from Previous Section"),wordcraft_gnome::t!("Start At")]);
        let row=adw::ActionRow::new();row.set_title(wordcraft_gnome::t!("Starting Page Number"));let start=gtk::SpinButton::with_range(0.0,100000.0,1.0);start.set_valign(gtk::Align::Center);row.add_suffix(&start);numbers.add(&row);accessible_name(&start,wordcraft_gnome::t!("Starting Page Number"));
        let links=adw::PreferencesGroup::new();links.set_title(wordcraft_gnome::t!("Section Links"));links.set_description(Some(wordcraft_gnome::t!("Independent copies keep the current content. Linking uses the previous section; existing separate content remains available through Undo.")));page.add(&links);
        let choices=[wordcraft_gnome::t!("Keep Existing Links"),wordcraft_gnome::t!("Link to Previous Section"),wordcraft_gnome::t!("Make Independent Copies")];
        let headers=option_combo(&links,wordcraft_gnome::t!("Headers"),&choices);let footers=option_combo(&links,wordcraft_gnome::t!("Footers"),&choices);
        let out=Rc::new(Self{section,first,odd_even,header,footer,format,restart,start,headers,footers,snapshots,links_state});out.load(initial);
        {let weak=Rc::downgrade(&out);out.section.connect_selected_notify(move|r|{if let Some(s)=weak.upgrade(){s.load(r.selected()as usize);}});}
        {let start=out.start.downgrade();out.restart.connect_selected_notify(move|r|{if let Some(s)=start.upgrade(){s.set_sensitive(r.selected()==1);}});}
        out
    }
    pub(super) fn load(&self,i:usize){
        let Some(o)=self.snapshots.get(i)else{return;};
        let label=|n|match n{3=>wordcraft_gnome::t!("Linked to Previous Section"),0=>wordcraft_gnome::t!("Independent"),_=>wordcraft_gnome::t!("Some Variants Are Linked")};
        self.headers.set_subtitle(if i==0{wordcraft_gnome::t!("First Section")}else{label(self.links_state[i].0)});self.footers.set_subtitle(if i==0{wordcraft_gnome::t!("First Section")}else{label(self.links_state[i].1)});self.first.set_active(o.first);self.header.set_value(o.header as f64/PT_PER_MM);self.footer.set_value(o.footer as f64/PT_PER_MM);
        self.format.set_selected(match o.format.ooxml(){"decimal"=>0,"lowerRoman"=>1,"upperRoman"=>2,"lowerLetter"=>3,"upperLetter"=>4,_=>5});self.restart.set_selected(if o.start.is_some(){1}else{0});self.start.set_value(o.start.unwrap_or(1)as f64);self.start.set_sensitive(o.start.is_some());self.headers.set_selected(0);self.footers.set_selected(0);self.headers.set_sensitive(i>0);self.footers.set_sensitive(i>0);
    }
    pub(super) fn values(&self)->HeaderOptions{
        self.header.update();self.footer.update();self.start.update();let section=self.section.selected()as usize;let mut out=self.snapshots[section].clone();
        out.first=self.first.is_active();out.odd_even=self.odd_even.is_active();out.header=(self.header.value()*PT_PER_MM)as f32;out.footer=(self.footer.value()*PT_PER_MM)as f32;
        if let Some(fmt)=["decimal","lowerRoman","upperRoman","lowerLetter","upperLetter"].get(self.format.selected()as usize){out.format=wordcraft_doc::section::NumFormat::from_ooxml(fmt);}
        out.start=if self.restart.selected()==0{None}else{Some(self.start.value_as_int()as u32)};
        let link=|i|match i{1=>Links::Previous,2=>Links::Independent,_=>Links::Keep};out.headers=link(self.headers.selected());out.footers=link(self.footers.selected());out
    }
}
pub(super) fn apply_properties(s:&mut Session,setup:PageSetup,whole:bool,section:usize,headers:&HeaderOptions)->Result<bool,String>{
    let snap=s.edit_snapshot();let pending=s.pending.clone();let mark=s.undo_mark();
    let result=(||{let a=document_properties::apply_at(s,setup,whole,section)?;let b=page_options::apply(s,headers)?;Ok::<_,String>(a||b)})();
    match result{Ok(changed)=>{if changed{s.merge_undo_since(mark);}Ok(changed)},Err(e)=>{s.restore(snap);s.pending=pending;Err(e)}}
}


