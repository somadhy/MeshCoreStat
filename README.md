# MeshCoreStat
Cross‑platform console application for retrieving detailed runtime statistics from a MeshCore node over a serial (COM) port.

_Русская версия: см. [README.ru.md](README.ru.md)._ 

## Overview

MeshCoreStat connects to a MeshCore node via the specified COM port and sequentially issues the following [CLI commands](https://github.com/meshcore-dev/MeshCore/blob/main/docs/cli_commands.md):
`ver`, `board`, `get name`, `stats-core`, `stats-radio`, `stats-packets`.

The responses to these commands are parsed and rendered as a compact terminal dashboard. The layout is the same for both one‑shot and auto‑refresh modes:

- **Node**: node name (`get name`), board type (`board`), MeshCore firmware version (`ver`), and connection parameters (port, baud rate, refresh interval).
- **Core**: battery voltage, uptime in a human‑readable form and in seconds, error counter, and queue length.
- **Radio**: noise floor, last RSSI/SNR, total TX/RX air time and their share of node uptime in percent.
- **Packets (total)**: cumulative packet counters (`recv`, `sent`, `flood_*`, `direct_*`, `recv_errors`).
- **Relative (per hour)**: the same counters converted to “packets per hour” based on the current uptime.

The application is optimized for a small footprint and can be used comfortably on low‑end machines and single‑board computers.

## Command‑line arguments

The application accepts the following arguments:

- **Port (required)**: COM port to which the MeshCore node is connected (for example, `COM12` or `/dev/ttyUSB0`).
- **Baud rate (optional)**: serial baud rate, defaults to `115200`.
- **Auto‑refresh interval, seconds (optional)**: polling interval.
  - `0` (default) — single snapshot with immediate exit; the dashboard remains visible in the terminal.
  - `> 0` — enables auto‑refresh mode.

Refer to the built‑in help (`--help`) for the exact CLI syntax and additional options.

## Typical usage

- **Quick node health check**:  
  `meshcorestat --port COM12`  
  Draws the dashboard (Node/Core/Radio/Packets) once and exits without clearing the screen.

- **Real‑time monitoring**:  
  `meshcorestat --port COM12 --interval 2`  
  Updates the dashboard every 2 seconds (including relative statistics in packets per hour); exit with `q` or `Esc`.

## Example MeshCore responses

`ver`:
```
 -> v1.13.0-295f67d (Build: 15-Feb-2026)
```

`board`:
```
  -> Heltec V4 OLED
```

`get name`:
```
 -> > UZAO Teply Stan 100500
```

`stats-core`:
```
  -> {"battery_mv":4157,"uptime_secs":21059,"errors":0,"queue_len":0}
```

`stats-radio`:
```
  -> {"noise_floor":-88,"last_rssi":-89,"last_snr":-5.25,"tx_air_secs":384,"rx_air_secs":1462}
```

`stats-packets`:
```
  -> {"recv":3725,"sent":976,"flood_tx":953,"direct_tx":23,"flood_rx":3615,"direct_rx":110,"recv_errors":776}
```