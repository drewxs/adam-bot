use log;

use crate::cfg::LOG_FILE;

pub fn setup_logging() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("tracing", log::LevelFilter::Error)
        .level_for("serenity", log::LevelFilter::Error)
        .level_for("songbird", log::LevelFilter::Error)
        .level_for("symphonia_core", log::LevelFilter::Error)
        .level_for("symphonia_bundle_mp3", log::LevelFilter::Error)
        .chain(std::io::stdout())
        .chain(fern::log_file(LOG_FILE).unwrap())
        .apply()
        .unwrap();
}
