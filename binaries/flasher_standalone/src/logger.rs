use tracing_subscriber::filter::EnvFilter;

pub fn init() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
