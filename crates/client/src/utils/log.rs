const LOG_FOLDER: &str = "D:/azookey-windows/logs";

pub fn setup_logger() -> anyhow::Result<()> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d-%H.%M.%S");
    let log_file = format!("{}/{}.log", LOG_FOLDER, timestamp);

    std::fs::create_dir_all(LOG_FOLDER)?;
    std::fs::File::create(&log_file)?;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file(&log_file)?)
        .apply()?;

    Ok(())
}
