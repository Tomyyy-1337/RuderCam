use crate::text;

use std::{
    io,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::events::{AppEvent, Task, TaskStatus};
use crate::pipeline::{read_version_number, write_version_number};
use crate::upload::{self, FlashEvent};
use text::{parse_output_lines, wrap_line};

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
    statuses: [TaskStatus; Task::ALL.len()],
    auto_upload_status: Option<TaskStatus>,
    output: Vec<OutputLine>,
    scroll: u16,
    auto_scroll: bool,
    finished: bool,
    network_error: bool,
    parallel_active: bool,
    parallel_output: [Vec<Line<'static>>; 2],
    fail_marker: Option<usize>,
    jump_pending: bool,
}

pub struct OutputLine {
    line: Line<'static>,
}

enum FlashState {
    Hidden,
    ChangingVersion {
        input: String,
    },
    Scanning,
    SelectNetwork {
        networks: Vec<String>,
        selected: usize,
    },
    Working {
        log: Vec<String>,
        cancel: Arc<AtomicBool>,
        ssid: String,
    },
    Finished {
        log: Vec<String>,
        result: Result<(), String>,
    },
}

impl App {
    pub fn new() -> Self {
        Self {
            version: read_version_number().unwrap_or_else(|_| "unknown".to_string()),
            statuses: [TaskStatus::Pending; Task::ALL.len()],
            auto_upload_status: upload::auto_upload_enabled().then_some(TaskStatus::Pending),
            output: Vec::new(),
            scroll: 0,
            auto_scroll: true,
            finished: false,
            network_error: false,
            parallel_active: false,
            parallel_output: [Vec::new(), Vec::new()],
            fail_marker: None,
            jump_pending: false,
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::TaskStarted(task) => {
                self.statuses[task.index()] = TaskStatus::Running;
                if !self.output.is_empty() {
                    self.output.push(OutputLine {
                        line: Line::default(),
                    });
                }
                self.output.push(OutputLine {
                    line: Line::from(vec![
                        Span::styled(
                            "▶  ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(
                                "Step {}/{}: {}",
                                task.index() + 1,
                                Task::ALL.len(),
                                task.name()
                            ),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                });
            }
            AppEvent::Output(line) => {
                let lower = line.to_lowercase();
                if NETWORK_ERROR_MARKERS
                    .iter()
                    .any(|marker| lower.contains(marker))
                {
                    self.network_error = true;
                }

                for parsed in parse_output_lines(line) {
                    self.output.push(OutputLine { line: parsed });
                }
            }
            AppEvent::TaskFinished(task, result) => {
                match result {
                    Ok(()) => {
                        self.statuses[task.index()] = TaskStatus::Done;
                        self.output.push(OutputLine {
                            line: Line::from(vec![
                                Span::styled(
                                    "✓  ",
                                    Style::default()
                                        .fg(Color::Green)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!("{} completed", task.name()),
                                    Style::default()
                                        .fg(Color::Green)
                                        .add_modifier(Modifier::BOLD),
                                ),
                            ]),
                        });
                    }
                    Err(e) => {
                        self.statuses[task.index()] = TaskStatus::Failed;
                        self.output.push(OutputLine {
                            line: Line::from(vec![
                                Span::styled(
                                    "✗  ",
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!("{} failed: {e}", task.name()),
                                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                                ),
                            ]),
                        });
                        self.fail_marker = Some(self.output.len() - 1);
                        self.jump_pending = true;
                    }
                }
                if task.index() == Task::ALL.len() - 1 || self.has_error() {
                    self.finished = true;
                }
            }
            AppEvent::ParallelStarted => {
                self.parallel_active = true;
                self.parallel_output = [Vec::new(), Vec::new()];
                self.statuses[Task::Frontend.index()] = TaskStatus::Running;
                self.statuses[Task::Backend.index()] = TaskStatus::Running;
                for task in [Task::Frontend, Task::Backend] {
                    self.parallel_output[task.index()].push(Line::from(vec![
                        Span::styled(
                            "▶  ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(
                                "Step {}/{}: {}",
                                task.index() + 1,
                                Task::ALL.len(),
                                task.name()
                            ),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                }
            }
            AppEvent::ParallelOutput(task, line) => {
                for parsed in parse_output_lines(line) {
                    self.parallel_output[task.index()].push(parsed);
                }
            }
            AppEvent::ParallelTaskFinished(task, result) => {
                self.statuses[task.index()] = if result.is_ok() {
                    TaskStatus::Done
                } else {
                    TaskStatus::Failed
                };
                let (marker, label, style) = match result {
                    Ok(()) => (
                        "✓  ",
                        format!("{} completed", task.name()),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Err(error) => (
                        "✗  ",
                        format!("{} failed: {error}", task.name()),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                };
                self.parallel_output[task.index()].push(Line::from(vec![
                    Span::styled(marker, style),
                    Span::styled(label, style),
                ]));
            }
            AppEvent::ParallelFinished => {
                self.parallel_active = false;
            }
        }
    }

    fn has_error(&self) -> bool {
        self.statuses.iter().any(|s| *s == TaskStatus::Failed)
    }
}

fn task_border_style(status: TaskStatus) -> Style {
    let color = match status {
        TaskStatus::Pending => Color::White,
        TaskStatus::Running => Color::Yellow,
        TaskStatus::Done => Color::Green,
        TaskStatus::Failed => Color::Red,
    };
    Style::default().fg(color)
}

#[derive(PartialEq, Eq)]
pub enum RunOutcome {
    Quit,
    Retry,
}

pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: std::sync::mpsc::Receiver<AppEvent>,
) -> io::Result<RunOutcome> {
    let mut app = App::new();
    let mut visible_height: u16 = 20;
    let mut max_scroll: u16 = 0;

    let mut flash_state = FlashState::Hidden;
    let mut original_ssid: Option<String> = None;
    let mut auto_upload_pending = false;
    let (flash_tx, flash_rx): (_, Receiver<FlashEvent>) = mpsc::channel();

    loop {
        let was_finished = app.finished;
        while let Ok(event) = rx.try_recv() {
            app.handle_event(event);
        }

        if !was_finished && app.finished && !app.has_error() && upload::auto_upload_enabled() {
            auto_upload_pending = true;
        }

        if auto_upload_pending && matches!(flash_state, FlashState::Hidden) {
            auto_upload_pending = false;
            app.auto_upload_status = Some(TaskStatus::Running);
            start_upload(&mut flash_state, &mut original_ssid, &flash_tx, true);
        }

        while let Ok(event) = flash_rx.try_recv() {
            match event {
                FlashEvent::NetworksFound(Ok(networks)) => {
                    if matches!(flash_state, FlashState::Scanning) {
                        flash_state = FlashState::SelectNetwork {
                            networks,
                            selected: 0,
                        };
                    }
                }
                FlashEvent::NetworksFound(Err(e)) => {
                    if matches!(flash_state, FlashState::Scanning) {
                        flash_state = FlashState::Finished {
                            log: Vec::new(),
                            result: Err(e),
                        };
                    }
                }
                FlashEvent::Log(line) => {
                    if let FlashState::Working { log, .. } = &mut flash_state {
                        log.push(line);
                    }
                }
                FlashEvent::Finished(result) => {
                    if let FlashState::Working { log, ssid, .. } = &flash_state {
                        if result.is_ok() {
                            upload::remember_last_network(ssid);
                        }
                        if app.auto_upload_status == Some(TaskStatus::Running) {
                            app.auto_upload_status = Some(if result.is_ok() {
                                TaskStatus::Done
                            } else {
                                TaskStatus::Failed
                            });
                        }
                        flash_state = FlashState::Finished {
                            log: log.clone(),
                            result,
                        };
                    }
                }
            }
        }

        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(24), Constraint::Min(0)])
                .split(frame.area());

            let show_flash_hint =
                app.finished && !app.has_error() && matches!(flash_state, FlashState::Hidden);

            let mut items: Vec<ListItem> = Task::ALL
                .iter()
                .zip(app.statuses.iter())
                .map(|(task, status)| {
                    let (marker, style) = match status {
                        TaskStatus::Pending => (" ", Style::default().fg(Color::DarkGray)),
                        TaskStatus::Running => (
                            "~",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        TaskStatus::Done => ("✓", Style::default().fg(Color::Green)),
                        TaskStatus::Failed => (
                            "✗",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("[{marker}] "), style),
                        Span::styled(task.name(), style),
                    ]))
                })
                .collect();

            let (marker, style) = match app.auto_upload_status {
                None => (
                    "-",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ),
                Some(TaskStatus::Pending) => (" ", Style::default().fg(Color::DarkGray)),
                Some(TaskStatus::Running) => (
                    "~",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Some(TaskStatus::Done) => ("✓", Style::default().fg(Color::Green)),
                Some(TaskStatus::Failed) => (
                    "✗",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            };
            let ssid = upload::last_network().unwrap_or_else(|| "No known SSID".to_string());
            items.push(ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("[{marker}] "), style),
                    Span::styled("Upload to", style),
                ]),
                Line::from(Span::styled(format!("    {ssid}"), style)),
            ]));

            let tasks_block = Block::default()
                .borders(Borders::ALL)
                .title(format!("Tasks · v{}", app.version));
            let hint_height = if show_flash_hint { 2 } else { 1 };
            let task_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(hint_height)])
                .split(tasks_block.inner(chunks[0]));
            frame.render_widget(tasks_block, chunks[0]);
            frame.render_widget(List::new(items), task_chunks[0]);

            let auto_upload_hint = match upload::last_network() {
                Some(_) => "a: toggle auto upload",
                None => "auto upload unavailable",
            };
            let mut hint_lines = if show_flash_hint {
                vec![Line::from(Span::styled(
                    "f: upload to pi",
                    Style::default().fg(Color::DarkGray),
                ))]
            } else {
                Vec::new()
            };
            hint_lines.push(Line::from(Span::styled(
                auto_upload_hint,
                Style::default().fg(Color::DarkGray),
            )));
            frame.render_widget(Paragraph::new(hint_lines), task_chunks[1]);

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

            if app.parallel_active {
                let panes = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(right_chunks[0]);

                for (task, title) in [(Task::Frontend, "Frontend"), (Task::Backend, "Backend")] {
                    let pane_idx = task.index();
                    let pane_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(task_border_style(app.statuses[pane_idx]))
                        .title(title);
                    let pane_area = pane_block.inner(panes[pane_idx]);
                    let text: Vec<Line> = app.parallel_output[pane_idx]
                        .iter()
                        .flat_map(|line| wrap_line(line.clone(), pane_area.width as usize))
                        .collect();
                    let pane_scroll = text
                        .len()
                        .saturating_sub(pane_area.height as usize)
                        .min(u16::MAX as usize) as u16;
                    let paragraph = Paragraph::new(text)
                        .block(pane_block)
                        .scroll((pane_scroll, 0));
                    frame.render_widget(paragraph, panes[pane_idx]);
                }
                visible_height = right_chunks[0].height;
            } else {
                let output_border = if app.has_error() {
                    Color::Red
                } else if app.finished {
                    Color::White
                } else {
                    Color::Yellow
                };
                let output_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(output_border))
                    .title(if app.has_error() {
                        "Output (failed, press Enter/r to retry or q to quit)"
                    } else {
                        "Output (q: quit, r: restart, v: version number)"
                    });
                let output_area = output_block.inner(right_chunks[0]);
                visible_height = output_area.height;

                let mut fail_marker_end: Option<usize> = None;
                let mut text: Vec<Line> = Vec::new();
                for (i, output_line) in app.output.iter().enumerate() {
                    text.extend(wrap_line(
                        output_line.line.clone(),
                        output_area.width as usize,
                    ));
                    if Some(i) == app.fail_marker {
                        fail_marker_end = Some(text.len());
                    }
                }
                max_scroll = text
                    .len()
                    .saturating_sub(visible_height as usize)
                    .min(u16::MAX as usize) as u16;
                if app.jump_pending {
                    if let Some(end) = fail_marker_end {
                        app.auto_scroll = false;
                        let scroll_target = end.saturating_sub(visible_height as usize);
                        app.scroll = (scroll_target as u16).min(max_scroll);
                    }
                    app.jump_pending = false;
                }
                if app.auto_scroll {
                    app.scroll = max_scroll;
                } else {
                    app.scroll = app.scroll.min(max_scroll);
                }
                let paragraph = Paragraph::new(text)
                    .block(output_block)
                    .scroll((app.scroll, 0));
                frame.render_widget(paragraph, right_chunks[0]);
            }

            if app.network_error {
                let warning = Paragraph::new(Line::from(Span::styled(
                    "No network connection detected. Reconnect to the network and try again.",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )))
                .block(Block::default().borders(Borders::ALL))
                .wrap(Wrap { trim: true });
                frame.render_widget(warning, right_chunks[1]);
            }

            render_flash_overlay(frame, right_chunks[0], &flash_state);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Release {
                    if key.code == KeyCode::Char('a') && upload::last_network().is_some() {
                        app.auto_upload_status =
                            upload::toggle_auto_upload().then_some(TaskStatus::Pending);
                        continue;
                    }

                    if !matches!(flash_state, FlashState::Hidden) {
                        if handle_flash_key(
                            &mut flash_state,
                            key.code,
                            &flash_tx,
                            &original_ssid,
                            &mut app.version,
                        ) {
                            return Ok(RunOutcome::Retry);
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(RunOutcome::Quit),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(RunOutcome::Quit);
                        }
                        KeyCode::Char('f') if app.finished && !app.has_error() => {
                            start_upload(&mut flash_state, &mut original_ssid, &flash_tx, false);
                        }
                        KeyCode::Char('v') => {
                            flash_state = FlashState::ChangingVersion {
                                input: app.version.clone(),
                            };
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
                            app.scroll = app.scroll.saturating_add(visible_height).min(max_scroll);
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

fn start_upload(
    flash_state: &mut FlashState,
    original_ssid: &mut Option<String>,
    flash_tx: &std::sync::mpsc::Sender<FlashEvent>,
    use_last_network: bool,
) {
    *original_ssid = upload::current_ssid();
    if use_last_network {
        if let Some(ssid) = upload::last_network() {
            let cancel =
                upload::flash_async(flash_tx.clone(), ssid.clone(), PathBuf::from(ARCHIVE_PATH));
            *flash_state = FlashState::Working {
                log: Vec::new(),
                cancel,
                ssid,
            };
            return;
        }
    }
    *flash_state = FlashState::Scanning;
    upload::scan_networks_async(flash_tx.clone(), false);
}

/// Advances the flash overlay state machine in response to a key press.
fn handle_flash_key(
    flash_state: &mut FlashState,
    code: KeyCode,
    flash_tx: &std::sync::mpsc::Sender<FlashEvent>,
    original_ssid: &Option<String>,
    version: &mut String,
) -> bool {
    let mut restart = false;

    match flash_state {
        FlashState::ChangingVersion { input } => match code {
            KeyCode::Esc => *flash_state = FlashState::Hidden,
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Char(character) if character.is_ascii_digit() || character == '.' => {
                input.push(character);
            }
            KeyCode::Enter => {
                let normalized = input.trim_start_matches('v');
                if !normalized.is_empty()
                    && normalized
                        .chars()
                        .all(|character| character.is_ascii_digit() || character == '.')
                    && write_version_number(normalized).is_ok()
                {
                    *version = normalized.to_string();
                    *flash_state = FlashState::Hidden;
                    restart = true;
                }
            }
            _ => {}
        },
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
                upload::scan_networks_async(flash_tx.clone(), true);
            }
            KeyCode::Enter => {
                if let Some(ssid) = networks.get(*selected).cloned() {
                    let cancel = upload::flash_async(
                        flash_tx.clone(),
                        ssid.clone(),
                        PathBuf::from(ARCHIVE_PATH),
                    );
                    *flash_state = FlashState::Working {
                        log: Vec::new(),
                        cancel,
                        ssid,
                    };
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

    restart
}

fn render_flash_overlay(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    flash_state: &FlashState,
) {
    if matches!(flash_state, FlashState::Hidden) {
        return;
    }

    let visible_height = area.height.saturating_sub(2) as usize;

    let (title, lines, border_color): (&str, Vec<Line>, Color) = match flash_state {
        FlashState::Hidden => unreachable!(),
        FlashState::ChangingVersion { input } => (
            "Change version (Enter to save, Esc to cancel)",
            vec![
                Line::from(format!("Version: {input}")),
                Line::default(),
                Line::from(Span::styled(
                    "Use numbers and dots, for example 1.2.4",
                    Style::default().fg(Color::DarkGray),
                )),
            ],
            Color::White,
        ),
        FlashState::Scanning => (
            "Select WLAN network (r to refresh, Esc to cancel)",
            vec![Line::from("Scanning for WLAN networks...")],
            Color::White,
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
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else {
                        Line::from(format!("  {name}"))
                    }
                })
                .collect();
            (
                "Select WLAN network (r to refresh, Esc to cancel)",
                lines,
                Color::White,
            )
        }
        FlashState::Working { log, .. } => {
            let mut lines: Vec<Line> = log.iter().map(|l| Line::from(l.clone())).collect();
            lines.push(Line::from(Span::styled(
                "Working...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "Esc to cancel",
                Style::default().fg(Color::DarkGray),
            )));
            ("Uploading to pi", lines, Color::Yellow)
        }
        FlashState::Finished { log, result } => {
            let mut lines: Vec<Line> = log.iter().map(|l| Line::from(l.clone())).collect();
            match result {
                Ok(()) => lines.push(Line::from(Span::styled(
                    "✓ Update uploaded successfully",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ))),
                Err(e) => lines.push(Line::from(Span::styled(
                    format!("✗ Flash failed: {e}"),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ))),
            }
            lines.push(Line::default());
            lines.push(Line::from("Connection to pi remains active until closed"));
            lines.push(Line::from("Press Enter or Esc to close"));
            (
                "Upload finished",
                lines,
                if result.is_ok() {
                    Color::Cyan
                } else {
                    Color::Red
                },
            )
        }
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}
