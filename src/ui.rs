use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use ratatui::Terminal;

use crate::app::{build_initial_state, format_duration, tick_update};
use crate::cli::Args;

pub fn run_tui(args: Args) -> Result<()> {
    // Non-interactive mode (interval == 0): один кадр, без alternate screen,
    // чтобы вывод остался в основном буфере терминала.
    if args.interval == 0 {
        let mut stdout = io::stdout();
        // Очистим экран и поставим курсор в начало, чтобы не просвечивали предыдущие команды.
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        run_tui_inner(&mut terminal, args)
    } else {
        // Интерактивный режим с автообновлением: используем альтернативный экран и raw mode.
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = run_tui_inner(&mut terminal, args);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        res
    }
}

fn run_tui_inner(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, args: Args) -> Result<()> {
    let mut state = build_initial_state(&args)?;

    // interval == 0: единичный снимок, без автообновления и ожиданий.
    if args.interval == 0 {
        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(7), // header (title + Node with 4 строками)
                        Constraint::Length(8), // core + radio
                        Constraint::Min(5),    // packets + relative stats
                    ]
                    .as_ref(),
                )
                .split(size);

            draw_header(f, chunks[0], &state, &args);
            draw_core_and_radio(f, chunks[1], &state);
            draw_packets_and_charts(f, chunks[2], &state);
        })?;
        return Ok(());
    }

    let tick_rate = Duration::from_secs(args.interval.max(1));
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(7), // header (title + Node с 4 строками)
                        Constraint::Length(8), // core + radio
                        Constraint::Min(5),    // packets + relative stats
                    ]
                    .as_ref(),
                )
                .split(size);

            draw_header(f, chunks[0], &state, &args);
            draw_core_and_radio(f, chunks[1], &state);
            draw_packets_and_charts(f, chunks[2], &state);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if let Err(err) = tick_update(&mut state, &args.port, args.baud, args.interval.max(1)) {
                eprintln!("Tick update error: {err:?}");
            }
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn draw_header(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    state: &crate::app::AppState,
    args: &Args,
) {
    // Верхняя строка — название инструмента, по центру.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(area);

    let title = Paragraph::new(Line::from(Span::styled(
        "MeshCoreStat",
        Style::default().add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    f.render_widget(title, rows[0]);

    // Блок Node с данными узла.
    let port_line = if args.interval == 0 {
        format!("Port  : {} @ {} baud", args.port, args.baud)
    } else {
        format!(
            "Port  : {} @ {} baud, interval: {} s (press 'q' or Esc to quit)",
            args.port, args.baud, args.interval
        )
    };

    let text = vec![
        Line::from(format!("Name  : {}", state.node.name)),
        Line::from(format!("Board : {}", state.node.board)),
        Line::from(format!("FW    : {}", state.node.version)),
        Line::from(port_line),
    ];

    let block = Block::default().borders(Borders::ALL).title("Node");
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, rows[1]);
}

fn draw_core_and_radio(f: &mut Frame, area: ratatui::layout::Rect, state: &crate::app::AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let core_lines = vec![
        Line::from(format!("Battery : {} mV", state.core.battery_mv)),
        Line::from(format!(
            "Uptime  : {} ({})",
            format_duration(state.core.uptime_secs),
            state.core.uptime_secs
        )),
        Line::from(format!("Errors  : {}", state.core.errors)),
        Line::from(format!("Queue   : {}", state.core.queue_len)),
    ];
    let core_block = Block::default().borders(Borders::ALL).title("Core");
    let core_paragraph = Paragraph::new(core_lines).block(core_block);
    f.render_widget(core_paragraph, chunks[0]);

    let radio_lines = vec![
        Line::from(format!("Noise floor : {} dBm", state.radio.noise_floor)),
        Line::from(format!("Last RSSI   : {} dBm", state.radio.last_rssi)),
        Line::from(format!("Last SNR    : {} dB", state.radio.last_snr)),
        {
            let uptime = state.core.uptime_secs.max(1);
            let pct = (state.radio.tx_air_secs as f64) * 100.0 / (uptime as f64);
            Line::from(format!(
                "TX air time : {} ({}) [{:.1}% of uptime]",
                format_duration(state.radio.tx_air_secs),
                state.radio.tx_air_secs,
                pct
            ))
        },
        {
            let uptime = state.core.uptime_secs.max(1);
            let pct = (state.radio.rx_air_secs as f64) * 100.0 / (uptime as f64);
            Line::from(format!(
                "RX air time : {} ({}) [{:.1}% of uptime]",
                format_duration(state.radio.rx_air_secs),
                state.radio.rx_air_secs,
                pct
            ))
        },
    ];
    let radio_block = Block::default().borders(Borders::ALL).title("Radio");
    let radio_paragraph = Paragraph::new(radio_lines).block(radio_block);
    f.render_widget(radio_paragraph, chunks[1]);
}

fn draw_packets_and_charts(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    state: &crate::app::AppState,
) {
    // Ограничим высоту блоков, чтобы рамки не тянулись до самого низа.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(11), Constraint::Min(0)].as_ref())
        .split(area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rows[0]);

    // Absolute packet counters (left side).
    let packets_lines = vec![
        Line::from(format!("Recv        : {}", state.packets.recv)),
        Line::from(format!("Sent        : {}", state.packets.sent)),
        Line::from(format!("Flood TX    : {}", state.packets.flood_tx)),
        Line::from(format!("Direct TX   : {}", state.packets.direct_tx)),
        Line::from(format!("Flood RX    : {}", state.packets.flood_rx)),
        Line::from(format!("Direct RX   : {}", state.packets.direct_rx)),
        Line::from(format!("Recv errors : {}", state.packets.recv_errors)),
    ];
    let packets_block = Block::default()
        .borders(Borders::ALL)
        .title("Packets (total)");
    let packets_paragraph = Paragraph::new(packets_lines).block(packets_block);
    f.render_widget(packets_paragraph, chunks[0]);

    // Per-hour rates based on current counters and uptime (right side).
    let uptime_secs = state.core.uptime_secs.max(1);
    let to_per_hour = |value: u64| (value as f64) * 3600.0 / (uptime_secs as f64);

    let relative_lines = vec![
        Line::from(format!(
            "Recv/h      : {:.2}",
            to_per_hour(state.packets.recv)
        )),
        Line::from(format!(
            "Sent/h      : {:.2}",
            to_per_hour(state.packets.sent)
        )),
        Line::from(format!(
            "Flood TX/h  : {:.2}",
            to_per_hour(state.packets.flood_tx)
        )),
        Line::from(format!(
            "Flood RX/h  : {:.2}",
            to_per_hour(state.packets.flood_rx)
        )),
        Line::from(format!(
            "Direct TX/h : {:.2}",
            to_per_hour(state.packets.direct_tx)
        )),
        Line::from(format!(
            "Direct RX/h : {:.2}",
            to_per_hour(state.packets.direct_rx)
        )),
        Line::from(format!(
            "Errors/h    : {:.2}",
            to_per_hour(state.packets.recv_errors)
        )),
    ];
    let relative_block = Block::default()
        .borders(Borders::ALL)
        .title("Relative (per hour)");
    let relative_paragraph = Paragraph::new(relative_lines).block(relative_block);
    f.render_widget(relative_paragraph, chunks[1]);
}
