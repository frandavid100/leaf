// Opt-in synthetic GUI diagnostic. No timers, logging or document contents are
// collected in normal operation. Inclusive spans must not be summed together.
#[derive(Default)]
struct PerfMetrics { spans:std::collections::BTreeMap<&'static str,(u64,u128)>, counts:std::collections::BTreeMap<&'static str,u64> }
thread_local! {static PERF_METRICS:RefCell<PerfMetrics>=RefCell::new(PerfMetrics::default());}
fn perf_enabled()->bool{static ENABLED:std::sync::OnceLock<bool>=std::sync::OnceLock::new();*ENABLED.get_or_init(||std::env::args().any(|a|a=="--performance-ui-test"||a=="--host-cache-ui-test"))}
struct PerfSpan(Option<(&'static str,std::time::Instant)>);
fn perf_scope(name:&'static str)->PerfSpan{PerfSpan(perf_enabled().then(||(name,std::time::Instant::now())))}
impl Drop for PerfSpan {fn drop(&mut self){if let Some((name,start))=self.0{let ns=start.elapsed().as_nanos();PERF_METRICS.with(|m|{let mut m=m.borrow_mut();let entry=m.spans.entry(name).or_default();entry.0+=1;entry.1+=ns;});}}}
fn perf_count(name:&'static str,n:u64){if perf_enabled(){PERF_METRICS.with(|m|*m.borrow_mut().counts.entry(name).or_default()+=n);}}
fn perf_pump(){let context=glib::MainContext::default();let start=std::time::Instant::now();while context.pending()&&start.elapsed().as_millis()<400{context.iteration(false);}}
fn perf_phase(name:&str,work:impl FnOnce())->serde_json::Value{
    PERF_METRICS.with(|m|*m.borrow_mut()=PerfMetrics::default());
    let start=std::time::Instant::now();work();let wall=start.elapsed().as_secs_f64()*1000.0;
    PERF_METRICS.with(|m|{let m=m.borrow();let spans:std::collections::BTreeMap<_,_>=m.spans.iter().map(|(k,(calls,ns))|(*k,serde_json::json!({"calls":calls,"ms":*ns as f64/1e6}))).collect();serde_json::json!({"phase":name,"wall_ms":wall,"inclusive_spans":spans,"counts":m.counts,"rss_kib":lifecycle_rss()})})
}
fn performance_ui_test(ui:&Rc<Ui>,app:&Application){
    ui.document_box.settings().set_property("gtk-enable-animations",false);
    wordcraft_gnome::preferences::set_autosave_mode(0).unwrap();
    let paragraphs=std::env::var("LEAF_PERF_PARAGRAPHS").ok().and_then(|s|s.parse::<usize>().ok()).unwrap_or(600);
    let text=(0..paragraphs).map(|i|format!("Paragraph {i}: {} {}",if i%25==0{"needle"}else{"ordinary"},"Café, selección, documents and useful writing. ".repeat(4))).collect::<Vec<_>>().join("\n");
    let doc=wordcraft_doc::Document::from_text(&text);
    let folder=std::env::temp_dir().join(format!("leaf-perf-{}",std::process::id()));std::fs::create_dir_all(&folder).unwrap();
    for ext in ["docx","odt"]{std::fs::write(folder.join(format!("fixture.{ext}")),wordcraft_engine::io::save_bytes(&format!("fixture.{ext}"),&doc).unwrap()).unwrap();}
    let mut phases=Vec::new();let mut loaded=None;
    for ext in ["docx","odt"]{phases.push(perf_phase(&format!("import_{ext}"),||{loaded=Some(open_document(&folder.join(format!("fixture.{ext}"))).unwrap());}));}
    let mut session=loaded.take().unwrap();let mut pages=0;
    phases.push(perf_phase("first_layout",||{let _p=perf_scope("boundary.first_layout");pages=session.layout().pages.len();}));
    assert!(pages>2);let original=session.doc.clone();
    phases.push(perf_phase("open_widgets_and_initial_rasters",||opened(ui,session,"Synthetic performance fixture")));
    perf_pump();refresh_pages(ui);refresh_thumbnails(ui);perf_pump();
    // Warm cache refreshes expose host traversal work without raster costs.
    phases.push(perf_phase("thumbnail_cache_hits_80",||{for _ in 0..80{refresh_thumbnails(ui);}}));
    phases.push(perf_phase("cursor_right_80",||{for _ in 0..80{selected_command(ui,"right",false);}}));
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc,original);
    // Clear a cross-page selection and move between pages; old highlights must disappear.
    selected_command(ui,"select-all",false);selected_command(ui,"docEnd",false);selected_command(ui,"docStart",false);perf_pump();
    phases.push(perf_phase("scroll_pages",||{
        for fraction in [0.2,0.8,0.4,1.0,0.0]{let adj=ui.scroll.vadjustment();adj.set_value((adj.upper()-adj.page_size()).max(0.0)*fraction);refresh_pages(ui);perf_pump();}
    }));
    phases.push(perf_phase("scroll_thumbnails",||{
        for fraction in [0.2,0.8,0.4,1.0,0.0]{let adj=ui.thumbnails_scroll.vadjustment();adj.set_value((adj.upper()-adj.page_size()).max(0.0)*fraction);refresh_thumbnails(ui);perf_pump();}
    }));
    selected_command(ui,"docStart",false);perf_pump();
    phases.push(perf_phase("edit_8_and_undo",||{for _ in 0..8{apply_user_edit(ui,Edit::Insert("x".into()));}for _ in 0..8{if ui.current.borrow().as_ref().unwrap().session.doc==original{break;}apply_user_edit(ui,Edit::Undo);}}));
    // Use WordCraft history grouping; undo only as many entries as this typing created.
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc,original,"typing and undo must preserve canonical content");
    let search=ui.search.borrow().as_ref().unwrap().clone();search.show(false);search.query.set_text("needle");
    phases.push(perf_phase("active_search",||search.refresh()));assert_eq!(search.hits.borrow().len(),paragraphs.div_ceil(25));
    search.toggle.set_active(false);perf_pump();
    phases.push(perf_phase("inactive_search_refresh_4",||{for _ in 0..4{search.refresh();}}));
    search.show(false);search.refresh();assert_eq!(search.hits.borrow().len(),paragraphs.div_ceil(25));search.toggle.set_active(false);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc,original);
    let (page_rasters,thumb_rasters)={let d=ui.current.borrow();let d=d.as_ref().unwrap();(d.painted_pages.len(),d.painted_thumbs.len())};
    let report=serde_json::json!({"schema":1,"paragraphs":paragraphs,"pages":pages,"scale":ui.document_box.scale_factor(),"renderer":"cairo","phases":phases,"canonical_document_unchanged":true,"cached_page_rasters":page_rasters,"cached_thumbnail_rasters":thumb_rasters,"notes":"Inclusive spans overlap. Boundary calls include Leaf adapters and WordCraft; they are not exclusive engine CPU profiles. Synthetic body text only; no general document-performance claim."});
    std::fs::write(std::env::var("LEAF_PERF_REPORT").unwrap_or_else(|_|"leaf-performance.json".into()),serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    std::fs::remove_dir_all(folder).unwrap();eprintln!("LEAF_PERFORMANCE_OK paragraphs={paragraphs} pages={pages}");app.quit();
}
