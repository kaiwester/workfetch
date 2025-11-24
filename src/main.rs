use chrono::{DateTime, Duration, Local, Timelike};
use colored::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use sysinfo::System;

// Structure to store in the file
#[derive(Serialize, Deserialize, Debug)]
struct WorkSession {
    start_time: DateTime<Local>,
}

fn main() {
    // Determine the actual start time (Load from file OR get boot time)
    let real_start_time = get_or_create_start_time();

    // Round to the nearest 15 minutes
    let rounded_start_time = round_to_nearest_15(real_start_time);

    // Calculate target time
    let work_duration = Duration::minutes((7.5 * 60.0) as i64);
    let break_duration = Duration::minutes(30);
    let total_required = work_duration + break_duration;

    let end_time = rounded_start_time + total_required;

    // Calculate remaining time
    let now = Local::now();
    let remaining = end_time - now;

    // Visual indicator if we are using a restored time vs fresh boot
    let source_label = if real_start_time.date_naive() == System::boot_time_as_datetime().date_naive() 
        && real_start_time < System::boot_time_as_datetime() {
        "Restored Start" // Using cached time (reboot detected)
    } else {
        "System Start"   // Using fresh boot time
    };

    // Collect entries for side-by-side output
    let mut entries: Vec<(&str, String, &str)> = Vec::new();
    entries.push((source_label, real_start_time.format("%H:%M:%S").to_string(), "blue"));
    entries.push(("Rounded Start", rounded_start_time.format("%H:%M").to_string(), "cyan"));
    entries.push(("---", "-----------------------------------".to_string(), "dimmed"));
    entries.push(("Target Work Time", "7 Std 30 Min".to_string(), "green"));
    entries.push(("Break Time", "30 Min".to_string(), "green"));
    entries.push(("End of Day", end_time.format("%H:%M").to_string(), "magenta"));
    entries.push(("---", "-----------------------------------".to_string(), "dimmed"));

    if remaining.num_minutes() > 0 {
        let hours = remaining.num_minutes() / 60;
        let minutes = remaining.num_minutes() % 60;
        entries.push(("Remaining", format!("{} hrs {} min", hours, minutes), "yellow"));
    } else {
        entries.push(("Remaining", "DONE! 🎉".to_string(), "red"));
        entries.push(("", "You have reached your goal for today.".to_string(), "bold"));
    }

    print_logo_and_entries(&entries);
}

/// Core Logic: Handles the persistence
fn get_or_create_start_time() -> DateTime<Local> {
    let file_path = get_config_path();
    let now = Local::now();

    // Try to read existing file
    if let Ok(session) = read_session(&file_path) {
        // CHECK: Is the stored date TODAY?
        if session.start_time.date_naive() == now.date_naive() {
            // It is today's file. Return the stored time.
            return session.start_time;
        }
    }

    // If we are here, either no file exists OR the file is from an old date.
    // We must calculate a fresh start time based on current uptime.
    let boot_time_sec = System::boot_time();
    let boot_time = DateTime::from_timestamp(boot_time_sec as i64, 0)
        .expect("Invalid boot timestamp")
        .with_timezone(&Local);

    // Save this new session to file
    let new_session = WorkSession { start_time: boot_time };
    let _ = save_session(&file_path, &new_session); // Ignore write errors for CLI simplicity

    boot_time
}

/// Helper to get a safe path to store the file: e.g., AppData/Roaming/WorkFetch
fn get_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "internal", "workfetch") {
        let config_dir = proj_dirs.config_dir();
        // Ensure directory exists
        if !config_dir.exists() {
            let _ = fs::create_dir_all(config_dir);
        }
        return config_dir.join("last_session.json");
    }
    // Fallback to local directory if AppData fails
    PathBuf::from("work_session.json")
}

fn read_session(path: &PathBuf) -> Result<WorkSession, io::Error> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let session = serde_json::from_reader(reader)?;
    Ok(session)
}

fn save_session(path: &PathBuf, session: &WorkSession) -> Result<(), io::Error> {
    let file = File::create(path)?;
    serde_json::to_writer(file, session)?;
    Ok(())
}

// Helper extension to get boot time as DateTime easily
trait BootTimeExt {
    fn boot_time_as_datetime() -> DateTime<Local>;
}

impl BootTimeExt for System {
    fn boot_time_as_datetime() -> DateTime<Local> {
        let boot_time_sec = System::boot_time();
        DateTime::from_timestamp(boot_time_sec as i64, 0)
            .unwrap()
            .with_timezone(&Local)
    }
}

fn round_to_nearest_15(time: DateTime<Local>) -> DateTime<Local> {
    let minute = time.minute();
    let remainder = minute % 15;

    if remainder < 8 {
        time - Duration::minutes(remainder as i64)
    } else {
        time + Duration::minutes((15 - remainder) as i64)
    }
    .with_second(0)
    .unwrap()
    .with_nanosecond(0)
    .unwrap()
}

fn print_logo_and_entries(entries: &[(&str, String, &str)]) {
    // Minified Beckhoff "B" logo
    let logo_lines = [
        "##################",
        "###+=======+######",
        "###-          ####",
        "###-   *##=    ###",
        "###-   *##:    ###",
        "###-   +=-     ###",
        "###-          ####",
        "###-   :.     ####",
        "###-   *##=    ###",
        "###-   *##=    ###",
        "###-   ++=     ###",
        "###-          ####",
        "###========+######",
        "##################",
    ];

    let logo_width = logo_lines.iter().map(|l| l.len()).max().unwrap_or(0);

    // Prepare formatted entry lines (with colors for text, not logo)
    let formatted: Vec<String> = entries
        .iter()
        .map(|(k, v, color)| {
            if *k == "---" {
                v.dimmed().to_string()
            } else if k.is_empty() {
                v.bold().to_string()
            } else {
                format!("{:<18} : {}", k.bold(), v.color(*color).bold())
            }
        })
        .collect();

    let max_rows = logo_lines.len().max(formatted.len());

    println!();
    for i in 0..max_rows {
        let logo_part = if i < logo_lines.len() {
            logo_lines[i]
        } else {
            ""
        };
        let entry_part = if i < formatted.len() {
            &formatted[i]
        } else {
            ""
        };
        println!("{:<logo_width$}    {}", logo_part, entry_part, logo_width = logo_width);
    }
    println!();
}