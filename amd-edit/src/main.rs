mod fan_speed;

use std::io;
use std::sync::Arc;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tui::backend::CrosstermBackend;
use tui::layout::*;
use tui::widgets::*;
use tui::Terminal;

use crate::fan_speed::{FanSpeed, FanSpeedState};

#[derive(gumdrop::Options)]
struct Opts {
    help: bool,
    #[options(help = "Fan speed config file")]
    fan_config: Option<String>,
}

enum Page {
    FanSpeed(FanSpeedState),
}

pub enum Dir {
    Left,
    Right,
    Up,
    Down,
}

impl Page {
    pub fn index(&self) -> usize {
        match self {
            Page::FanSpeed(_) => 0,
        }
    }
}

fn main() -> Result<(), io::Error> {
    let config: Opts = gumdrop::parse_args_default_or_exit();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_writer(Arc::new(
                std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open("/tmp/amd-edit.log")
                    .expect("Failed to open log file"),
            )),
        )
        .init();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let config = amdgpu_config::fan::load_config(
        &config
            .fan_config
            .unwrap_or_else(|| amdgpu_config::fan::DEFAULT_FAN_CONFIG_PATH.into()),
    )
    .expect("AMD GUI Fan speed config does not exists");

    let mut page = Page::FanSpeed(FanSpeedState::default());

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
                .split(size);

            let block = Block::default()
                .title("AMD GPU")
                .borders(Borders::empty())
                .title_alignment(Alignment::Center);

            let tabs = Tabs::new(vec!["Fan Speed".into(), "Monitoring".into()])
                .select(page.index())
                .block(block);

            f.render_widget(tabs, chunks[0]);

            match &mut page {
                Page::FanSpeed(ref mut state) => f.render_widget(
                    FanSpeed::default()
                        .with_values(config.speed_matrix())
                        .with_state(state),
                    chunks[1],
                ),
            }
        })?;

        if let Ok(ev) = event::read() {
            match ev {
                Event::FocusGained => {}
                Event::FocusLost => {}
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => match &mut page {
                        Page::FanSpeed(state) => state.toggle_select(),
                    },
                    dir @ (KeyCode::Right | KeyCode::Left | KeyCode::Up | KeyCode::Down) => {
                        match (dir, &mut page) {
                            (KeyCode::Left | KeyCode::Down, Page::FanSpeed(ref mut state)) => {
                                state.move_dir(Dir::Left);
                            }
                            (KeyCode::Right | KeyCode::Up, Page::FanSpeed(ref mut state)) => {
                                state.move_dir(Dir::Right);
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char(c) => match &mut page {
                        Page::FanSpeed(state) => state.text_input(c),
                    },
                    KeyCode::Backspace => match &mut page {
                        Page::FanSpeed(state) => state.erase_last(),
                    },
                    _ => {}
                },
                Event::Mouse(_mouse) => {}
                Event::Paste(_) => {}
                Event::Resize(_, _) => {}
            }
        }
    }

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
