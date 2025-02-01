/// Init logger
pub(crate) fn init_logger(level: tracing::Level) {
    use tracing_subscriber::FmtSubscriber;

    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .compact()
            .with_max_level(level)
            .without_time()
            .finish(),
    )
    .expect("Fail to set global default subscriber");
}
