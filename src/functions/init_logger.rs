use log::*;

pub fn init_logger(level: LevelFilter) {
    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .init();
    trace!("Logger started.")
}
