# MeshCoreStat

Cross‑platform console and TUI application for retrieving detailed runtime statistics from a MeshCore node over a serial (COM) port.

_Русская версия: см. [README.ru.md](README.ru.md)._ 

---

## Overview

MeshCoreStat connects to a MeshCore node via the specified COM port and sequentially issues the following [CLI commands](https://github.com/meshcore-dev/MeshCore/blob/main/docs/cli_commands.md):
`ver`, `board`, `get name`, `stats-core`, `stats-radio`, `stats-packets`.

The responses to these commands are parsed and rendered as a compact terminal dashboard. The layout is the same for both one‑shot and auto‑refresh modes and is optimized to fit into a typical terminal window.

### Screenshot

![MeshCoreStat TUI screenshot](assets/ss.png)

### Features

- **Cross‑platform CLI tool** – works on Windows and Linux (and other platforms supported by Rust and `serialport`).
- **Single snapshot or live monitoring** – one‑shot mode for quick checks, auto‑refresh mode for continuous observation.
- **Compact terminal dashboard** – blocks for Node, Core, Radio, and packets (absolute and relative per hour).
- **Low resource usage** – suitable for low‑end machines and single‑board computers.
- **Safe read‑only operation** – talks to MeshCore only via public CLI commands and does not change node configuration.

### Dashboard layout

The main dashboard consists of the following blocks:

- **Node**  
  Node name (`get name`), board type (`board`), MeshCore firmware version (`ver`), and connection parameters (port, baud rate, refresh interval).

- **Core**  
  Battery voltage, uptime (human‑readable and in seconds), error counter, and queue length.

- **Radio**  
  Noise floor, last RSSI/SNR, total TX/RX air time and their share of node uptime in percent.

- **Packets (total)**  
  Cumulative packet counters (`recv`, `sent`, `flood_*`, `direct_*`, `recv_errors`); for `recv_errors` the UI also shows its share of total receptions (`recv + recv_errors`) in percent.

- **Relative (per hour)**  
  The same counters converted to “packets per hour” based on the current uptime.

---

## Installation

### Prebuilt binaries

For typical users the easiest way is to download a prebuilt binary:

1. Go to the **Releases** section of the repository on GitHub.
2. Pick the latest release (for example, `v0.1.0`).
3. Download the archive for your platform (Windows or Linux).
4. Unpack it and put the `meshcorestat`/`meshcorestat.exe` binary somewhere in your `PATH` or run it directly.

### Building from source

Requirements:

- Rust toolchain (stable), installed via [`rustup`](https://rustup.rs/).
- A C toolchain and system dependencies for `serialport` (on Linux this typically means `libudev-dev`, which CI also installs).

Clone the repository and build:

```bash
git clone https://github.com/meshcore-dev/MeshCoreStat.git
cd MeshCoreStat
cargo build --release
```

The resulting binary will be located at:

- `target/release/meshcorestat` (Linux and other Unix‑like OS)
- `target\release\meshcorestat.exe` (Windows)

You can copy it to a directory from your `PATH` if desired.

---

## Usage

Run the binary with the desired options:

```bash
meshcorestat --port <PORT> [--baud <BAUD>] [--interval <SECS>]
```

The exact CLI syntax is available via:

```bash
meshcorestat --help
```

### Command‑line arguments

The application accepts the following arguments:

- **Port (required)**: COM port to which the MeshCore node is connected (for example, `COM12` or `/dev/ttyUSB0`).
- **Baud rate (optional)**: serial baud rate, defaults to `115200`.
- **Auto‑refresh interval, seconds (optional)**: polling interval.
  - `0` (default) — single snapshot with immediate exit; the dashboard remains visible in the terminal.
  - `> 0` — enables auto‑refresh mode.

Additional flags and options may be added over time. Always refer to `--help` for the most up‑to‑date list.

### TUI controls

In auto‑refresh mode (`--interval > 0`) the application runs an interactive terminal UI:

- **`q` or `Esc`** – exit the application.
- **Terminal resize** – the layout will automatically adapt to the new terminal size.

In one‑shot mode (`--interval 0` or omitted) the dashboard is drawn once and the application exits; the output remains in the terminal for later inspection.

### Typical usage scenarios

- **Quick node health check**  
  Show a one‑time snapshot of node state:

  ```bash
  meshcorestat --port COM12
  ```

- **Real‑time monitoring**  
  Continuously update the dashboard every 2 seconds:

  ```bash
  meshcorestat --port COM12 --interval 2
  ```

  This mode is useful when looking at radio performance, packet rates and error counters over time. To exit, press `q` or `Esc`.

---

## Example MeshCore responses

These are examples of raw CLI responses from a MeshCore node that MeshCoreStat expects and parses.

`ver`:

```text
 -> v1.13.0-295f67d (Build: 15-Feb-2026)
```

`board`:

```text
  -> Heltec V4 OLED
```

`get name`:

```text
 -> > UZAO Teply Stan 100500
```

`stats-core`:

```text
  -> {"battery_mv":4157,"uptime_secs":21059,"errors":0,"queue_len":0}
```

`stats-radio`:

```text
  -> {"noise_floor":-88,"last_rssi":-89,"last_snr":-5.25,"tx_air_secs":384,"rx_air_secs":1462}
```

`stats-packets`:

```text
  -> {"recv":3725,"sent":976,"flood_tx":953,"direct_tx":23,"flood_rx":3615,"direct_rx":110,"recv_errors":776}
```

---

## Developer guide

This section is intended for contributors and anyone building MeshCoreStat from source.

### Project structure

At a high level the project is organized as follows:

- `src/main.rs` – application entry point and high‑level wiring.
- `src/cli.rs` – command‑line interface (argument parsing and help).
- `src/serial.rs` – COM port handling and MeshCore CLI requests/responses.
- `src/model.rs` – data structures representing MeshCore statistics.
- `src/app.rs` – application state and polling logic.
- `src/ui.rs` – terminal UI built with `ratatui`.

### Building and running

Debug build:

```bash
cargo run -- --port COM12 --interval 2
```

Release build:

```bash
cargo build --release
```

### Code quality and tests

The CI pipeline runs the following checks, and it is recommended to run them locally before submitting changes:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --release
```

Please ensure that:

- The code is formatted with `cargo fmt`.
- There are no Clippy warnings.
- Tests pass on your platform.

### Releases and versioning

Releases are created from Git tags of the form `vX.Y.Z`:

- When a tag `vX.Y.Z` is pushed, the CI workflow:
  - Derives the version `X.Y.Z` from the tag.
  - Updates the `version` field in `Cargo.toml` before building.
  - Builds release binaries for supported platforms and attaches them to the GitHub Release.

The Git tag is the single source of truth for the application version.

---

## License

MeshCoreStat is distributed under the terms of the MIT License. See the `LICENSE` file for details.
