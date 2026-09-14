use std::{
    io,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    time::Duration,
};

use ansi_to_tui::IntoText;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Terminal,
};

use crate::upload::{self, FlashEvent};
use crate::pipeline::{read_version_number, AppEvent, TaskStatus, TASK_NAMES};

const ARCHIVE_PATH: &str = "update.tar";

const NETWORK_ERROR_MARKERS: [&str; 5] = [
    "no such host",
    "dial tcp",
    "failed to resolve source metadata",
    "no route to host",
    "temporary failure in name resolution",
];

pub struct App {
    version: String,
    statuses: [TaskStatus; TASK_NAMES.len()],
    output: Vec<OutputLine>,
    scroll: u16,
    auto_scroll: bool,
    finished: bool,
    network_error: bool,
}

pub struct OutputLine {
    line: Line<'static>,
}

enum FlashState {
    Hidden,
    Scanning,
    SelectNetwork { networks: Vec<String>, selected: usize },
    Working { log: Vec<String>, cancel: Arc<AtomicBool> },
    Finished { log: Vec<String>, result: Result<(), String> },
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

impl App {
    pub fn new() -> Self {
        Self {
            version: read_version_number().unwrap_or_else(|_| "unknown".to_string()),
            statuses: [TaskStatus::Pending; TASK_NAMES.len()],
            output: Vec::new(),
            scroll: 0,
            auto_scroll: true,
            finished: false,
            network_error: false,
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::TaskStarted(idx) => {
                self.statuses[idx] = TaskStatus::Running;
                if !self.output.is_empty() {
                    self.output.push(OutputLine {
                        line: Line::default(),
                    });
                }
                self.output.push(OutputLine {
                    line: Line::from(vec![
                        Span::styled(
                            "▶  ",
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("Step {}/{}: {}", idx + 1, TASK_NAMES.len(), TASK_NAMES[idx]),
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        ),
                    ]),
                });
            }
            AppEvent::Output(line) => {
                let lower = line.to_lowercase();
                if NETWORK_ERROR_MARKERS.iter().any(|marker| lower.contains(marker)) {
                    self.network_error = true;
                }

                let parsed_lines = match line.into_text() {
                    Ok(text) => text.lines,
                    Err(_) => {
                        vec![Line::from(Span::styled(line, Style::default().fg(Color::Gray)))]
                    }
                };

                for mut parsed in parsed_lines {
                    for span in &mut parsed.spans {
                        if span.style.fg.is_none() {
                            span.style = span.style.fg(Color::Gray);
                        }
                    }
                    self.output.push(OutputLine { line: parsed });
                }
            }
            AppEvent::TaskFinished(idx, result) => {
                match result {
                    Ok(()) => {
                        self.statuses[idx] = TaskStatus::Done;
                        self.output.push(OutputLine {
                            line: Line::from(vec![
                                Span::styled(
                                    "✓  ",
                                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!("{} completed", TASK_NAMES[idx]),
                                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                                ),
                            ]),
                        });
                    }
                    Err(e) => {
                        self.statuses[idx] = TaskStatus::Failed;
                        self.output.push(OutputLine {
                            line: Line::from(vec![
                                Span::styled(
                                    "✗  ",
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!("{} failed: {e}", TASK_NAMES[idx]),
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                                ),
                            ]),
                        });
                    }
                }
                if idx == TASK_NAMES.len() - 1 || self.has_error() {
                    self.finished = true;
                }
            }
        }
    }

    fn has_error(&self) -> bool {
        self.statuses.iter().any(|s| *s == TaskStatus::Failed)
    }
}

fn split_words_and_spaces(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut in_space = false;

    for (i, c) in text.char_indices() {
        let is_space = c.is_whitespace();
        if i == 0 {
            in_space = is_space;
            continue;
        }
        if is_space != in_space {
            result.push(&text[start..i]);
            start = i;
            in_space = is_space;
        }
    }
    if start < text.len() {
        result.push(&text[start..]);
    }
    result
}

fn wrap_line(line: Line<'static>, width: usize) -> Vec<Line<'static>> {
    if width == 0 {
        return vec![line];
    }

    let total_width: usize = line.spans.iter().map(|s| s.width()).sum();
    if total_width <= width {
        return vec![line];
    }

    let mut acc_lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_width: usize = 0;

    for span in line.spans {
        let style = span.style;
        let text = span.content;
        let words = split_words_and_spaces(&text);

        for word in words {
            let word_span = Span::styled(word.to_string(), style);
            let word_width = word_span.width();

            if current_width + word_width <= width {
                current_spans.push(word_span);
                current_width += word_width;
            } else if word_width > width {
                for ch in word.chars() {
                    let ch_str = ch.to_string();
                    let ch_span = Span::styled(ch_str, style);
                    let ch_width = ch_span.width();

                    if current_width + ch_width > width && current_width > 0 {
                        acc_lines.push(Line::from(std::mem::take(&mut current_spans)));
                        current_width = 0;
                    }
                    current_spans.push(ch_span);
                    current_width += ch_width;
                }
            } else {
                if current_width > 0 {
                    acc_lines.push(Line::from(std::mem::take(&mut current_spans)));
                    current_width = 0;
                }
                if word.trim_start().is_empty() {
                    continue;
                }
                let word_span = Span::styled(word.to_string(), style);
                let word_width = word_span.width();
                current_spans.push(word_span);
                current_width += word_width;
            }
        }
    }

    if !current_spans.is_empty() {
        acc_lines.push(Line::from(current_spans));
    }

    if acc_lines.is_empty() {
        vec![Line::default()]
    } else {
        acc_lines
    }
}

#[derive(PartialEq, Eq)]
pub enum RunOutcome {
    Quit,
    Retry,
}

pub fn run_ui(rx: std::sync::mpsc::Receiver<AppEvent>) -> io::Result<RunOutcome> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, rx);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: std::sync::mpsc::Receiver<AppEvent>,
) -> io::Result<RunOutcome> {
    let mut app = App::new();
    let mut visible_height: u16 = 20;
    let mut max_scroll: u16 = 0;

    let mut flash_state = FlashState::Hidden;
    let mut original_ssid: Option<String> = None;
    let (flash_tx, flash_rx): (_, Receiver<FlashEvent>) = mpsc::channel();

    loop {
        while let Ok(event) = rx.try_recv() {
            app.handle_event(event);
        }

        while let Ok(event) = flash_rx.try_recv() {
            match event {
                FlashEvent::NetworksFound(Ok(networks)) => {
                    if matches!(flash_state, FlashState::Scanning) {
                        flash_state = FlashState::SelectNetwork { networks, selected: 0 };
                    }
                }
                FlashEvent::NetworksFound(Err(e)) => {
                    if matches!(flash_state, FlashState::Scanning) {
                        flash_state = FlashState::Finished { log: Vec::new(), result: Err(e) };
                    }
                }
                FlashEvent::Log(line) => {
                    if let FlashState::Working { log, .. } = &mut flash_state {
                        log.push(line);
                    }
                }
                FlashEvent::Finished(result) => {
                    if let FlashState::Working { log, .. } = &flash_state {
                        flash_state = FlashState::Finished { log: log.clone(), result };
                    }
                }
            }
        }

        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(25), Constraint::Min(0)])
                .split(frame.area());

            let show_flash_hint = app.finished && !app.has_error() && matches!(flash_state, FlashState::Hidden);
            let left_chunks = if show_flash_hint {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(4)])
                    .split(chunks[0])
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0)])
                    .split(chunks[0])
            };

            let items: Vec<ListItem> = TASK_NAMES
                .iter()
                .zip(app.statuses.iter())
                .map(|(name, status)| {
                    let (marker, style) = match status {
                        TaskStatus::Pending => (" ", Style::default().fg(Color::DarkGray)),
                        TaskStatus::Running => (
                            "~",
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                        ),
                        TaskStatus::Done => ("✓", Style::default().fg(Color::Green)),
                        TaskStatus::Failed => (
                            "✗",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("[{marker}] "), style),
                        Span::styled(*name, style),
                    ]))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(format!("Tasks · v{}", app.version)));
            frame.render_widget(list, left_chunks[0]);

            if show_flash_hint {
                let hint_lines = vec![
                    Line::from(Span::styled(
                        "Press Enter or f",
                        Style::default().fg(Color::Cyan),
                    )),
                    Line::from(Span::styled(
                        "to upload this update",
                        Style::default().fg(Color::Cyan),
                    )),
                ];
                let hint = Paragraph::new(hint_lines)
                    .block(Block::default().borders(Borders::ALL).title("Update"))
                    .wrap(Wrap { trim: true });
                frame.render_widget(hint, left_chunks[1]);
            }

            let right_chunks = if app.network_error {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(3)])
                    .split(chunks[1])
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0)])
                    .split(chunks[1])
            };

            let output_block = Block::default().borders(Borders::ALL).title(if app.has_error() {
                "Output (failed, press Enter/r to retry or q to quit)"
            } else if app.finished {
                "Output (finished, q to quit, r to restart)"
            } else {
                "Output (working, q to quit, r to restart)"
            });
            let output_area = output_block.inner(right_chunks[0]);
            visible_height = output_area.height;

            let text: Vec<Line> = app
                .output
                .iter()
                .flat_map(|output_line| {
                    wrap_line(output_line.line.clone(), output_area.width as usize)
                })
                .collect();
            max_scroll = text
                .len()
                .saturating_sub(visible_height as usize)
                .min(u16::MAX as usize) as u16;
            if app.auto_scroll {
                app.scroll = max_scroll;
            } else {
                app.scroll = app.scroll.min(max_scroll);
            }
            let paragraph = Paragraph::new(text)
                .block(output_block)
                .scroll((app.scroll, 0));
            frame.render_widget(paragraph, right_chunks[0]);

            if app.network_error {
                let warning = Paragraph::new(Line::from(Span::styled(
                    "⚠ No network connection detected. Reconnect to the network and try again.",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )))
                .block(Block::default().borders(Borders::ALL))
                .wrap(Wrap { trim: true });
                frame.render_widget(warning, right_chunks[1]);
            }

            render_flash_overlay(frame, &flash_state);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Release {
                    if !matches!(flash_state, FlashState::Hidden) {
                        handle_flash_key(&mut flash_state, key.code, &flash_tx, &original_ssid);
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(RunOutcome::Quit),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(RunOutcome::Quit)
                        }
                        KeyCode::Char('f') if app.finished && !app.has_error() => {
                            original_ssid = upload::current_ssid();
                            flash_state = FlashState::Scanning;
                            upload::scan_networks_async(flash_tx.clone());
                        }
                        KeyCode::Enter if app.finished && !app.has_error() => {
                            original_ssid = upload::current_ssid();
                            flash_state = FlashState::Scanning;
                            upload::scan_networks_async(flash_tx.clone());
                        }
                        KeyCode::Enter if app.finished => return Ok(RunOutcome::Retry),
                        KeyCode::Char('r') => return Ok(RunOutcome::Retry),
                        KeyCode::Up => {
                            app.auto_scroll = false;
                            app.scroll = app.scroll.saturating_sub(1);
                        }
                        KeyCode::Down => {
                            app.scroll = app.scroll.saturating_add(1).min(max_scroll);
                        }
                        KeyCode::PageUp => {
                            app.auto_scroll = false;
                            app.scroll = app.scroll.saturating_sub(visible_height);
                        }
                        KeyCode::PageDown => {
                            app.scroll = app.scroll
                                .saturating_add(visible_height)
                                .min(max_scroll);
                        }
                        KeyCode::End => {
                            app.auto_scroll = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Advances the flash overlay state machine in response to a key press.
fn handle_flash_key(
    flash_state: &mut FlashState,
    code: KeyCode,
    flash_tx: &std::sync::mpsc::Sender<FlashEvent>,
    original_ssid: &Option<String>,
) {
    match flash_state {
        FlashState::Scanning => {
            if code == KeyCode::Esc {
                *flash_state = FlashState::Hidden;
            }
        }
        FlashState::SelectNetwork { networks, selected } => match code {
            KeyCode::Esc => *flash_state = FlashState::Hidden,
            KeyCode::Up => *selected = selected.saturating_sub(1),
            KeyCode::Down => *selected = (*selected + 1).min(networks.len().saturating_sub(1)),
            KeyCode::Char('r') => {
                *flash_state = FlashState::Scanning;
                upload::scan_networks_async(flash_tx.clone());
            }
            KeyCode::Enter => {
                if let Some(ssid) = networks.get(*selected).cloned() {
                    let cancel = upload::flash_async(flash_tx.clone(), ssid, PathBuf::from(ARCHIVE_PATH));
                    *flash_state = FlashState::Working { log: Vec::new(), cancel };
                }
            }
            _ => {}
        },
        FlashState::Working { cancel, .. } => {
            if code == KeyCode::Esc {
                cancel.store(true, Ordering::Relaxed);
                *flash_state = FlashState::Hidden;
                if let Some(ssid) = original_ssid.clone() {
                    upload::reconnect_async(ssid);
                }
            }
        }
        FlashState::Finished { .. } => {
            if matches!(code, KeyCode::Enter | KeyCode::Esc) {
                *flash_state = FlashState::Hidden;
                if let Some(ssid) = original_ssid.clone() {
                    upload::reconnect_async(ssid);
                }
            }
        }
        FlashState::Hidden => {}
    }
}

/// Renders the flash-to-pi popup on top of the main UI, if visible.
fn render_flash_overlay(frame: &mut ratatui::Frame, flash_state: &FlashState) {
    if matches!(flash_state, FlashState::Hidden) {
        return;
    }

    let area = centered_rect(70, 60, frame.area());
    let visible_height = area.height.saturating_sub(2) as usize;

    let (title, lines): (&str, Vec<Line>) = match flash_state {
        FlashState::Hidden => unreachable!(),
        FlashState::Scanning => (
            "Flash to pi",
            vec![Line::from("Scanning for WLAN networks...")],
        ),
        FlashState::SelectNetwork { networks, selected } => {
            let visible_height = visible_height.max(1);
            let offset = selected
                .saturating_sub(visible_height.saturating_sub(1))
                .min(networks.len().saturating_sub(visible_height));
            let lines = networks
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_height)
                .map(|(i, name)| {
                    if i == *selected {
                        Line::from(Span::styled(
                            format!("> {name}"),
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        ))
                    } else {
                        Line::from(format!("  {name}"))
                    }
                })
                .collect();
            ("Select a known WLAN network (r to refresh, Esc to cancel)", lines)
        }
        FlashState::Working { log, .. } => {
            let mut lines: Vec<Line> = log.iter().map(|l| Line::from(l.clone())).collect();
            lines.push(Line::from(Span::styled(
                "Working...",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "Esc to cancel",
                Style::default().fg(Color::DarkGray),
            )));
            ("Flashing update to pi", lines)
        }
        FlashState::Finished { log, result } => {
            let mut lines: Vec<Line> = log.iter().map(|l| Line::from(l.clone())).collect();
            match result {
                Ok(()) => lines.push(Line::from(Span::styled(
                    "✓ Update uploaded successfully",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ))),
                Err(e) => lines.push(Line::from(Span::styled(
                    format!("✗ Flash failed: {e}"),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ))),
            }
            lines.push(Line::default());
            lines.push(Line::from("Press Enter or Esc to close"));
            ("Flash to pi", lines)
        }
    };

    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
