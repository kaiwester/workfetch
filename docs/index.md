---
title: Getting Started
---

# workfetch — Quick Start

A small terminal application for visualizing your working day: start time (persisted
across reboots), rounded start, target work &amp; break durations, end-of-day time, and
remaining time. Includes a simple ASCII logo for a compact dashboard feel.

```
##################    System Start       : 07:41:12
###+=======+######    Rounded Start      : 07:45
###-          ####    -----------------------------------
###-   *##=    ###    Target Work Time   : 8 Std 0 Min
###-   *##:    ###    Break Time         : 45 Min
###-   +=-     ###    End of Day         : 16:30
###-          ####    -----------------------------------
###-   :.     ####    Remaining          : 5 Std 12 Min
###-   *##=    ###
###-   *##=    ###
###-   ++=     ###
###-          ####
###========+######
##################
```

> **Note on units:** Duration strings use German abbreviations — `Std` (Stunden =
> hours) and `Min` (Minuten = minutes).

---

## Install

### Quick install (recommended)

**Windows** (PowerShell):

```powershell
irm https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.ps1 | iex
```

**Linux**:

```sh
curl -fsSL https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.sh | sh
```

Restart your terminal, then run `workfetch`.

### Pre-built binaries

Download the latest archive for your platform from the
[Releases](https://github.com/KaiWesterschwiensterdt/workfetch/releases/latest) page,
extract it, and place the binary somewhere on your `PATH`.

### With Rust (`cargo install`)

If you have the Rust toolchain installed, install straight from the repository:

```sh
cargo install --git https://github.com/KaiWesterschwiensterdt/workfetch
```

This builds and drops `workfetch` into `~/.cargo/bin` (already on your PATH), so you can
run `workfetch` immediately in a new terminal.

### Build from source

```sh
git clone https://github.com/KaiWesterschwiensterdt/workfetch.git
cd workfetch
cargo build --release

# Binary is at:
./target/release/workfetch          # Linux
.\target\release\workfetch.exe      # Windows
```

---

## Usage

```
workfetch [OPTIONS]

Options:
      --work <MINUTES>         Override work duration in minutes
      --break <MINUTES>        Override break duration in minutes
      --override-start <HH:MM> Override today's work start time
  -h, --help                   Print help
```

### Examples

```sh
# Use config file defaults
workfetch

# 7-hour workday with 30-minute break, just for this run
workfetch --work 420 --break 30

# Record that you actually started at 09:00 today
workfetch --override-start 09:00
```

`--work` and `--break` apply only for the current run and do not modify `config.toml`.  
`--override-start` writes the given time to `last_session.json` for today and exits.

---

## Configuration

A TOML file is created on first run at the platform config directory:

| Platform | Path |
|----------|------|
| Linux    | `~/.config/workfetch/config.toml` |
| Windows  | `%APPDATA%\workfetch\config.toml` |

Fallback (if directory resolution fails): `./config.toml` in the working directory.

```toml
# config.toml
work_minutes = 480    # Total planned work time in minutes (8 h default)
break_minutes = 45    # Planned break time in minutes (45 min default)
```

Change values and re-run — no rebuild required.

---

## Persistence model

- `last_session.json` stores the start timestamp for the current day.
- If the file's date matches today, that time is reused (survives reboots).
- If the date has changed (new day), boot time becomes the new start and the file is
  overwritten.
- Use `--override-start HH:MM` to manually correct the start time for today.

---

## Uninstall

**Windows** (PowerShell):

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.ps1))) -Uninstall
```

**Linux**:

```sh
curl -fsSL https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.sh | sh -s -- --uninstall
```

---

<p align="center">
  <a href="https://github.com/KaiWesterschwiensterdt/workfetch">Source on GitHub</a>
</p>
