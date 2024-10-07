use std::{
    collections::HashMap,
    error::Error,
    io::{self, Stdout},
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout},
    style::Color,
};
use ratatui::{
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::CrosstermBackend,
    widgets::{Block, Paragraph},
    Frame, Terminal,
};
use tokio::sync::RwLock;

pub struct TerminalUi {
    sacn_status: (String, Color),
    light_status: HashMap<String, (String, Color)>,
    app_status: (String, Color),
    terminal: RwLock<Terminal<CrosstermBackend<Stdout>>>,
}

impl TerminalUi {
    pub fn new() -> Self {
        let terminal = Self::setup_terminal().unwrap();
        Self {
            sacn_status: (String::new(), Color::Reset),
            light_status: HashMap::new(),
            app_status: (String::new(), Color::Reset),
            terminal: RwLock::new(terminal),
        }
    }

    fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        Ok(Terminal::new(CrosstermBackend::new(stdout))?)
    }

    pub async fn restore_terminal(&mut self) -> Result<(), Box<dyn Error>> {
        disable_raw_mode()?;
        let mut term = self.terminal.write().await;
        execute!(term.backend_mut(), LeaveAlternateScreen,)?;
        Ok(term.show_cursor()?)
    }

    pub fn set_sacn_status(&mut self, status: &str, color: Color) {
        self.sacn_status = (status.to_string(), color);
    }

    pub fn set_light_status(&mut self, id: &str, status: &str, color: Color) {
        self.light_status
            .insert(id.to_string(), (status.to_string(), color));
    }

    pub fn set_app_status(&mut self, status: &str, color: Color) {
        self.app_status = (status.to_string(), color);
    }

    pub async fn ui_loop(lock: &RwLock<Self>) {
        let mut should_exit = false;
        while !should_exit {
            let self_ref = lock.read().await;
            let mut terminal_ref = self_ref.terminal.write().await;

            let _ = terminal_ref.draw(|f| {
                self_ref.ui(f);
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
            should_exit = TerminalUi::handle_events().unwrap();
        }
    }

    fn handle_events() -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(25))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn ui(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min((self.light_status.len() + 2) as u16),
                ]
                .as_ref(),
            )
            .split(frame.area());

        let app_status_block = Block::default()
            .title("App Status")
            .borders(ratatui::widgets::Borders::ALL);
        let app_status_paragraph = Paragraph::new(self.app_status.0.as_str())
            .style(self.app_status.1)
            .block(app_status_block);
        frame.render_widget(app_status_paragraph, chunks[0]);

        let sacn_status_block = Block::default()
            .title("Sacn Status")
            .borders(ratatui::widgets::Borders::ALL);
        let sacn_status_paragraph = Paragraph::new(self.sacn_status.0.as_str())
            .style(self.sacn_status.1)
            .block(sacn_status_block);
        frame.render_widget(sacn_status_paragraph, chunks[1]);

        let light_status_block = Block::default()
            .title("Light Status")
            .borders(ratatui::widgets::Borders::ALL);
        let light_status_paragraphs: Vec<Paragraph> = self
            .light_status
            .iter()
            .map(|(id, status)| Paragraph::new(format!("{}: {}", id, status.0)).style(status.1))
            .collect();

        let light_status_inner_area = light_status_block.inner(chunks[2]);
        let light_status_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                light_status_paragraphs
                    .iter()
                    .map(|_| Constraint::Length(1))
                    .collect::<Vec<_>>(),
            )
            .split(light_status_inner_area);

        if let Some(first_light) = self.light_status.values().next() {
            if self
                .light_status
                .values()
                .all(|status| status.1 == first_light.1)
            {
                frame.render_widget(light_status_block.style(first_light.1), chunks[2]);
            } else {
                frame.render_widget(light_status_block, chunks[2]);
            }
        } else {
            frame.render_widget(light_status_block, chunks[2]);
        }

        for (i, paragraph) in light_status_paragraphs.into_iter().enumerate() {
            frame.render_widget(paragraph, light_status_layout[i]);
        }
    }
}
