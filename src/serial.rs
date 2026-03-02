use std::time::Duration;

use anyhow::{Context, Result};
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

fn debug_enabled() -> bool {
    std::env::var("MESHCORESTAT_DEBUG").as_deref() == Ok("1")
}

/// Open a serial (COM) port with the given settings.
/// Waits briefly after open so the device can stabilize (opening often toggles DTR and resets the MCU).
pub fn open_port(name: &str, baud: u32) -> Result<Box<dyn SerialPort>> {
    if debug_enabled() {
        eprintln!("[debug] Opening port `{}` at {} baud...", name, baud);
    }
    let port_info = serialport::available_ports()
        .context("failed to list serial ports")?
        .into_iter()
        .find(|p| p.port_name == name)
        .with_context(|| format!("serial port `{}` not found", name))?;
    // Match KiTTY settings: 115200, 8N1, XON/XOFF software flow control.
    let builder = serialport::new(port_info.port_name, baud)
        .data_bits(DataBits::Eight)
        .stop_bits(StopBits::One)
        .parity(Parity::None)
        .flow_control(FlowControl::Software)
        // Short read timeout so we don't wait long after response is received.
        .timeout(Duration::from_millis(100));

    let mut port = builder
        .open()
        .with_context(|| format!("failed to open serial port `{}`", name))?;

    // Try to mimic typical terminal behaviour: raise DTR/RTS so the device sees an active host.
    let _ = port.write_data_terminal_ready(true);
    let _ = port.write_request_to_send(true);
    if debug_enabled() {
        eprintln!("[debug] DTR/RTS set high, waiting 800 ms for device...");
    }
    std::thread::sleep(Duration::from_millis(800));

    Ok(port)
}

/// Send a single command and read a single-line response as String.
///
/// This helper is intended for non-JSON commands like `ver`, `board`, `get name`.
pub fn send_command_text(port: &mut dyn SerialPort, cmd: &str) -> Result<String> {
    let raw = send_command_raw(port, cmd)?;
    Ok(normalize_line(&raw))
}

/// Send a command and read raw response.
/// Reads until no more data приходит в течение таймаута порта.
pub fn send_command_raw(port: &mut dyn SerialPort, cmd: &str) -> Result<String> {
    let _ = port.clear(serialport::ClearBuffer::Input);

    let command = format!("{cmd}\r\n");
    let bytes = command.as_bytes();
    if debug_enabled() {
        eprintln!("[debug] Send ({} bytes): {:?}", bytes.len(), command);
    }
    port.write_all(bytes)
        .with_context(|| format!("failed to write command `{cmd}`"))?;
    port.flush().ok();

    std::thread::sleep(Duration::from_millis(50));

    let mut buf = Vec::new();
    let mut chunk = [0u8; 256];

    loop {
        match port.read(&mut chunk) {
            Ok(0) => continue,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                if debug_enabled() {
                    let s = String::from_utf8_lossy(&chunk[..n]);
                    eprintln!("[debug] Recv chunk ({} bytes): {:?}", n, s);
                }
            }
            Err(e) => {
                // Таймаут считаем концом ответа и не показываем системное сообщение пользователю.
                if debug_enabled() {
                    eprintln!("[debug] Read finished (treating as end of response)");
                }
                let _ = e; // suppress unused warning in release without debug
                break;
            }
        }
    }

    if buf.is_empty() {
        anyhow::bail!("no response bytes for command `{cmd}`");
    }

    let s = String::from_utf8_lossy(&buf).to_string();
    Ok(s)
}

/// Extract the response line (the one containing "->") from raw buffer; return value after "->".
/// Raw buffer may contain echoed command on first line(s), then "  -> value\r\n".
pub fn normalize_line(s: &str) -> String {
    let line = s
        .lines()
        .find(|l| l.trim().contains("->"))
        .unwrap_or_else(|| s.lines().next().unwrap_or(""));
    line.trim()
        .trim_start_matches("->")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_line;

    #[test]
    fn test_normalize_line_basic() {
        assert_eq!(normalize_line(" -> v1.13.0-295f67d (Build: 15-Feb-2026)\r\n"), "v1.13.0-295f67d (Build: 15-Feb-2026)");
        assert_eq!(normalize_line("  -> Heltec V4 OLED\n"), "Heltec V4 OLED");
        assert_eq!(normalize_line("-> something"), "something");
    }

    #[test]
    fn test_normalize_line_with_echo_and_payload() {
        let raw = "stats-core\r\n  -> {\"battery_mv\":4157,\"uptime_secs\":21059}\r\n";
        assert_eq!(
            normalize_line(raw),
            "{\"battery_mv\":4157,\"uptime_secs\":21059}"
        );
    }
}

