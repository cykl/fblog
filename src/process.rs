use crate::filter;
use crate::inspect::InspectLogger;
use crate::log::{self, LogSettings};
use ansi_term::{Colour, Style};
use lazy_static::lazy_static;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::io::Write;
use std::io::{self, BufRead};

lazy_static! {
  static ref BOLD_ORANGE: Style = Colour::RGB(255, 135, 22).bold();
}

pub fn process_input(log_settings: &LogSettings, input: &mut dyn io::BufRead, maybe_filter: Option<&str>, implicit_return: bool) {
  for line in input.lines() {
    let read_line = &line.expect("Should be able to read line");
    match process_input_line(log_settings, read_line, None, maybe_filter, implicit_return) {
      Ok(_) => (),
      Err(_) => print_unknown_line(read_line),
    }
  }
}

fn print_unknown_line(line: &str) {
  println!("{} {}", BOLD_ORANGE.paint("??? >"), line);
}

fn process_input_line(
  log_settings: &LogSettings,
  read_line: &str,
  maybe_prefix: Option<&str>,
  maybe_filter: Option<&str>,
  implicit_return: bool,
) -> Result<(), ()> {
  let mut inspect_logger = InspectLogger::new();
  match serde_json::from_str::<Value>(read_line) {
    Ok(Value::Object(log_entry)) => {
      process_json_log_entry(log_settings, &mut inspect_logger, maybe_prefix, &log_entry, maybe_filter, implicit_return);
      Ok(())
    }
    _ => {
      if !log_settings.inspect {
        if log_settings.with_prefix && maybe_prefix.is_none() {
          match read_line.find('{') {
            Some(pos) => {
              let prefix = &read_line[..pos];
              let rest = &read_line[pos..];
              process_input_line(log_settings, rest, Some(prefix), maybe_filter, implicit_return)
            }
            None => Err(()),
          }
        } else {
          Err(())
        }
      } else {
        Err(())
      }
    }
  }
}

fn process_json_log_entry(
  log_settings: &LogSettings,
  inspect_logger: &mut InspectLogger,
  maybe_prefix: Option<&str>,
  log_entry: &Map<String, Value>,
  maybe_filter: Option<&str>,
  implicit_return: bool,
) {
  let string_log_entry = &extract_string_values(log_entry);
  if let Some(filter) = maybe_filter {
    match filter::show_log_entry(string_log_entry, filter, implicit_return) {
      Ok(true) => process_log_entry(log_settings, inspect_logger, maybe_prefix, string_log_entry),
      Ok(false) => (),
      Err(e) => {
        writeln!(io::stderr(), "{}: '{:?}'", Colour::Red.paint("Failed to apply filter expression"), e).expect("Should be able to write to stderr");
        std::process::exit(1)
      }
    }
  } else {
    process_log_entry(log_settings, inspect_logger, maybe_prefix, string_log_entry)
  }
}

fn process_log_entry(log_settings: &LogSettings, inspect_logger: &mut InspectLogger, maybe_prefix: Option<&str>, log_entry: &BTreeMap<String, String>) {
  if log_settings.inspect {
    inspect_logger.print_unknown_keys(log_entry, &mut io::stdout())
  } else {
    log::print_log_line(&mut io::stdout(), maybe_prefix, log_entry, log_settings)
  }
}

fn extract_string_values(log_entry: &Map<String, Value>) -> BTreeMap<String, String> {
  log_entry
    .iter()
    .filter_map(|(key, value)| {
      if let Value::String(ref string_value) = *value {
        Some((key.to_owned(), string_value.to_owned()))
      } else {
        None
      }
    })
    .collect()
}
