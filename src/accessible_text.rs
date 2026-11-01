//! Read-only character/position index for assistive technology. WordCraft owns
//! all editing, selection, undo and pagination; there is no second text buffer.
use std::{collections::BTreeMap, sync::Arc};
use unicode_segmentation::UnicodeSegmentation;
use wordcraft_doc::{Document,Path,Pos,StoryRef};
use wordcraft_engine::Session;

#[derive(Clone,Debug)]
struct Paragraph {
    path:Path,
    text:Arc<str>,
    start:u32,
    len:u32,
}

#[derive(Clone,Debug)]
pub struct TextIndex {
    story:StoryRef,
    text:String,
    // UTF-8 byte boundaries for AT-SPI's Unicode character offsets, including EOF.
    bytes:Vec<usize>,
    paragraphs:Vec<Paragraph>,
    paths:BTreeMap<Path,usize>,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Change {pub start:u32,pub old_end:u32,pub new_end:u32}

impl Default for TextIndex {
    fn default()->Self{Self{story:StoryRef::Body,text:String::new(),bytes:vec![0],paragraphs:Vec::new(),paths:BTreeMap::new()}}
}

impl TextIndex {
    pub fn new(doc:&Document,story:StoryRef)->Self {
        let mut out=Self{story,..Default::default()};
        let mut count=0;
        for path in doc.para_paths(story) {
            let Some(p)=doc.para(story,&path)else{continue;};
            if !out.paragraphs.is_empty(){out.text.push('\n');count+=1;}
            let start=count;
            for ch in p.text.chars(){
                // Preserve character positions while avoiding C-string truncation
                // and representing manual line/page breaks as readable newlines.
                out.text.push(match ch{'\0'=>'\u{fffd}','\u{000b}'|'\u{000c}'|'\u{000e}'=>'\n',_=>ch});
                count+=1;
            }
            out.paths.insert(path.clone(),out.paragraphs.len());
            out.paragraphs.push(Paragraph{path,text:Arc::from(p.text.as_str()),start,len:count-start});
        }
        out.bytes=out.text.char_indices().map(|(i,_)|i).chain(std::iter::once(out.text.len())).collect();
        out
    }
    pub fn text(&self)->&str{&self.text}
    pub fn len(&self)->u32{self.bytes.len().saturating_sub(1)as u32}
    pub fn is_empty(&self)->bool{self.len()==0}
    pub fn contents(&self,start:u32,end:u32)->Option<&str>{
        let end=end.min(self.len());
        if start>end{return None;}
        self.text.get(*self.bytes.get(start as usize)?..*self.bytes.get(end as usize)?)
    }
    pub fn offset(&self,pos:&Pos)->Option<u32>{
        if pos.story!=self.story{return None;}
        let p=self.paragraphs.get(*self.paths.get(&pos.path)?)?;
        Some(p.start+p.text.get(..pos.off)?.chars().count()as u32)
    }
    pub fn position(&self,offset:u32)->Option<Pos>{
        if offset>self.len(){return None;}
        let index=self.paragraphs.partition_point(|p|p.start<=offset).saturating_sub(1);
        let p=self.paragraphs.get(index)?;
        let local=(offset-p.start).min(p.len)as usize;
        let off=p.text.char_indices().nth(local).map(|(i,_)|i).unwrap_or(p.text.len());
        Some(Pos{story:self.story,path:p.path.clone(),off})
    }
    pub fn paragraph(&self,offset:u32)->Option<(u32,u32)>{
        if offset>self.len(){return None;}
        let i=self.paragraphs.partition_point(|p|p.start<=offset).saturating_sub(1);
        let p=self.paragraphs.get(i)?;
        Some((p.start,self.paragraphs.get(i+1).map_or(self.len(),|p|p.start)))
    }
    pub fn character(&self,offset:u32)->Option<(u32,u32)>{
        if offset>self.len(){None}else{Some((offset,offset.saturating_add(1).min(self.len())))}
    }
    pub fn word(&self,offset:u32)->Option<(u32,u32)>{
        let (start,end)=self.paragraph(offset)?;
        let text=self.contents(start,end)?;
        let mut starts=vec![0];
        let base=self.bytes[start as usize];
        starts.extend(text.unicode_word_indices().filter_map(|(i,_)|self.bytes.binary_search(&(base+i)).ok().map(|n|n as u32-start)));
        starts.sort_unstable();starts.dedup();
        let i=starts.partition_point(|n|*n<=offset-start).saturating_sub(1);
        Some((start+starts[i],starts.get(i+1).map_or(end,|n|start+n)))
    }
    pub fn sentence(&self,offset:u32)->Option<(u32,u32)>{
        let(start,end)=self.paragraph(offset)?;
        let text=self.contents(start,end)?;
        let mut from=start;
        for (_,sentence) in text.split_sentence_bound_indices(){
            let to=from+sentence.chars().count()as u32;
            if offset<to{return Some((from,to));}from=to;
        }
        Some((from,end))
    }
    /// Visible soft-wrapped lines come from the same public layout API as caret
    /// navigation. The final line includes its paragraph separator, if present.
    pub fn line(&self,session:&mut Session,offset:u32)->Option<(u32,u32)>{
        let pos=self.position(offset)?;
        let(a,b,last)=session.layout().line_span(&pos,session.page_hint)?;
        let start=self.offset(&a)?;let mut end=self.offset(&b)?;
        if last&&end<self.len(){end+=1;}
        Some((start,end))
    }
    pub fn selection(&self,session:&Session)->Option<(u32,u32)>{
        let ends=session.sel_ends();
        let a=self.offset(&ends.anchor)?;let b=self.offset(&ends.focus)?;
        (a!=b).then_some((a.min(b),a.max(b)))
    }
    /// Smallest single replacement in character units. Deletion is notified
    /// while the old snapshot is readable, insertion after installing the new one.
    pub fn change(&self,next:&Self)->Option<Change>{
        if self.text==next.text{return None;}
        let prefix=self.text.chars().zip(next.text.chars()).take_while(|(a,b)|a==b).count()as u32;
        let suffix=self.text.chars().rev().take((self.len()-prefix)as usize)
            .zip(next.text.chars().rev().take((next.len()-prefix)as usize)).take_while(|(a,b)|a==b).count()as u32;
        Some(Change{start:prefix,old_end:self.len()-suffix,new_end:next.len()-suffix})
    }
}
