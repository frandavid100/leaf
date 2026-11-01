//! GNU gettext catalogs, shared by the GTK host and its command adapters.
//! Catalog strings are copied once into a process-owned immutable table; widget
//! constructors can safely borrow them without retaining windows or leaking strings.
use std::{collections::HashMap, ffi::{CStr,CString,c_char,c_int}, sync::OnceLock};
include!("../po/messages.rs");
const DOMAIN: &CStr = c"leaf";
unsafe extern "C" {
    fn setlocale(category:c_int,locale:*const c_char)->*mut c_char;
    fn bindtextdomain(domain:*const c_char,directory:*const c_char)->*mut c_char;
    fn bind_textdomain_codeset(domain:*const c_char,codeset:*const c_char)->*mut c_char;
    fn dgettext(domain:*const c_char,message:*const c_char)->*mut c_char;
    fn dngettext(domain:*const c_char,singular:*const c_char,plural:*const c_char,n:std::ffi::c_ulong)->*mut c_char;
}
static CATALOG:OnceLock<HashMap<&'static str,String>>=OnceLock::new();
fn directory()->std::path::PathBuf {
    if let Some(p)=std::env::var_os("LEAF_LOCALEDIR"){return p.into();}
    if let Ok(exe)=std::env::current_exe(){if let Some(prefix)=exe.parent().and_then(|p|p.parent()){
        let p=prefix.join("share/locale");if p.is_dir(){return p;}
    }}
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("po/locale")
}
pub fn init(){
    static LOCALE:std::sync::Once=std::sync::Once::new();
    LOCALE.call_once(||{unsafe{setlocale(6,c"".as_ptr());}});
    catalog();
}
fn catalog()->&'static HashMap<&'static str,String>{CATALOG.get_or_init(||{
    let dir=CString::new(directory().to_string_lossy().as_bytes()).expect("locale path contains NUL");
    // GNU/Linux LC_ALL. Configure once, before GTK or worker threads start.
    unsafe {bindtextdomain(DOMAIN.as_ptr(),dir.as_ptr());bind_textdomain_codeset(DOMAIN.as_ptr(),c"UTF-8".as_ptr());}
    MESSAGES.iter().map(|&id|{
        let c=CString::new(id).expect("message contains NUL");
        // gettext owns this pointer (or returns our input); copy before c drops.
        let value=unsafe{let p=dgettext(DOMAIN.as_ptr(),c.as_ptr());if p.is_null(){id.to_owned()}else{CStr::from_ptr(p).to_string_lossy().into_owned()}};
        (id,value)
    }).collect()
})}
pub fn tr(id:&'static str)->&'static str {catalog().get(id).map(String::as_str).unwrap_or(id)}
/// Native gettext evaluates each language's Plural-Forms expression.
pub fn plural(one:&str,many:&str,n:u64)->String {
    init();let one=CString::new(one).unwrap();let many=CString::new(many).unwrap();
    unsafe {let p=dngettext(DOMAIN.as_ptr(),one.as_ptr(),many.as_ptr(),n as std::ffi::c_ulong);if p.is_null(){String::new()}else{CStr::from_ptr(p).to_string_lossy().into_owned()}}
}
/// Named replacements permit translators to reorder complete placeholders.
/// Values are inserted verbatim, so braces inside document names are safe.
pub fn interpolate(template:&str,values:&[(&str,String)])->String {
    let mut out=String::new();let mut chars=template.chars().peekable();let mut index=0;
    while let Some(ch)=chars.next(){
        if ch=='{' && chars.peek()==Some(&'{'){chars.next();out.push('{');continue;}
        if ch=='}' && chars.peek()==Some(&'}'){chars.next();out.push('}');continue;}
        if ch!='{'{out.push(ch);continue;}
        let mut field=String::new();for c in chars.by_ref(){if c=='}'{break;}field.push(c);}
        let name=field.split(':').next().unwrap_or("");let key=if name.is_empty(){let s=index.to_string();index+=1;s}else{name.to_owned()};
        if let Some((_,v))=values.iter().find(|(k,_)|*k==key){out.push_str(v);}else{out.push('{');out.push_str(&field);out.push('}');}
    }out
}
#[macro_export]macro_rules! t {($message:literal)=>{$crate::i18n::tr($message)};}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn reordered_fields_and_document_braces(){assert_eq!(interpolate("{b}: {a}",&[("a","x{y}".into()),("b","name".into())]),"name: x{y}");}
 #[test]fn positional_and_escaped_braces(){assert_eq!(interpolate("{{{}}} {}",&[("0","one".into()),("1","two".into())]),"{one} two");}
}
