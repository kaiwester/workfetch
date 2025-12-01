# workfetch

A small terminal application for visualizing your working day: start time (persisted across reboot), rounded start, target work & break durations, end-of-day time, and remaining minutes. Includes a simple ASCII logo for a compact dashboard feel.

## Features

- Auto-detects system boot time and persists session start (`last_session.json`).
- Rounds start time to nearest 15 minutes (configurable logic point for future).
- User-configurable work & break durations via `config.toml`.
- Colorized, aligned output for quick terminal glance.
- Survives system reboot: will restore previous start time if same day.

## Installation

```powershell
# Clone
git clone https://github.com/KaiWesterschwiensterdt/workfetch.git
cd workfetch

# Build
cargo build --release

# Run (dev)
cargo run

# Run (release binary)
./target/release/workfetch
```

## Configuration

A TOML file is created on first run:

**Path (Windows example):** `C:\Users\<YOU>\AppData\Roaming\com\internal\workfetch\config.toml`

Fallback (if `ProjectDirs` fails): `./config.toml` in the working directory.

```toml
# config.toml
work_minutes = 480    # Total planned work time in minutes (8h default)
break_minutes = 45    # Planned break time in minutes (45m default)
```

Change values and re-run the tool; no rebuild needed unless you change code.

## Persistence Model

- `last_session.json` stores the starting point for the day.
- If the file's date matches today, that time is reused.
- If system reboot occurred midday, start time is preserved ("Restored Start").
- If date changed (new day), boot time becomes new start and file is overwritten.

## Output Example

```
##################    System Start       : 07:41:12
###+=======+######    Rounded Start      : 07:45
###-          ####    Target Work Time   : 8 Std 0 Min
###-   *##=    ###    Break Time         : 45 Min
###-   *##:    ###    End of Day         : 16:30
###-   +=-     ###    -----------------------------------
###-          ####    Remaining          : 5 Std 12 Min
###-   :.     ####
###-   *##=    ###
###-   *##=    ###
###-   ++=     ###
###-          ####
###========+######
##################
```

## Roadmap / Ideas

- CLI flags to override config for a single run (`--work 420 --break 30`).
- Validation & friendly warnings for extreme values.
- Optional lunch break logic / multi-break schedule.
- Export daily summary (CSV / JSON).
- Cross-platform packaging.

## Contributing

Open an issue or PR with concise description. Keep changes focused and small.

## License

Specify a license if desired (e.g. MIT). Currently unspecified.

---

Feel free to suggest improvements or request features.
