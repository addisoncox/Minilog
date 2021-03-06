//! A simple configurable logger implementation
//!
//! Supports logging to a specified file, as well as
//! setting and adjusting log message levels, and the
//! format of log messages

use log::*;
use std::fs::OpenOptions;
use std::io::Write;

/// Consists of name for path of file to log to, and string
/// which serves as a format string for log messages
pub struct Minilog {
	logfile_name: String,
	fmt_string: String,
}

impl Minilog {
	/// Initializes the logger, must be called before attempting
	/// to write log messages
	///
	/// # Examples
	///
	/// ```
	/// # use log::LevelFilter;
	/// # use minilog::Minilog;
	/// Minilog::init(LevelFilter::Info, "logs.txt", "{level} - {msg}");
	/// ```
	pub fn init(
		loglevel: LevelFilter,
		logfile_name: &str,
		fmt_string: &str,
	) -> Result<(), SetLoggerError> {
		set_boxed_logger(Box::new(Minilog {
			logfile_name: logfile_name.to_owned(),
			fmt_string: fmt_string.to_owned(),
		}))
		.map(|()| set_max_level(loglevel))
	}
	///Initializes a logger with default settings
	///
	/// # Examples
	/// ```
	/// # use log::LevelFilter;
	/// # use minilog::Minilog;
	/// Minilog::init_default();
	/// ```
	pub fn init_default() -> Result<(), SetLoggerError> {
		set_boxed_logger(Box::new(Minilog {
			logfile_name: "logs.txt".to_owned(),
			fmt_string: "{level}: {msg}".to_owned(),
		}))
		.map(|()| set_max_level(LevelFilter::Trace))
	}
	///Sets the maximum level of log message to write
	///
	/// # Examples
	///
	/// ```
	/// # use log::LevelFilter;
	/// # use minilog::Minilog;
	/// Minilog::set_log_level(LevelFilter::Info);
	/// ```
	pub fn set_log_level(loglevel: LevelFilter) {
		set_max_level(loglevel);
	}
	///Logs or panics if loglevel is too low
	///
	/// # Examples
	///
	/// ```
	/// # use log::Level;
	/// # use log::LevelFilter;
	/// # use minilog::Minilog;
	/// # use std::fs;
	/// Minilog::init(LevelFilter::Info, "minilog_output_test.txt", "{level} - {msg}");
	/// Minilog::log_or_panic(Level::Error, "Error!");
	/// # fs::remove_file("minilog_output_test.txt").expect("Unable to delete test file.");
	/// ```
	///
	/// ```should_panic
	/// # use log::Level;
	/// # use log::LevelFilter;
	/// # use minilog::Minilog;
	/// //should panic
	/// Minilog::init(LevelFilter::Info, "minilog_output_test.txt", "{level} - {msg}");
	/// Minilog::log_or_panic(Level::Trace, "Trace!");
	/// ```
	///
	pub fn log_or_panic(loglevel: Level, msg: &str) {
		if loglevel > max_level() {
			panic!("{} is too low to log", loglevel);
		}
		let mut record = Record::builder();
		log!(loglevel,
			"{}", format!(
				"{}", format_args!(
					"{}", record
						.args(format_args!("{}", msg))
						.level(loglevel)
						.build()
						.args()
				)
			)
		);
	}
	///logs a message, upgrading the log level if log level isn't high enough
	/// ```
	/// # use log::{Level, LevelFilter};
	/// # use minilog::Minilog;
	/// # use std::fs;
	/// Minilog::init(LevelFilter::Info, "minilog_output_test.txt", "{level} - {msg}");
	/// Minilog::log_upgrade(Level::Trace, "Trace!");
	/// let file_contents =
	///		fs::read_to_string("minilog_output_test.txt").expect("Was unable to read file.");
	///# fs::remove_file("minilog_output_test.txt").expect("Unable to delete test file.");
	///assert_eq!(
	///		file_contents,
	///		"TRACE - Trace!\n"
	///);
	/// ```
	pub fn log_upgrade(loglevel: Level, msg: &str) {
		if loglevel > max_level() {
			set_max_level(loglevel.to_level_filter())
		}
		let mut record = Record::builder();
		log!(loglevel,
			"{}", format!(
				"{}", format_args!(
					"{}", record
						.args(format_args!("{}", msg))
						.level(loglevel)
						.build()
						.args()
				)
			)
		);
	}
	///logs a message, temporarily upgrading loglevel if it isn't high enough
	/// ```
    /// # use log::{Level, LevelFilter, trace};
	/// # use minilog::Minilog;
	/// # use std::fs;
	/// Minilog::init(LevelFilter::Info, "minilog_output_test.txt", "{level} - {msg}");
	/// Minilog::log_upgrade(Level::Trace, "Trace!");
	/// let file_contents =
	///		fs::read_to_string("minilog_output_test.txt").expect("Was unable to read file.");
	///# fs::remove_file("minilog_output_test.txt").expect("Unable to delete test file.");
	///assert_eq!(
	///		file_contents,
	///		"TRACE - Trace!\n"
	///);
	/// ```
	pub fn log_upgrade_temp(loglevel: Level, msg: &str) {
		if loglevel > max_level() {
			let current_level = max_level(); 
			Minilog::log_upgrade(loglevel, msg);
			set_max_level(current_level);
		}
	}
    ///returns option with the maximum log level, none if logging is off
    ///```
    ///# use log::{Level, LevelFilter, trace};
    ///# use minilog::Minilog;
    ///Minilog::init(LevelFilter::Trace, "minilog_output_test.txt", "{level} - {msg}");
    ///assert_eq!(Minilog::log_level(), Some(Level::Trace));
    ///```
    pub fn log_level() -> Option<Level> {
        max_level().to_level()
    }
}

impl Log for Minilog {
	///Returns whether logging is enabled for a given level
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.level() <= max_level()
	}

	///Logs a message to file, using the format string provided.
	/// The "level", "msg", "modpath", "line", or "file" enclosed in
	/// curly braces will be replaced.
	/// # Panics
	/// Panics if it can't open the file or write to it
	fn log(&self, record: &Record) {
		if self.enabled(record.metadata()) {
			let log_msg = self.fmt_string
				.replacen(
					"{level}",
					&format!("{}", format_args!("{}", record.level())),
					1,
				)
				.replacen(
					"{msg}",
					&format!("{}", format_args!("{}", record.args())),
					1,
				)
				.replacen(
					"{modpath}",
					&format!("{}", format_args!("{}", record.module_path().unwrap_or(""))),
					1,
				)
				.replacen(
					"{file}",
					&format!("{}", format_args!("{}", record.file().unwrap_or(""))),
					1,
				)
				.replacen(
					"{line}",
					&format!("{}", format_args!("{}", record.line().unwrap_or(0))),
					1
				);
			if self.logfile_name == "stdout" {
				println!("{}", log_msg);
			} else if self.logfile_name == "stderr" {
				eprintln!("{}", log_msg);
			} else {
				let mut file = OpenOptions::new()
					.read(true)
					.append(true)
					.create(true)
					.open(&self.logfile_name);
				match &mut file {
					Ok(file) => match writeln!(file, "{}", log_msg) {
						Ok(_) => {}
						Err(e) => panic!("{}: Write failed", e),
					},
					Err(e) => panic!("{}: Failed to write to logfile {}", e, &self.logfile_name),
				}
			}
		}
	}

	///preserved for trait implementation
	fn flush(&self) {}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;
	use std::path::Path;
	#[test]
	fn test() {
		match Minilog::init(LevelFilter::Info, "Minilog_test_main.txt", "{level}: {msg}") {
			Ok(_) => {}
			Err(e) => panic!("{}: Could not set the logger!", e),
		}
		log!(Level::Error, "Test log!");
		error!("Test error!");
		warn!("Test warning!");
		trace!("Test trace! exluded");
		Minilog::set_log_level(LevelFilter::Trace);
		trace!("Test trace! not excluded");
		let file_contents =
			fs::read_to_string("Minilog_test_main.txt").expect("Was unable to read file.");
		fs::remove_file("Minilog_test_main.txt").expect("Unable to delete test file.");
		assert_eq!(
			file_contents,
			"ERROR: Test log!\nERROR: Test error!\nWARN: Test warning!\nTRACE: Test trace! not excluded\n"
		);
	}
	#[test]
	#[ignore]
	fn test_direct_to_stdout_log() {
		match Minilog::init(LevelFilter::Info, "stdout", "{level}: {msg}") {
			Ok(_) => {}
			Err(e) => panic!("{}: Could not set the logger!", e),
		}
		info!("Log message");
		assert_eq!(false, Path::new("stdout").exists())
	}
}
