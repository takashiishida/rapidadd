use chrono::Local;
use clap::{App, Arg};
use config::{Config, ConfigError, File as ConfigFile};
use dirs;
use serde::Deserialize;
use std::{
    io::{prelude::*, Seek, SeekFrom},
    path::PathBuf,
};

#[derive(Debug, Deserialize)]
struct AppConfig {
    daily_path: String,
    file_extension: String,
    date_format: String,
}

fn load_config() -> Result<AppConfig, ConfigError> {
    let home_dir = dirs::home_dir().expect("Could not find the home directory");
    // if you have a different location, change the config file path
    let config_path = home_dir.join(".config/rapidadd/config.toml");
    // you can also edit the default values directly w/o preparing a config file
    let builder = Config::builder()
        .set_default("daily_path", "./")?
        .set_default("file_extension", "md")?
        .set_default("date_format", "%Y-%m-%d_%a")?
        .add_source(ConfigFile::from(config_path).required(false));

    let config = builder.build()?;

    return config.try_deserialize::<AppConfig>();
}

fn get_daily_file_path(config: &AppConfig) -> PathBuf {
    let date = Local::now()
        .date_naive()
        .format(&config.date_format)
        .to_string();
    let filename = format!("{}.{}", date, config.file_extension);
    let mut file_path = PathBuf::from(&config.daily_path);
    file_path.push(filename);

    file_path
}


fn main() -> std::io::Result<()> {
    let matches = App::new("RapidAdd")
        .about("Quickly add to the end of a daily file, or print its content.")
        .arg(
            Arg::with_name("entry")
                .help("The text that will be appended to the end of your daily file.")
                .required_unless("print") // Make it required unless "print" is used
                .conflicts_with("print") // ensure mutual exclusivity
                .multiple(true)
                .takes_value(true)
                .index(1),
        )
        .arg(
            Arg::with_name("print")
                .short('p') // a shorthand for the flag `-p`
                .long("print")
                .help("Prints the content of the daily file.")
                .takes_value(false) // because it's a boolean flag
                .conflicts_with("entry"), // ensure mutual exclusivity
        )
        .get_matches();

    let config = load_config().expect("Failed to load config");

    // Handle the "print" flag
    if matches.is_present("print") {
        let file_path = get_daily_file_path(&config);

        // Attempt to open the file for reading
        match std::fs::read_to_string(&file_path) {
            Ok(contents) => {
                println!("{}", contents);
                return Ok(());
            },
            Err(e) => {
                eprintln!("Failed to read the daily file: {}", e);
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to read the daily file."));
            }
        }
    } else {
        let user_entry = matches.values_of("entry").unwrap().collect::<Vec<&str>>().join(" ");
        let time = Local::now().format("%H:%M").to_string();
        let mut entry = format!("- {} {}", time, user_entry);
        let file_path = get_daily_file_path(&config);
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .read(true) // Add read permission to check the last char
            .open(&file_path)?; // we don't want .create(true) here to avoid creating new files

        // check if file ends with \n, if not, add \n
        let file_len = file.metadata()?.len(); // Get the length of the file
        if file_len > 0 {
            let mut buffer = vec![0; 1]; // Create a buffer to read the last byte
            file.seek(SeekFrom::End(-1))?; // Seek to the last byte of the file
            file.read_exact(&mut buffer)?;
            if buffer[0] != b'\n' {
                entry.insert_str(0, "\n");
            }
        }

        file.seek(SeekFrom::End(0))?;
        writeln!(file, "{}", entry)?;
        println!("Entry added to {}", file_path.to_string_lossy());
    }

    Ok(())
}
