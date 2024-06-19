use criterion::Criterion;

pub fn criterion() -> Criterion {
    let c = Criterion::default().configure_from_args();
    #[cfg(unix)]
    let c = {
        use pprof::criterion::{Output, PProfProfiler};
        c.with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None)))
    };
    c
}
