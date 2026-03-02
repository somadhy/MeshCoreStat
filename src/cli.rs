use clap::{ArgAction, Parser};

/// Command-line arguments for meshcorestat.
#[derive(Debug, Parser)]
#[command(name = "meshcorestat")]
#[command(about = "MeshCore node statistics viewer over a serial (COM) port")]
pub struct Args {
    /// Serial port name (e.g. COM3, /dev/ttyUSB0)
    #[arg(short, long)]
    pub port: String,

    /// Baud rate for the serial connection
    #[arg(long, default_value_t = 115_200)]
    pub baud: u32,

    /// Auto-refresh interval in seconds (0 - no auto-refresh)
    #[arg(short, long, default_value_t = 0)]
    pub interval: u64,

    /// Metrics to plot as histograms in TUI (only used when interval > 0)
    ///
    /// Example: --metrics battery_mv --metrics last_rssi
    #[arg(long, action = ArgAction::Append)]
    pub metrics: Vec<String>,

    /// Print debug info (sent/received bytes) to stderr
    #[arg(long)]
    pub debug: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}

