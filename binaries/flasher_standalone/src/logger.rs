use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

pub fn init() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .with_span_events(FmtSpan::CLOSE)
        .with_line_number(true)
        .init();
}
