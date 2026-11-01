//! Optional cross-application audit worker. No GUI, fixture downloads or release gate.
use std::{fs, path::Path, sync::Arc};
use serde_json::{Value, json};
use wordcraft_doc::{Block, Document, Paragraph, PartKind, Pos, StoryRef, section::{NumFormat, SectionProps}};
use wordcraft_engine::Session;
use wordcraft_gnome::{header_footer::{self, NumberPosition, Region}, proofing, selection};

fn fixture(sections: bool) -> Session {
    let count = if sections { 3 } else { 1 };
    let lines = (0..count).flat_map(|section| (0..65).map(move |i| {
        if i == 0 { format!("SECTION{} Leaf audit BODY_SENTINEL", section+1) }
        else if i == 1 { "English colour and Spanish oración. UNCHECKED_TOKEN".into() }
        else { format!("Section {} paragraph {i}: ordinary text for independent page and field checks.",section+1) }
    })).collect::<Vec<_>>();
    let mut d = Document::from_text(&lines.join("\n"));
    let mut props = Vec::new();
    for section in 0..count {
        let mut p = SectionProps::default();
        p.title_page = true;
        p.header = 21.0; p.footer = 27.0;
        p.page_num_format = if section == 0 { NumFormat::Decimal } else { NumFormat::UpperRoman };
        p.page_num_start = match section { 0 => Some(1), 1 => Some(7), _ => None };
        if section < 2 {
            let mut ids = Vec::new();
            for variant in ["DEFAULT", "EVEN", "FIRST"] {
                ids.push(d.add_part(PartKind::Header, vec![Arc::new(Block::Para(Paragraph::with_text(
                    &format!("HEADER_S{}_{variant}",section+1), Default::default())))]));
            }
            p.headers.default = Some(ids[0]); p.headers.even = Some(ids[1]); p.headers.first = Some(ids[2]);
        }
        props.push(p);
    }
    for (section,p) in props.into_iter().enumerate() {
        if section == count-1 { d.last_section = p; }
        else if let Block::Para(para) = Arc::make_mut(&mut d.body[(section+1)*65-1]) { para.section = Some(Box::new(p)); }
    }
    d.settings.even_odd_headers = true;
    let mut s = Session::new(d);
    proofing::language(&mut s,"en-GB",false,true).unwrap();
    for section in 0..count {
        let index=section*65+1;
        let text=s.doc.para_at(&Pos::body(index,0)).unwrap().text.to_string();
        let from=text.find("Spanish").unwrap();let end=text.find(". UNCHECKED").unwrap();
        selection::select_range(&mut s,&Pos::body(index,from),&Pos::body(index,end)).unwrap();
        proofing::language(&mut s,"es-ES",false,false).unwrap();
        let from=text.find("UNCHECKED_TOKEN").unwrap();
        selection::select_range(&mut s,&Pos::body(index,from),&Pos::body(index,text.len())).unwrap();
        proofing::language(&mut s,"en-GB",true,false).unwrap();
    }
    // Each displayed variant gets real PAGE and NUMPAGES fields via Leaf's adapter.
    let pages=s.layout().pages.len();
    let mut seen=std::collections::HashSet::new();
    for page in 0..pages {
        header_footer::enter(&mut s,page,Region::Footer).unwrap();
        if seen.insert(s.sel.focus.story) {
            s.run("text.insert",&json!({"text":"FOOTER "})).unwrap();
            header_footer::insert_number(&mut s,page,NumberPosition::Current,"center",true).unwrap();
        }
        header_footer::close(&mut s).unwrap();
    }
    s
}

fn snapshot(s:&mut Session)->Value {
    let pages=s.layout().pages.iter().map(|p|json!({"section":p.section,"number":p.number,
        "header":p.header_story.map(|id|s.doc.plain_text(StoryRef::Part(id))),
        "footer":p.footer_story.map(|id|s.doc.plain_text(StoryRef::Part(id)))})).collect::<Vec<_>>();
    let language=s.doc.para_paths(StoryRef::Body).iter().filter_map(|path| {
        let p=s.doc.para(StoryRef::Body,path)?;
        if !p.text.contains("Spanish") {return None;}
        Some(json!({"text":p.text,"probes":["English","Spanish","UNCHECKED_TOKEN"].map(|needle| {
            let offset=p.text.find(needle).unwrap();
            let c=s.doc.styles.resolve_char(p.props.style.as_deref(),p.props_of_char(offset));
            json!({"token":needle,"lang":c.lang,"no_proof":c.no_proof})
        })}))
    }).collect::<Vec<_>>();
    json!({"body_text":s.doc.plain_text(StoryRef::Body),"sections":s.doc.sections().iter().map(|(_,p)|*p).collect::<Vec<_>>(),
        "even_odd":s.doc.settings.even_odd_headers,"pages":pages,"language":language,
        "parts":s.doc.parts.iter().map(|(id,p)|json!({"kind":p.kind,"text":s.doc.plain_text(StoryRef::Part(*id)),
            "fields":p.blocks.iter().filter_map(|b|if let Block::Para(p)=&**b {Some(&p.objects)}else{None}).collect::<Vec<_>>()})).collect::<Vec<_>>()})
}
fn run()->Result<(),Box<dyn std::error::Error>> {
    let args=std::env::args().collect::<Vec<_>>();
    match args[1].as_str() {
        "generate" => {let out=Path::new(&args[2]);fs::create_dir_all(out)?;
            for (name,multi) in [("variants",false),("sections",true)] {for ext in ["docx","odt"] {
                let s=fixture(multi);let path=out.join(format!("{name}.{ext}"));
                fs::write(&path,wordcraft_engine::io::save_bytes(path.to_str().unwrap(),&s.doc)?)?;
            }}
        },
        "inspect" => {let input=Path::new(&args[2]);let out=Path::new(&args[3]);fs::create_dir_all(out)?;
            let mut s=wordcraft_gnome::open_document(input)?;
            fs::write(out.join("state.json"),serde_json::to_vec_pretty(&snapshot(&mut s))?)?;
            let path=out.join(format!("leaf-saved.{}",input.extension().unwrap().to_str().unwrap()));
            fs::write(&path,wordcraft_engine::io::save_bytes(path.to_str().unwrap(),&s.doc)?)?;
        },
        _=>return Err("use generate DIRECTORY or inspect FILE DIRECTORY".into())
    }
    println!("INTEROP_WORKER_OK");Ok(())
}
fn main(){if let Err(e)=run(){eprintln!("{e}");std::process::exit(1);}}
