use serde::Deserialize;
use std::collections::VecDeque;

#[derive(Debug, Clone, Deserialize)]
pub struct CoreStats {
    pub battery_mv: i32,
    pub uptime_secs: u64,
    pub errors: u64,
    pub queue_len: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RadioStats {
    pub noise_floor: i32,
    pub last_rssi: i32,
    pub last_snr: f32,
    pub tx_air_secs: u64,
    pub rx_air_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PacketStats {
    pub recv: u64,
    pub sent: u64,
    pub flood_tx: u64,
    pub direct_tx: u64,
    pub flood_rx: u64,
    pub direct_rx: u64,
    pub recv_errors: u64,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub version: String,
    pub board: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MetricKind {
    BatteryMv,
    LastRssi,
    LastSnr,
    RecvPackets,
    SentPackets,
    FloodTx,
    FloodRx,
    DirectTx,
    DirectRx,
    RecvErrors,
}

impl MetricKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "battery_mv" => Some(MetricKind::BatteryMv),
            "last_rssi" => Some(MetricKind::LastRssi),
            "last_snr" => Some(MetricKind::LastSnr),
            "recv" | "recv_packets" => Some(MetricKind::RecvPackets),
            "sent" | "sent_packets" => Some(MetricKind::SentPackets),
            "flood_tx" => Some(MetricKind::FloodTx),
            "flood_rx" => Some(MetricKind::FloodRx),
            "direct_tx" => Some(MetricKind::DirectTx),
            "direct_rx" => Some(MetricKind::DirectRx),
            "recv_errors" => Some(MetricKind::RecvErrors),
            _ => None,
        }
    }
}

/// Time series for a single metric (currently unused but kept for potential future extensions).
#[derive(Debug)]
pub struct MetricSeries {
    pub values: VecDeque<f64>,
    pub max_len: usize,
}

impl MetricSeries {
    pub fn new(max_len: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(max_len),
            max_len,
        }
    }

    pub fn push(&mut self, v: f64) {
        if self.values.len() == self.max_len {
            self.values.pop_front();
        }
        self.values.push_back(v);
    }
}

#[cfg(test)]
mod tests {
    use super::{CoreStats, MetricKind, PacketStats, RadioStats};

    #[test]
    fn test_metric_from_str() {
        assert!(matches!(MetricKind::from_str("battery_mv"), Some(MetricKind::BatteryMv)));
        assert!(matches!(MetricKind::from_str("last_rssi"), Some(MetricKind::LastRssi)));
        assert!(matches!(MetricKind::from_str("recv"), Some(MetricKind::RecvPackets)));
        assert!(MetricKind::from_str("unknown").is_none());
    }

    #[test]
    fn parse_core_stats_json() {
        let json = r#"{"battery_mv":4157,"uptime_secs":21059,"errors":0,"queue_len":0}"#;
        let stats: CoreStats = serde_json::from_str(json).expect("valid core stats json");
        assert_eq!(stats.battery_mv, 4157);
        assert_eq!(stats.uptime_secs, 21059);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.queue_len, 0);
    }

    #[test]
    fn parse_radio_stats_json() {
        let json = r#"{"noise_floor":-88,"last_rssi":-89,"last_snr":-5.25,"tx_air_secs":384,"rx_air_secs":1462}"#;
        let stats: RadioStats = serde_json::from_str(json).expect("valid radio stats json");
        assert_eq!(stats.noise_floor, -88);
        assert_eq!(stats.last_rssi, -89);
        assert!((stats.last_snr + 5.25).abs() < 1e-6);
        assert_eq!(stats.tx_air_secs, 384);
        assert_eq!(stats.rx_air_secs, 1462);
    }

    #[test]
    fn parse_packet_stats_json() {
        let json = r#"{"recv":3725,"sent":976,"flood_tx":953,"direct_tx":23,"flood_rx":3615,"direct_rx":110,"recv_errors":776}"#;
        let stats: PacketStats = serde_json::from_str(json).expect("valid packet stats json");
        assert_eq!(stats.recv, 3725);
        assert_eq!(stats.sent, 976);
        assert_eq!(stats.flood_tx, 953);
        assert_eq!(stats.direct_tx, 23);
        assert_eq!(stats.flood_rx, 3615);
        assert_eq!(stats.direct_rx, 110);
        assert_eq!(stats.recv_errors, 776);
    }
}

