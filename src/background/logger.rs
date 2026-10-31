use std::sync::LazyLock;

use owo_colors::OwoColorize;
use windows::Win32::UI::Shell::FOLDERID_LocalAppData;

use crate::{error::Result, windows_api::WindowsApi};

pub struct DockBarLogger {}

fn format_now() -> String {
    static FMT: LazyLock<Vec<time::format_description::BorrowedFormatItem>> = LazyLock::new(|| {
        time::format_description::parse_borrowed::<1>(
            "[[[year]-[month]-[day]][[[hour]:[minute]:[second]]",
        )
        .expect("valid time format")
    });

    time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .format(&FMT)
        .unwrap_or_else(|_| "?time?".to_owned())
}

impl DockBarLogger {
    const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024; // 5MB

    pub fn init() -> Result<()> {
        let logs_folder =
            WindowsApi::known_folder(FOLDERID_LocalAppData)?.join("com.dockbar.app/logs");
        std::fs::create_dir_all(&logs_folder)?;

        let log_path = logs_folder.join("DockBar.log");
        if log_path.exists() {
            let metadata = std::fs::metadata(&log_path)?;
            if metadata.len() > Self::MAX_LOG_SIZE {
                let bak_path = logs_folder.join("DockBar.log.bak");
                std::fs::rename(&log_path, &bak_path)?;
            }
        }

        let file_dispatch = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    format_now(),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .chain(fern::log_file(log_path)?);

        // Trace in dev for diagnosis, Info in release: the file is only rotated on
        // startup, so a chatty dependency logging at trace level can grow it to
        // gigabytes in a single long-running session (a render loop did exactly
        // that once), which also costs constant I/O on every frame.
        let base_level = if cfg!(dev) {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        };

        let dispatch = fern::Dispatch::new()
            .level(base_level)
            .level_for("tao", log::LevelFilter::Error)
            .level_for("os_info", log::LevelFilter::Error)
            .level_for("notify", log::LevelFilter::Error)
            .level_for("notify_debouncer_full", log::LevelFilter::Error)
            .level_for("discord_presence", log::LevelFilter::Error)
            .level_for("wmi", log::LevelFilter::Error)
            .level_for("tungstenite", log::LevelFilter::Error)
            .level_for("tokio_tungstenite", log::LevelFilter::Error)
            // async runtime / server / http-client internals are extremely chatty
            // at trace level and carry no diagnostic value for us
            .level_for("tokio", log::LevelFilter::Error)
            .level_for("mio", log::LevelFilter::Error)
            .level_for("salvo", log::LevelFilter::Error)
            .level_for("hyper", log::LevelFilter::Error)
            .level_for("reqwest", log::LevelFilter::Error)
            .level_for("rustls", log::LevelFilter::Error)
            .level_for("scc", log::LevelFilter::Error)
            .chain(file_dispatch);

        #[cfg(dev)]
        let dispatch = dispatch.chain(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "{}[{}][{}] {}",
                        format_now(),
                        match record.level() {
                            log::Level::Error => "ERROR".red().to_string(),
                            log::Level::Warn => "WARN~".yellow().to_string(),
                            log::Level::Info => "INFO~".bright_blue().to_string(),
                            log::Level::Debug => "DEBUG".bright_green().to_string(),
                            log::Level::Trace => "TRACE".bright_black().to_string(),
                        },
                        if record.level() == log::Level::Error {
                            record
                                .file()
                                .filter(|f| !f.contains("exposed.rs"))
                                .map(|f| {
                                    format!(
                                        "{}:{}",
                                        f.replace("\\", "/"),
                                        record.line().unwrap_or_default()
                                    )
                                })
                                .unwrap_or_else(|| record.target().to_owned())
                                .bright_red()
                                .to_string()
                        } else {
                            record.target().bright_black().to_string()
                        },
                        message
                    ))
                })
                .chain(std::io::stdout()),
        );

        dispatch.apply()?;
        register_panic_hook();
        Ok(())
    }
}

pub fn register_panic_hook() {
    let base_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let cause = info
            .payload()
            .downcast_ref::<String>()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                info.payload()
                    .downcast_ref::<&str>()
                    .unwrap_or(&"<cause unknown>")
                    .to_string()
            });

        let mut string_location = String::from("<location unknown>");
        if let Some(location) = info.location() {
            string_location = format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }

        log::error!("A panic occurred:\n  Cause: {cause}\n  Location: {string_location}");
        base_hook(info);
    }));
}
