use chrono::{DateTime, Duration, Local, NaiveTime, TimeDelta, Timelike, Utc};
use chrono::TimeZone as _;
use clap::Parser;
use colored::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use sysinfo::System;

/// Persisted work session, stored as JSON in the platform config directory.
///
/// A new session is written whenever the stored date differs from today,
/// using the system boot time as the initial start time.
#[derive(Serialize, Deserialize, Debug)]
struct WorkSession {
    /// The local timestamp at which the work session began.
    start_time: DateTime<Local>,
}

/// User-configurable durations that control the end-of-day calculation.
///
/// Persisted as TOML in the platform config directory. Defaults are written
/// on first run (480 min work, 45 min break) and can be overridden per-run
/// via `--work` and `--break` CLI flags.
#[derive(Serialize, Deserialize, Debug)]
struct UserConfig {
    /// Target number of working minutes per day (default: 480, i.e. 8 hours).
    work_minutes: u32,
    /// Break duration in minutes added on top of work time (default: 45).
    break_minutes: u32,
}

/// Command-line arguments for workfetch.
///
/// All flags are optional. When omitted the values from `config.toml` are used.
#[derive(Parser)]
#[command(about = "Visualize your working hours at a glance")]
struct Cli {
    /// Override work duration in minutes
    #[arg(long)]
    work: Option<u32>,
    /// Override break duration in minutes
    #[arg(long = "break")]
    r#break: Option<u32>,
    /// Override today's work start time (format: HH:MM)
    #[arg(long = "override-start")]
    override_start: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // Handle --override-start: write new session and exit
    if let Some(time_str) = &cli.override_start {
        match NaiveTime::parse_from_str(time_str, "%H:%M") {
            Ok(t) => {
                let today = Local::now().date_naive();
                let naive_dt = today.and_time(t);
                let start_dt: DateTime<Local> = Local
                    .from_local_datetime(&naive_dt)
                    .single()
                    .expect("Ambiguous or invalid local time");
                if start_dt > Local::now() + Duration::minutes(1) {
                    eprintln!("workfetch: warning: override start time is in the future");
                }
                let session = WorkSession { start_time: start_dt };
                let file_path = get_config_path();
                if let Err(e) = save_session(&file_path, &session) {
                    eprintln!("workfetch: error: could not save session: {e}");
                    std::process::exit(1);
                }
                println!("Start time set to {} for today.", start_dt.format("%H:%M"));
                return;
            }
            Err(_) => {
                eprintln!("workfetch: error: invalid time format '{}', expected HH:MM", time_str);
                std::process::exit(1);
            }
        }
    }

    // Load or create user configuration for work/break durations
    let mut user_cfg: UserConfig = load_or_create_user_config();
    if let Some(w) = cli.work {
        user_cfg.work_minutes = w;
    }
    if let Some(b) = cli.r#break {
        user_cfg.break_minutes = b;
    }

    // Get or create start time from persistent storage
    let real_start_time: DateTime<Local> = get_or_create_start_time();

    // Round to the nearest 15 minutes
    let rounded_start_time: DateTime<Local> = round_to_nearest_15(real_start_time);

    // Calculate target time
    let work_duration: TimeDelta = Duration::minutes(user_cfg.work_minutes as i64);
    let break_duration: TimeDelta = Duration::minutes(user_cfg.break_minutes as i64);
    let total_required: TimeDelta = work_duration + break_duration;

    let end_time: DateTime<Local> = rounded_start_time + total_required;

    // Calculate remaining time
    let now: DateTime<Local> = Local::now();
    let remaining: TimeDelta = end_time - now;

    // Visual indicator if we are using a restored time vs fresh boot
    let boot_dt = boot_time_as_datetime();
    let source_label: &str = if real_start_time.date_naive() == boot_dt.date_naive()
        && real_start_time < boot_dt
    {
        "Restored Start" // Using cached time (reboot detected)
    } else {
        "System Start" // Using fresh boot time
    };

    // Collect entries for side-by-side output
    let mut entries: Vec<(&str, String, &str)> = Vec::new();
    entries.push((
        source_label,
        real_start_time.format("%H:%M:%S").to_string(),
        "blue",
    ));
    entries.push((
        "Rounded Start",
        rounded_start_time.format("%H:%M").to_string(),
        "cyan",
    ));
    entries.push((
        "---",
        "-----------------------------------".to_string(),
        "dimmed",
    ));
    entries.push((
        "Target Work Time",
        create_duration_string(user_cfg.work_minutes as i64),
        "green",
    ));
    entries.push((
        "Break Time",
        create_duration_string(user_cfg.break_minutes as i64),
        "green",
    ));
    entries.push((
        "End of Day",
        end_time.format("%H:%M").to_string(),
        "magenta",
    ));
    entries.push((
        "---",
        "-----------------------------------".to_string(),
        "dimmed",
    ));

    if remaining.num_seconds() > 0 {
        entries.push((
            "Remaining",
            create_duration_string(remaining.num_minutes()),
            "yellow",
        ));
    } else {
        entries.push(("Remaining", "DONE! 🎉".to_string(), "red"));
        entries.push((
            "",
            "You have reached your goal for today.".to_string(),
            "bold",
        ));
    }

    print_logo_and_entries(&entries);
}

/// Formats a minute count as a human-readable duration string.
///
/// Uses German abbreviations: `Std` (Stunden = hours), `Min` (Minuten = minutes).
///
/// # Examples
/// ```
/// // 45 → "45 Min"
/// // 90 → "1 Std 30 Min"
/// // 480 → "8 Std 0 Min"
/// ```
fn create_duration_string(total_minutes: i64) -> String {
    let hours: i64 = total_minutes / 60;
    let minutes: i64 = total_minutes % 60;
    if hours > 0 {
        format!("{} Std {} Min", hours, minutes)
    } else {
        format!("{} Min", minutes)
    }
}

/// Loads the user configuration from disk, or creates and writes a default if absent.
///
/// Config is read from `config.toml` in the platform config directory.
/// If the file is missing or unparseable, the defaults (480 min work, 45 min break)
/// are written to disk and returned. A warning is printed to stderr if the write fails.
fn load_or_create_user_config() -> UserConfig {
    let path: PathBuf = get_user_config_path();
    // Try read existing
    if let Ok(contents) = fs::read_to_string(&path) {
        if let Ok(cfg) = toml::from_str::<UserConfig>(&contents) {
            return cfg;
        }
    }

    // Defaults
    let default_cfg = UserConfig {
        work_minutes: 480, // 8 hours
        break_minutes: 45, // 45 minutes
    };
    if let Ok(serialized) = toml::to_string(&default_cfg) {
        if let Err(e) = fs::write(&path, serialized) {
            eprintln!("workfetch: warning: could not save config: {e}");
        }
    }
    default_cfg
}

/// Returns today's work start time, persisting it if not already recorded.
///
/// Decision logic:
/// 1. If `last_session.json` contains a timestamp **from today**, return it as-is.
/// 2. Otherwise (first run, new day, or stale file), use the system boot time as the
///    new start, write it to `last_session.json`, and return it.
///
/// A warning is printed to stderr if the session file cannot be written.
fn get_or_create_start_time() -> DateTime<Local> {
    let file_path: PathBuf = get_config_path();
    let now: DateTime<Local> = Local::now();

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
    let boot_time: DateTime<Local> = boot_time_as_datetime();

    // Save this new session to file
    let new_session: WorkSession = WorkSession {
        start_time: boot_time,
    };
    if let Err(e) = save_session(&file_path, &new_session) {
        eprintln!("workfetch: warning: could not save session: {e}");
    }

    boot_time
}

/// Returns the path to the session file (`last_session.json`).
///
/// Uses [`directories::ProjectDirs`] to resolve the platform config directory
/// (e.g. `~/.config/workfetch/` on Linux, `%APPDATA%\workfetch\` on Windows).
/// Falls back to `work_session.json` in the current directory if resolution fails.
/// Creates the config directory if it does not yet exist.
fn get_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "internal", "workfetch") {
        let config_dir = proj_dirs.config_dir();
        // Ensure directory exists
        if !config_dir.exists() {
            if let Err(e) = fs::create_dir_all(config_dir) {
                eprintln!("workfetch: warning: could not create config dir: {e}");
            }
        }
        return config_dir.join("last_session.json");
    }
    // Fallback to local directory if AppData fails
    PathBuf::from("work_session.json")
}

/// Returns the path to the user configuration file (`config.toml`).
///
/// Uses the same platform config directory as [`get_config_path`].
/// Falls back to `config.toml` in the current directory if resolution fails.
/// Creates the config directory if it does not yet exist.
fn get_user_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "internal", "workfetch") {
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            if let Err(e) = fs::create_dir_all(config_dir) {
                eprintln!("workfetch: warning: could not create config dir: {e}");
            }
        }
        return config_dir.join("config.toml");
    }
    PathBuf::from("config.toml")
}

/// Deserializes a [`WorkSession`] from a JSON file at `path`.
///
/// Returns an `Err` if the file does not exist or cannot be parsed.
fn read_session(path: &Path) -> Result<WorkSession, io::Error> {
    let file: File = File::open(path)?;
    let reader: BufReader<File> = io::BufReader::new(file);
    let session: WorkSession = serde_json::from_reader(reader)?;
    Ok(session)
}

/// Serializes `session` to JSON and writes it to `path`, truncating any existing file.
fn save_session(path: &Path, session: &WorkSession) -> Result<(), io::Error> {
    let file: File = File::create(path)?;
    serde_json::to_writer(file, session)?;
    Ok(())
}

/// Returns the system's last boot time as a [`DateTime<Local>`].
///
/// Reads the boot timestamp via [`sysinfo::System::boot_time`] and converts it
/// from a Unix epoch seconds value to a local datetime.
///
/// # Panics
/// Panics if the OS returns a timestamp that cannot be represented as a valid datetime
/// (practically impossible on any sane system).
fn boot_time_as_datetime() -> DateTime<Local> {
    let boot_time_sec: u64 = System::boot_time();
    Utc.timestamp_opt(boot_time_sec as i64, 0)
        .single()
        .expect("Invalid boot timestamp")
        .with_timezone(&Local)
}

/// Rounds `time` to the nearest 15-minute boundary, clearing seconds and nanoseconds.
///
/// Rounds down when the minute offset within the current quarter is 0–7,
/// and rounds up when it is 8–14. For example:
/// - 09:07 → 09:00
/// - 09:08 → 09:15
/// - 09:52 → 09:45 (offset 7, round down)
/// - 09:53 → 10:00 (offset 8, round up, carries hour)
fn round_to_nearest_15(time: DateTime<Local>) -> DateTime<Local> {
    let minute: u32 = time.minute();
    let remainder: u32 = minute % 15;

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

/// Prints the ASCII logo side-by-side with the data entries table.
///
/// Each entry is a tuple of `(label, value, color_name)` where `color_name` is
/// a string accepted by the `colored` crate (e.g. `"green"`, `"yellow"`).
/// Two special label values control display formatting:
/// - `"---"` renders the value as a dimmed separator line.
/// - `""` (empty) renders the value as bold, full-width text with no label column.
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

    let logo_width: usize = logo_lines.iter().map(|l| l.len()).max().unwrap_or(0);

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

    let max_rows: usize = logo_lines.len().max(formatted.len());

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
        println!(
            "{:<logo_width$}    {}",
            logo_part,
            entry_part,
            logo_width = logo_width
        );
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_time(h: u32, m: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(2024, 1, 1, h, m, 0).unwrap()
    }

    // --- round_to_nearest_15 ---

    #[test]
    fn round_stays_on_quarter() {
        assert_eq!(round_to_nearest_15(make_time(9, 0)).minute(), 0);
        assert_eq!(round_to_nearest_15(make_time(9, 15)).minute(), 15);
        assert_eq!(round_to_nearest_15(make_time(9, 30)).minute(), 30);
        assert_eq!(round_to_nearest_15(make_time(9, 45)).minute(), 45);
    }

    #[test]
    fn round_down_at_7() {
        let r = round_to_nearest_15(make_time(9, 7));
        assert_eq!(r.hour(), 9);
        assert_eq!(r.minute(), 0);
    }

    #[test]
    fn round_up_at_8() {
        let r = round_to_nearest_15(make_time(9, 8));
        assert_eq!(r.hour(), 9);
        assert_eq!(r.minute(), 15);
    }

    #[test]
    fn round_down_at_22() {
        let r = round_to_nearest_15(make_time(9, 22));
        assert_eq!(r.hour(), 9);
        assert_eq!(r.minute(), 15);
    }

    #[test]
    fn round_up_at_23() {
        let r = round_to_nearest_15(make_time(9, 23));
        assert_eq!(r.hour(), 9);
        assert_eq!(r.minute(), 30);
    }

    #[test]
    fn round_up_at_59_carries_hour() {
        let r = round_to_nearest_15(make_time(9, 59));
        assert_eq!(r.hour(), 10);
        assert_eq!(r.minute(), 0);
    }

    #[test]
    fn round_zeros_seconds_and_nanos() {
        let r = round_to_nearest_15(make_time(9, 3));
        assert_eq!(r.second(), 0);
        assert_eq!(r.nanosecond(), 0);
    }

    // --- create_duration_string ---

    #[test]
    fn duration_zero_minutes() {
        assert_eq!(create_duration_string(0), "0 Min");
    }

    #[test]
    fn duration_45_minutes() {
        assert_eq!(create_duration_string(45), "45 Min");
    }

    #[test]
    fn duration_one_hour_exactly() {
        assert_eq!(create_duration_string(60), "1 Std 0 Min");
    }

    #[test]
    fn duration_90_minutes() {
        assert_eq!(create_duration_string(90), "1 Std 30 Min");
    }

    #[test]
    fn duration_8_hours() {
        assert_eq!(create_duration_string(480), "8 Std 0 Min");
    }
}
