use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::cli::Args;
use crate::model::{CoreStats, MetricKind, MetricSeries, NodeInfo, PacketStats, RadioStats};
use crate::serial::{normalize_line, open_port, send_command_raw, send_command_text};

pub struct AppState {
    pub node: NodeInfo,
    pub core: CoreStats,
    pub radio: RadioStats,
    pub packets: PacketStats,
    pub metrics: HashMap<MetricKind, MetricSeries>,
}

impl AppState {
    pub fn new(node: NodeInfo, core: CoreStats, radio: RadioStats, packets: PacketStats) -> Self {
        Self {
            node,
            core,
            radio,
            packets,
            metrics: HashMap::new(),
        }
    }
}

pub fn run(args: Args) -> Result<()> {
    // В обоих режимах используем один и тот же TUI.
    // При interval == 0 интерфейс отрисовывается один раз и программа завершается.
    crate::ui::run_tui(args)
}

pub fn build_initial_state(args: &Args) -> Result<AppState> {
    let mut port = open_port(&args.port, args.baud)?;

    // Static info
    let version = send_command_text(&mut *port, "ver")?;
    let board = send_command_text(&mut *port, "board")?;
    let name_raw = send_command_text(&mut *port, "get name")?;
    let name = normalize_line(&name_raw).trim_start_matches('>').trim().to_string();

    let node = NodeInfo { version, board, name };

    // First stats snapshot
    let core = fetch_core_stats(&mut *port)?;
    let radio = fetch_radio_stats(&mut *port)?;
    let packets = fetch_packet_stats(&mut *port)?;

    let mut state = AppState::new(node, core, radio, packets);
    init_metric_series(&mut state, &args.metrics);

    Ok(state)
}

fn init_metric_series(state: &mut AppState, metric_names: &[String]) {
    const HISTORY_LEN: usize = 60;

    // If no metrics were specified explicitly, choose a sensible default set.
    if metric_names.is_empty() {
        let defaults = [
            MetricKind::RecvPackets,
            MetricKind::SentPackets,
        ];
        for kind in defaults {
            state
                .metrics
                .entry(kind)
                .or_insert_with(|| MetricSeries::new(HISTORY_LEN));
        }
        return;
    }

    for name in metric_names {
        if let Some(kind) = MetricKind::from_str(name) {
            state
                .metrics
                .entry(kind)
                .or_insert_with(|| MetricSeries::new(HISTORY_LEN));
        }
    }
}

pub fn tick_update(state: &mut AppState, port_name: &str, baud: u32, interval_secs: u64) -> Result<()> {
    let mut port = open_port(port_name, baud)?;

    state.core = fetch_core_stats(&mut *port)?;
    state.radio = fetch_radio_stats(&mut *port)?;
    state.packets = fetch_packet_stats(&mut *port)?;

    let dt = interval_secs.max(1) as f64;

    // Update metric series with latest per-second rates for packet counters.
    let prev = &state.packets;
    for (kind, series) in state.metrics.iter_mut() {
        let value = match kind {
            MetricKind::RecvPackets => rate_per_sec(state.packets.recv, prev.recv, dt),
            MetricKind::SentPackets => rate_per_sec(state.packets.sent, prev.sent, dt),
            MetricKind::FloodTx => rate_per_sec(state.packets.flood_tx, prev.flood_tx, dt),
            MetricKind::FloodRx => rate_per_sec(state.packets.flood_rx, prev.flood_rx, dt),
            MetricKind::DirectTx => rate_per_sec(state.packets.direct_tx, prev.direct_tx, dt),
            MetricKind::DirectRx => rate_per_sec(state.packets.direct_rx, prev.direct_rx, dt),
            MetricKind::RecvErrors => rate_per_sec(state.packets.recv_errors, prev.recv_errors, dt),
            _ => 0.0,
        };
        series.push(value);
    }

    Ok(())
}

fn rate_per_sec(current: u64, previous: u64, dt: f64) -> f64 {
    if current >= previous {
        (current - previous) as f64 / dt
    } else {
        0.0
    }
}

pub fn fetch_core_stats(port: &mut dyn serialport::SerialPort) -> Result<CoreStats> {
    let raw = send_command_raw(port, "stats-core")?;
    let line = normalize_line(&raw);
    let stats: CoreStats =
        serde_json::from_str(&line).with_context(|| format!("failed to parse stats-core JSON: {line}"))?;
    Ok(stats)
}

pub fn fetch_radio_stats(port: &mut dyn serialport::SerialPort) -> Result<RadioStats> {
    let raw = send_command_raw(port, "stats-radio")?;
    let line = normalize_line(&raw);
    let stats: RadioStats =
        serde_json::from_str(&line).with_context(|| format!("failed to parse stats-radio JSON: {line}"))?;
    Ok(stats)
}

pub fn fetch_packet_stats(port: &mut dyn serialport::SerialPort) -> Result<PacketStats> {
    let raw = send_command_raw(port, "stats-packets")?;
    let line = normalize_line(&raw);
    let stats: PacketStats =
        serde_json::from_str(&line).with_context(|| format!("failed to parse stats-packets JSON: {line}"))?;
    Ok(stats)
}

pub fn format_duration(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let minutes = (secs % 3_600) / 60;
    let seconds = secs % 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m {seconds}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use super::format_duration;
    use super::rate_per_sec;

    #[test]
    fn test_format_duration_variants() {
        assert_eq!(format_duration(5), "5s");
        assert_eq!(format_duration(75), "1m 15s");
        assert_eq!(format_duration(3_700), "1h 1m 40s");
        assert_eq!(format_duration(90_000), "1d 1h 0m 0s");
    }

    #[test]
    fn test_rate_per_sec_basic() {
        // Normal increasing counter.
        assert!((rate_per_sec(110, 100, 5.0) - 2.0).abs() < 1e-6);
        // Counter reset should not produce отрицательные скорости.
        assert_eq!(rate_per_sec(5, 10, 5.0), 0.0);
    }
}


