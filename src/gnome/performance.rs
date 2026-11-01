use super::*;

// Opt-in synthetic GUI diagnostic. No timers, logging or document contents are
// collected in normal operation. Inclusive spans must not be summed together.
#[derive(Default)]
pub(super) struct PerfMetrics { pub(super) spans:std::collections::BTreeMap<&'static str,(u64,u128)>, pub(super) counts:std::collections::BTreeMap<&'static str,u64> }
thread_local! {static PERF_METRICS:RefCell<PerfMetrics>=RefCell::new(PerfMetrics::default());}
pub(super) fn perf_enabled()->bool{static ENABLED:std::sync::OnceLock<bool>=std::sync::OnceLock::new();*ENABLED.get_or_init(||std::env::args().any(|a|a=="--performance-ui-test"||a=="--host-cache-ui-test"))}
pub(super) struct PerfSpan(Option<(&'static str,std::time::Instant)>);
pub(super) fn perf_scope(name:&'static str)->PerfSpan{PerfSpan(perf_enabled().then(||(name,std::time::Instant::now())))}
impl Drop for PerfSpan {fn drop(&mut self){if let Some((name,start))=self.0{let ns=start.elapsed().as_nanos();PERF_METRICS.with(|m|{let mut m=m.borrow_mut();let entry=m.spans.entry(name).or_default();entry.0+=1;entry.1+=ns;});}}}
pub(super) fn perf_count(name:&'static str,n:u64){if perf_enabled(){PERF_METRICS.with(|m|*m.borrow_mut().counts.entry(name).or_default()+=n);}}
pub(super) fn perf_pump(){let context=glib::MainContext::default();let start=std::time::Instant::now();while context.pending()&&start.elapsed().as_millis()<400{context.iteration(false);}}
pub(super) fn perf_phase(name:&str,work:impl FnOnce())->serde_json::Value{
    PERF_METRICS.with(|m|*m.borrow_mut()=PerfMetrics::default());
    let start=std::time::Instant::now();work();let wall=start.elapsed().as_secs_f64()*1000.0;
    PERF_METRICS.with(|m|{let m=m.borrow();let spans:std::collections::BTreeMap<_,_>=m.spans.iter().map(|(k,(calls,ns))|(*k,serde_json::json!({"calls":calls,"ms":*ns as f64/1e6}))).collect();serde_json::json!({"phase":name,"wall_ms":wall,"inclusive_spans":spans,"counts":m.counts,"rss_kib":lifecycle_rss()})})
}

