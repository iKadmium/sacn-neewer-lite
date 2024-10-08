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

use crate::terminal_status::TerminalStatus;

pub struct TerminalUi {
    sacn_status: TerminalStatus,
    light_status: HashMap<String, TerminalStatus>,
    app_status: TerminalStatus,
    terminal: RwLock<Terminal<CrosstermBackend<Stdout>>>,
}

impl TerminalUi {
    pub fn new() -> Self {
        let terminal = Self::setup_terminal().unwrap();
        Self {
            sacn_status: TerminalStatus::new(),
            light_status: HashMap::new(),
            app_status: TerminalStatus::new(),
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
        self.sacn_status.color = color;
        self.sacn_status.status = status.to_string();
    }

    pub fn add_sacn_event(&mut self) {
        self.sacn_status.event_counter.increment();
    }

    pub fn set_light_status(&mut self, id: &str, status: &str, color: Color) {
        let status_obj = self
            .light_status
            .entry(id.to_string())
            .or_insert(TerminalStatus::new());

        status_obj.color = color;
        status_obj.status = status.to_string();
    }

    pub fn add_light_event(&mut self, id: &str) {
        let status_obj = self
            .light_status
            .entry(id.to_string())
            .or_insert(TerminalStatus::new());

        status_obj.event_counter.increment();
    }

    pub fn set_app_status(&mut self, status: &str, color: Color) {
        self.app_status.color = color;
        self.app_status.status = status.to_string();
    }

    pub async fn ui_loop(lock: &RwLock<Self>) {
        let mut should_exit = false;
        while !should_exit {
            lock.write().await.update_sparklines();

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

    fn update_sparklines(&mut self) {
        // Check and clear statuses if needed
        if self.app_status.event_counter.should_clear() {
            self.app_status.event_counter.clear();
        }

        if self.sacn_status.event_counter.should_clear() {
            self.sacn_status.event_counter.clear();
        }

        for status in self.light_status.values_mut() {
            if status.event_counter.should_clear() {
                status.event_counter.clear();
            }
        }
    }

    fn ui(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(5),
                    Constraint::Min((self.light_status.len() * 2 + 2) as u16),
                ]
                .as_ref(),
            )
            .split(frame.area());

        let app_status_block = Block::default()
            .title("App Status")
            .borders(ratatui::widgets::Borders::ALL);
        let app_status_paragraph = Paragraph::new(self.app_status.status.as_str())
            .style(self.app_status.color)
            .block(app_status_block);
        frame.render_widget(app_status_paragraph, chunks[0]);

        let sacn_status_block = Block::default()
            .title("Sacn")
            .borders(ratatui::widgets::Borders::ALL);
        let sacn_status_paragraph = Paragraph::new(self.sacn_status.status.as_str())
            .style(self.sacn_status.color)
            .block(sacn_status_block);
        frame.render_widget(sacn_status_paragraph, chunks[1]);

        // Adding sparkline for sacn status
        let sacn_sparkline = ratatui::widgets::Sparkline::default()
            .data(&self.sacn_status.event_counter.get_history().as_slices().0)
            .style(self.sacn_status.color);
        frame.render_widget(
            sacn_sparkline,
            chunks[1].inner(ratatui::layout::Margin {
                vertical: 2,
                horizontal: 1,
            }),
        );

        let light_status_block = Block::default()
            .title("Lights")
            .borders(ratatui::widgets::Borders::ALL);
        let light_status_inner_area = light_status_block.inner(chunks[2]);
        let light_status_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (0..self.light_status.len() * 2)
                    .map(|i| {
                        if i % 2 == 0 {
                            Constraint::Length(1)
                        } else {
                            Constraint::Length(1)
                        }
                    })
                    .collect::<Vec<_>>(),
            )
            .split(light_status_inner_area);

        frame.render_widget(light_status_block, chunks[2]);

        for (i, (_id, status)) in self.light_status.iter().enumerate() {
            let paragraph_index = i * 2;
            let sparkline_index = paragraph_index + 1;

            let paragraph =
                Paragraph::new(format!("{}: {}", _id, status.status)).style(status.color);
            frame.render_widget(paragraph, light_status_layout[paragraph_index]);

            // Adding sparkline for each light status
            let sparkline = ratatui::widgets::Sparkline::default()
                .data(status.event_counter.get_history().as_slices().0)
                .style(status.color);
            frame.render_widget(sparkline, light_status_layout[sparkline_index]);
        }
    }
}
