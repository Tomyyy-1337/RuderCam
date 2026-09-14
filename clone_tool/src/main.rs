mod utility;

use crate::utility::{
    RawDrive, TransferProgress, clone_drive_to_image, format_capacity, list_raw_drives, write_image_to_drive,
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Sparkline};
use std::io::stdout;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    CloneDrive,
    WriteImage,
}

impl Operation {
    fn title(self) -> &'static str {
        match self {
            Operation::CloneDrive => "Clone Drive To Image",
            Operation::WriteImage => "Write Image To Drive",
        }
    }
}

#[derive(Debug)]
enum AppEvent {
    Progress(TransferProgress),
    Finished(Result<(), String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Menu,
    DriveSelect,
    FilePath,
    ConfirmWrite,
    Running,
    Finished,
    Error,
}

struct App {
    screen: Screen,
    should_quit: bool,
    menu_index: usize,
    drive_index: usize,
    drives: Vec<RawDrive>,
    operation: Option<Operation>,
    file_path: String,
    progress: TransferProgress,
    speed_history: Vec<u64>,
    current_speed_bps: u64,
    last_speed_sample: Option<(Instant, u64)>,
    status_message: String,
    error_message: String,
    rx: Option<Receiver<AppEvent>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Menu,
            should_quit: false,
            menu_index: 0,
            drive_index: 0,
            drives: Vec::new(),
            operation: None,
            file_path: String::from("pi.img"),
            progress: TransferProgress {
                done: 0,
                total: 0,
                eta: None,
            },
            speed_history: Vec::new(),
            current_speed_bps: 0,
            last_speed_sample: None,
            status_message: String::new(),
            error_message: String::new(),
            rx: None,
        }
    }
}

impl App {
    fn reset_to_menu(&mut self) {
        self.screen = Screen::Menu;
        self.menu_index = 0;
        self.drive_index = 0;
        self.drives.clear();
        self.operation = None;
        self.file_path = String::from("pi.img");
        self.progress = TransferProgress {
            done: 0,
            total: 0,
            eta: None,
        };
        self.speed_history.clear();
        self.current_speed_bps = 0;
        self.last_speed_sample = None;
        self.status_message.clear();
        self.error_message.clear();
        self.rx = None;
    }

    fn refresh_drives(&mut self) {
        self.drives = list_raw_drives();
        self.drive_index = 0;
    }

    fn selected_drive(&self) -> Option<RawDrive> {
        self.drives.get(self.drive_index).cloned()
    }

    fn start_operation(&mut self) {
        let Some(operation) = self.operation else {
            return;
        };
        let Some(drive) = self.selected_drive() else {
            self.error_message = "No drive selected.".to_string();
            self.screen = Screen::Error;
            return;
        };

        let file_path = self.file_path.trim().to_string();
        if file_path.is_empty() {
            self.error_message = "Image path cannot be empty.".to_string();
            self.screen = Screen::Error;
            return;
        }

        self.progress = TransferProgress {
            done: 0,
            total: drive.capacity,
            eta: None,
        };
        self.speed_history.clear();
        self.current_speed_bps = 0;
        self.last_speed_sample = Some((Instant::now(), 0));
        self.status_message = format!("{} in progress...", operation.title());
        self.screen = Screen::Running;

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        thread::spawn(move || {
            let result = run_operation(operation, drive, file_path, &tx);
            let _ = tx.send(AppEvent::Finished(result));
        });
    }

    fn poll_worker(&mut self) {
        loop {
            let event_result = {
                let Some(rx) = &self.rx else {
                    return;
                };
                rx.try_recv()
            };

            match event_result {
                Ok(AppEvent::Progress(progress)) => {
                    self.update_speed(progress.done);
                    self.progress = progress;
                }
                Ok(AppEvent::Finished(result)) => {
                    self.rx = None;
                    self.current_speed_bps = 0;
                    self.speed_history.push(0);
                    trim_speed_history(&mut self.speed_history);
                    match result {
                        Ok(()) => {
                            self.status_message = "Operation completed successfully.".to_string();
                            self.screen = Screen::Finished;
                        }
                        Err(err) => {
                            self.error_message = err;
                            self.screen = Screen::Error;
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.rx = None;
                    self.error_message = "Worker disconnected unexpectedly.".to_string();
                    self.screen = Screen::Error;
                    break;
                }
            }
        }
    }

    fn update_speed(&mut self, done: u64) {
        let now = Instant::now();

        let Some((last_time, last_done)) = self.last_speed_sample else {
            self.last_speed_sample = Some((now, done));
            return;
        };

        let elapsed = now.saturating_duration_since(last_time);
        if elapsed < Duration::from_millis(250) {
            return;
        }

        let delta = done.saturating_sub(last_done);
        let elapsed_secs = elapsed.as_secs_f64();
        let speed_bps = if elapsed_secs > 0.0 {
            (delta as f64 / elapsed_secs) as u64
        } else {
            0
        };

        self.current_speed_bps = speed_bps;
        self.speed_history.push(speed_bps);
        trim_speed_history(&mut self.speed_history);
        self.last_speed_sample = Some((now, done));
    }
}

fn trim_speed_history(speed_history: &mut Vec<u64>) {
    // Keep a generous rolling window so wider terminals can still render a full graph.
    const MAX_POINTS: usize = 4096;
    if speed_history.len() > MAX_POINTS {
        let excess = speed_history.len() - MAX_POINTS;
        speed_history.drain(0..excess);
    }
}

fn visible_speed_points(speed_history: &[u64], panel_width: u16) -> Vec<u64> {
    let target_len = panel_width.saturating_sub(2).max(1) as usize;
    if speed_history.len() >= target_len {
        speed_history[speed_history.len() - target_len..].to_vec()
    } else {
        let mut padded = vec![0; target_len - speed_history.len()];
        padded.extend_from_slice(speed_history);
        padded
    }
}

fn run_operation(
    operation: Operation,
    drive: RawDrive,
    file_path: String,
    tx: &Sender<AppEvent>,
) -> Result<(), String> {
    let progress_sender = |progress: TransferProgress| {
        let _ = tx.send(AppEvent::Progress(progress));
    };

    match operation {
        Operation::CloneDrive => clone_drive_to_image(&drive.name, &normalized_clone_path(&file_path), drive.capacity, progress_sender),
        Operation::WriteImage => write_image_to_drive(&file_path, &drive.name, drive.capacity, progress_sender),
    }
}

fn normalized_clone_path(path: &str) -> String {
    if path.ends_with(".img") {
        path.to_string()
    } else {
        format!("{path}.img")
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::default();

    while !app.should_quit {
        app.poll_worker();
        terminal.draw(|frame| render(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            let Event::Key(key) = event::read()? else {
                continue;
            };

            if key.kind != KeyEventKind::Press {
                continue;
            }

            handle_key(&mut app, key.code);
        }
    }

    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode) {
    match app.screen {
        Screen::Menu => match code {
            KeyCode::Up => {
                if app.menu_index > 0 {
                    app.menu_index -= 1;
                }
            }
            KeyCode::Down => {
                if app.menu_index < 2 {
                    app.menu_index += 1;
                }
            }
            KeyCode::Enter => match app.menu_index {
                0 => {
                    app.operation = Some(Operation::CloneDrive);
                    app.refresh_drives();
                    if app.drives.is_empty() {
                        app.error_message = "No removable physical drives found. Run as Administrator and retry.".to_string();
                        app.screen = Screen::Error;
                    } else {
                        app.screen = Screen::DriveSelect;
                    }
                }
                1 => {
                    app.operation = Some(Operation::WriteImage);
                    app.refresh_drives();
                    if app.drives.is_empty() {
                        app.error_message = "No removable physical drives found. Run as Administrator and retry.".to_string();
                        app.screen = Screen::Error;
                    } else {
                        app.screen = Screen::DriveSelect;
                    }
                }
                2 => {
                    app.should_quit = true;
                }
                _ => {}
            },
            KeyCode::Char('q') => app.should_quit = true,
            _ => {}
        },
        Screen::DriveSelect => match code {
            KeyCode::Up => {
                if app.drive_index > 0 {
                    app.drive_index -= 1;
                }
            }
            KeyCode::Down => {
                if app.drive_index + 1 < app.drives.len() {
                    app.drive_index += 1;
                }
            }
            KeyCode::Enter => {
                app.file_path = String::from("pi.img");
                app.screen = Screen::FilePath;
            }
            KeyCode::Esc => app.reset_to_menu(),
            _ => {}
        },
        Screen::FilePath => match code {
            KeyCode::Char(ch) => app.file_path.push(ch),
            KeyCode::Backspace => {
                app.file_path.pop();
            }
            KeyCode::Enter => {
                if matches!(app.operation, Some(Operation::WriteImage)) {
                    app.screen = Screen::ConfirmWrite;
                } else {
                    app.start_operation();
                }
            }
            KeyCode::Esc => app.screen = Screen::DriveSelect,
            _ => {}
        },
        Screen::ConfirmWrite => match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => app.start_operation(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.screen = Screen::FilePath,
            _ => {}
        },
        Screen::Running => {
            if code == KeyCode::Char('q') {
                app.should_quit = true;
            }
        }
        Screen::Finished | Screen::Error => match code {
            KeyCode::Enter | KeyCode::Esc => app.reset_to_menu(),
            KeyCode::Char('q') => app.should_quit = true,
            _ => {}
        },
    }
}

fn render(frame: &mut ratatui::Frame<'_>, app: &App) {
    match app.screen {
        Screen::Menu => render_menu(frame, app),
        Screen::DriveSelect => render_drive_select(frame, app),
        Screen::FilePath => render_file_path(frame, app),
        Screen::ConfirmWrite => render_confirm(frame, app),
        Screen::Running => render_running(frame, app),
        Screen::Finished => render_status(frame, "Success", &app.status_message, Color::Green),
        Screen::Error => render_status(frame, "Error", &app.error_message, Color::Red),
    }
}

fn render_menu(frame: &mut ratatui::Frame<'_>, app: &App) {
    let items = [
        "Clone physical drive to image",
        "Write image to physical drive",
        "Quit",
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            if idx == app.menu_index {
                ListItem::new(format!("> {item}"))
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            } else {
                ListItem::new(format!("  {item}"))
            }
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .title(" Clone Tool ")
            .borders(Borders::ALL),
    );

    frame.render_widget(list, frame.area());
}

fn render_drive_select(frame: &mut ratatui::Frame<'_>, app: &App) {
    let drive_items: Vec<ListItem> = app
        .drives
        .iter()
        .enumerate()
        .map(|(idx, drive)| {
            let text = format!(
                "{}  {}",
                drive.name,
                format_capacity(drive.capacity)
            );

            if idx == app.drive_index {
                ListItem::new(format!("> {text}"))
                    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                ListItem::new(format!("  {text}"))
            }
        })
        .collect();

    let list = List::new(drive_items).block(
        Block::default()
            .title(" Select Target Drive (Enter to continue, Esc to go back) ")
            .borders(Borders::ALL),
    );

    frame.render_widget(list, frame.area());
}

fn render_file_path(frame: &mut ratatui::Frame<'_>, app: &App) {
    let op = app.operation.map(|o| o.title()).unwrap_or("Operation");
    let body = format!(
        "Operation: {op}\n\nImage path:\n{}\n\nEnter to continue, Esc to go back",
        app.file_path
    );

    let paragraph = Paragraph::new(body)
        .block(Block::default().title(" Image Path ").borders(Borders::ALL))
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, frame.area());
}

fn render_confirm(frame: &mut ratatui::Frame<'_>, app: &App) {
    let drive_name = app
        .selected_drive()
        .map(|d| d.name)
        .unwrap_or_else(|| "unknown drive".to_string());

    let text = format!(
        "This will overwrite all data on {drive_name}.\n\nPress Y to continue or N to cancel."
    );

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title(" Confirm Write ")
                .borders(Borders::ALL),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, frame.area());
}

fn render_running(frame: &mut ratatui::Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Min(3),
        ])
        .split(frame.area());

    let title = Paragraph::new(app.status_message.clone())
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    frame.render_widget(title, chunks[0]);

    let ratio = if app.progress.total > 0 {
        (app.progress.done as f64 / app.progress.total as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let label = if app.progress.total > 0 {
        format!(
            "{} / {} ({:.2}%)",
            format_capacity(app.progress.done),
            format_capacity(app.progress.total),
            ratio * 100.0
        )
    } else {
        format!("{} transferred", format_capacity(app.progress.done))
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(ratio)
        .label(label);
    frame.render_widget(gauge, chunks[1]);

    let speed_points = visible_speed_points(&app.speed_history, chunks[2].width);
    let max_speed = speed_points.iter().copied().max().unwrap_or(0);
    let speed_title = if max_speed > 0 {
        format!(" Speed (max {}) ", format_rate(max_speed))
    } else {
        " Speed ".to_string()
    };

    let speed_graph = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(speed_title))
        .style(Style::default().fg(Color::Cyan))
        .data(&speed_points);
    frame.render_widget(speed_graph, chunks[2]);

    let eta_text = app
        .progress
        .eta
        .map(format_duration)
        .unwrap_or_else(|| "--:--".to_string());

    let avg_speed_bps = if app.speed_history.is_empty() {
        0
    } else {
        (app.speed_history.iter().copied().map(u128::from).sum::<u128>()
            / app.speed_history.len() as u128) as u64
    };

    let details = Paragraph::new(format!(
        "Current: {}\nAverage: {}\nETA: {eta_text}\nPress q to quit the app",
        format_rate(app.current_speed_bps),
        format_rate(avg_speed_bps)
    ))
    .block(Block::default().borders(Borders::ALL).title(" Details "));
    frame.render_widget(details, chunks[3]);
}

fn format_rate(bytes_per_second: u64) -> String {
    if bytes_per_second == 0 {
        return "0 B/s".to_string();
    }

    const UNITS: [&str; 5] = ["B/s", "KiB/s", "MiB/s", "GiB/s", "TiB/s"];
    let mut size = bytes_per_second as f64;
    let mut unit = 0usize;

    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    format!("{size:.2} {}", UNITS[unit])
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

fn render_status(frame: &mut ratatui::Frame<'_>, title: &str, message: &str, color: Color) {
    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));

    let paragraph = Paragraph::new(format!("{message}\n\nPress Enter to return to menu"))
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(color));

    frame.render_widget(paragraph, frame.area());
}
