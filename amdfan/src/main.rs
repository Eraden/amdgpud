use amdgpu_config::fan::{MatrixPoint, DEFAULT_FAN_CONFIG_PATH};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gumdrop::Options;
use std::path::PathBuf;
use std::time::Instant;
use std::{io, time::Duration};
use tui::backend::Backend;
use tui::style::{Color, Modifier, Style};
use tui::symbols::Marker;
use tui::{backend::CrosstermBackend, layout::*, widgets::*, Frame, Terminal};

#[derive(Options)]
struct Opts {
    #[options(help = "Help message")]
    help: bool,
    #[options(help = "Config file location")]
    config: Option<PathBuf>,
}

struct App<'a> {
    config: amdgpu_config::fan::Config,
    events: Vec<(&'a str, &'a str)>,
    table_state: TableState,
    selected_point: Option<usize>,
}

impl<'a> App<'a> {
    pub fn new(config: amdgpu_config::fan::Config) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Self {
            config,
            events: vec![],
            table_state,
            selected_point: None,
        }
    }

    /// Rotate through the event list.
    /// This only exists to simulate some kind of "progress"
    fn on_tick(&mut self) {
        if self.events.is_empty() {
            return;
        }
        let event = self.events.remove(0);
        self.events.push(event);
    }
}

fn main() -> Result<(), io::Error> {
    let opts: Opts = Options::parse_args_default_or_exit();
    let config_path = opts
        .config
        .as_deref()
        .and_then(|p| p.to_str())
        .unwrap_or(DEFAULT_FAN_CONFIG_PATH);
    let config = amdgpu_config::fan::load_config(config_path).unwrap();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let app = App::new(config);
    run_app(&mut terminal, app, tick_rate).unwrap();

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Status::Exit = handle_event(&mut app)? {
                return Ok(());
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

enum Status {
    Continue,
    Exit,
}

fn handle_event(app: &mut App) -> io::Result<Status> {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('q') => return Ok(Status::Exit),
            KeyCode::Char('w') => {
                let s = toml::to_string_pretty(&app.config).unwrap();
                std::fs::write(app.config.path(), &s)?;
            }
            KeyCode::Char(' ') => {
                if let Some(index) = app.table_state.selected() {
                    app.selected_point = Some(index);
                }
            }
            KeyCode::Down => match app.selected_point {
                Some(index) => {
                    change_value(app, index, -1.0, y, y_mut);
                }
                None => match app.table_state.selected() {
                    Some(index) => {
                        if index + 1 < app.config.speed_matrix().len() {
                            app.table_state.select(Some(index + 1));
                        }
                    }
                    None => {
                        app.table_state.select(Some(0));
                    }
                },
            },
            KeyCode::Up => match app.selected_point.clone() {
                Some(index) => {
                    change_value(app, index, 1.0, y, y_mut);
                }
                None => {
                    if let Some(prev) = app
                        .table_state
                        .selected()
                        .and_then(|idx| idx.checked_sub(1))
                    {
                        app.table_state.select(Some(prev));
                    }
                }
            },
            KeyCode::Left => {
                if let Some(index) = app.selected_point.clone() {
                    change_value(app, index, -1.0, x, x_mut);
                }
            }
            KeyCode::Right => {
                if let Some(index) = app.selected_point.clone() {
                    change_value(app, index, 1.0, x, x_mut);
                }
            }
            _ => {}
        }
    }
    Ok(Status::Continue)
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(90), Constraint::Length(4)].as_ref())
        .split(f.size());

    {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(100)].as_ref())
            .split(main_chunks[0]);

        {
            let rows = app
                .config
                .speed_matrix()
                .iter()
                .enumerate()
                .map(|(idx, point)| {
                    Row::new(vec![format!("{}", y(point)), format!("{}", x(point))]).style(
                        Style::default().fg(if Some(idx) == app.selected_point {
                            Color::Yellow
                        } else {
                            Color::White
                        }),
                    )
                });
            let table = Table::new(rows)
                .block(Block::default())
                .header(
                    Row::new(vec!["Speed", "Temp"])
                        .style(Style::default().fg(Color::Yellow))
                        .bottom_margin(1),
                )
                .widths(&[Constraint::Length(5), Constraint::Length(5)])
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("*")
                .column_spacing(1);
            f.render_stateful_widget(table, chunks[0], &mut app.table_state);
        }

        {
            let points = app
                .config
                .speed_matrix()
                .iter()
                .map(|point| (x(point), y(point)))
                .collect::<Vec<_>>();
            let dataset = vec![Dataset::default()
                .data(&points)
                .name("Temp/Speed")
                .graph_type(GraphType::Scatter)
                .marker(Marker::Dot)
                .style(Style::default().fg(Color::White))];
            let chart = Chart::new(dataset)
                .block(
                    Block::default()
                        .style(Style::default().fg(Color::White))
                        .border_style(Style::default().fg(Color::White))
                        .borders(Borders::all()),
                )
                .style(Style::default())
                .x_axis(Axis::default().title("Speed").bounds([0.0, 100.0]))
                .y_axis(Axis::default().title("Temp").bounds([0.0, 100.0]));
            f.render_widget(chart, chunks[1]);
        }

        {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(main_chunks[1]);

            f.render_widget(Paragraph::new("q to exit"), chunks[0]);
            f.render_widget(Paragraph::new("Move up/down to choose setting"), chunks[1]);
            f.render_widget(
                Paragraph::new("Press SPACE to select and up/down/left/right to change values"),
                chunks[2],
            );
            f.render_widget(Paragraph::new("w to save"), chunks[3]);
        }
    }
}

fn change_value<Read, Write>(app: &mut App, index: usize, change: f64, read: Read, write: Write)
where
    Read: FnOnce(&MatrixPoint) -> f64 + Copy,
    Write: FnOnce(&mut MatrixPoint) -> &mut f64,
{
    let prev = index
        .checked_sub(1)
        .and_then(|i| app.config.speed_matrix().get(i))
        .map(|v| read(v))
        .unwrap_or(0.0);
    let next = app
        .config
        .speed_matrix()
        .get(index + 1)
        .map(|v| read(v))
        .unwrap_or(100.0);
    let current = app
        .config
        .speed_matrix()
        .get(index)
        .map(read)
        .unwrap_or_default();

    if (prev..=next).contains(&(current + change)) {
        *app.config
            .speed_matrix_mut()
            .get_mut(index)
            .map(write)
            .unwrap() += change;
    }
}

fn y_mut(p: &mut MatrixPoint) -> &mut f64 {
    &mut p.speed
}

fn x_mut(p: &mut MatrixPoint) -> &mut f64 {
    &mut p.temp
}

fn y(p: &MatrixPoint) -> f64 {
    p.speed
}

fn x(p: &MatrixPoint) -> f64 {
    p.temp
}
