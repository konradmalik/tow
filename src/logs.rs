pub fn init() {
    stderrlog::new()
        .quiet(false)
        .timestamp(stderrlog::Timestamp::Millisecond)
        // 0 => LevelFilter::Error,
        // 1 => LevelFilter::Warn,
        // 2 => LevelFilter::Info,
        // 3 => LevelFilter::Debug,
        // _ => LevelFilter::Trace,
        .verbosity(3)
        .init()
        .unwrap();
}
