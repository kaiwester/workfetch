---
title: Getting Started
---

# workfetch — Quick Start

A tiny terminal dashboard for your working day: start time (persisted across
reboots), rounded start, target work &amp; break durations, end-of-day time, and
remaining time.

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
###========+######
##################
```

---

## Install

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.ps1 | iex
```

### Linux / macOS

```sh
curl -fsSL https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.sh | sh
```

### With Rust (any platform)

```sh
cargo install --git https://github.com/KaiWesterschwiensterdt/workfetch
```

Restart your terminal, then run:

```sh
workfetch
```

---

## Usage

```
workfetch [OPTIONS]

      --work <MINUTES>          Override work duration for this run
      --break <MINUTES>         Override break duration for this run
      --override-start <HH:MM>  Record today's actual start time
  -h, --help                    Print help
```

```sh
# Defaults from config
workfetch

# 7-hour day, 30-min break, this run only
workfetch --work 420 --break 30

# You actually started at 09:00 today
workfetch --override-start 09:00
```

---

## Configuration

A `config.toml` is created on first run:

| Platform | Path |
|----------|------|
| Windows  | `%APPDATA%\workfetch\config.toml` |
| Linux    | `~/.config/workfetch/config.toml` |
| macOS    | `~/Library/Application Support/workfetch/config.toml` |

```toml
work_minutes = 480    # planned work time (8 h)
break_minutes = 45    # planned break time
```

Change values, re-run — no rebuild needed.

---

## Uninstall

**Windows:**

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.ps1))) -Uninstall
```

**Linux / macOS:**

```sh
curl -fsSL https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.sh | sh -s -- --uninstall
```

---

<p align="center">
  <a href="https://github.com/KaiWesterschwiensterdt/workfetch">Source on GitHub</a>
</p>
