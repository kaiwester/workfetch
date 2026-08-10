# workfetch

[![CI](https://github.com/KaiWesterschwiensterdt/workfetch/actions/workflows/ci.yml/badge.svg)](https://github.com/KaiWesterschwiensterdt/workfetch/actions/workflows/ci.yml)
[![Release](https://github.com/KaiWesterschwiensterdt/workfetch/actions/workflows/release.yml/badge.svg)](https://github.com/KaiWesterschwiensterdt/workfetch/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE.md)

A small terminal application for visualizing your working day: start time (persisted across reboots), rounded start, target work & break durations, end-of-day time, and remaining time. Includes a simple ASCII logo for a compact dashboard feel.

📖 **Full guide & quick start:** <https://kaiwester.github.io/workfetch/>

## Features

- Auto-detects system boot time and persists session start (`last_session.json`).
- Rounds start time to nearest 15 minutes.
- User-configurable work & break durations via `config.toml` or CLI flags.
- Override today's start time manually with `--override-start`.
- Colorized, aligned output for a quick terminal glance.
- Survives system reboot: restores the previous start time if still the same day.
- Cross-platform config paths (Linux, macOS, Windows) via `directories`.

## Install

**Windows** (PowerShell):

```powershell
irm https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.ps1 | iex
```

**Linux / macOS**:

```sh
curl -fsSL https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.sh | sh
```

Restart your terminal, then run `workfetch`.

For pre-built binaries, `cargo install`, building from source, usage, configuration,
the persistence model, and uninstall instructions, see the
[**full guide**](https://kaiwester.github.io/workfetch/).

## Roadmap / Ideas

- Validation & friendly warnings for extreme config values.
- Optional lunch break logic / multi-break schedule.
- Export daily summary (CSV / JSON).
- Scoop / winget / Homebrew package manager support.

## Contributing

Open an issue or PR with a concise description. Keep changes focused and small.

## License

This project is licensed under the MIT License — see [LICENSE.md](LICENSE.md) for details.
