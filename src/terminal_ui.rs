use std::{
    collections::HashMap,
    error::Error,
    io::{self, Stdout},
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
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
    light_color: HashMap<String, Color>,
    app_status: TerminalStatus,
    terminal: RwLock<Terminal<CrosstermBackend<Stdout>>>,
}

impl TerminalUi {
    pub fn new() -> Self {
        let terminal = Self::setup_terminal().unwrap();
        Self {
            sacn_status: TerminalStatus::new(),
            light_status: HashMap::new(),
            light_color: HashMap::new(),
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

    pub fn set_light_color(&mut self, id: &str, color: Color) {
        self.light_color.insert(id.to_string(), color);
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

            should_exit = TerminalUi::handle_events().unwrap();
        }
    }

    fn handle_events() -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(1))? {
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

    fn render_status(title: &str, status: &TerminalStatus, frame: &mut Frame, area: Rect) {
        let app_status_block = Block::default()
            .title(title)
            .borders(ratatui::widgets::Borders::ALL);
        let app_status_paragraph = Paragraph::new(status.status.as_str())
            .style(status.color)
            .block(app_status_block);
        frame.render_widget(app_status_paragraph, area);
    }

    fn render_status_with_sparkline(
        title: &str,
        status: &TerminalStatus,
        frame: &mut Frame,
        area: Rect,
    ) {
        let status_block = Block::default()
            .title(title)
            .borders(ratatui::widgets::Borders::ALL)
            .style(status.color);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(status_block.inner(area));
        frame.render_widget(status_block, area);

        let paragraph = Paragraph::new(status.status.as_str()).style(status.color);
        frame.render_widget(paragraph, chunks[0]);

        // Adding sparkline for each light status
        let data = status.event_counter.get_as_vec();
        let sparkline = ratatui::widgets::Sparkline::default()
            .data(data.as_slice())
            .style(Color::Green);
        frame.render_widget(sparkline, chunks[1]);

        let rate_paragraph = Paragraph::new(format!(
            "Rate: {} events/s",
            status.event_counter.get_last_history()
        ));
        frame.render_widget(rate_paragraph, chunks[2]);
    }

    fn render_status_with_sparkline_and_color(
        title: &str,
        status: &TerminalStatus,
        color: &Color,
        frame: &mut Frame,
        area: Rect,
    ) {
        let status_block = Block::default()
            .title(title)
            .borders(ratatui::widgets::Borders::ALL)
            .style(status.color);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(5), Constraint::Min(20)].as_ref())
            .split(status_block.inner(area));

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(status_block.inner(horizontal_chunks[1]));
        frame.render_widget(status_block, area);

        let color_block = Block::default()
            .bg(*color)
            .border_style(*color)
            .borders(ratatui::widgets::Borders::ALL);
        let color_paragraph = Paragraph::new("▀").style(*color).block(color_block);

        frame.render_widget(color_paragraph, horizontal_chunks[0]);

        let paragraph = Paragraph::new(status.status.as_str()).style(status.color);
        frame.render_widget(paragraph, vertical_chunks[0]);

        // Adding sparkline for each light status
        let data = status.event_counter.get_as_vec();
        let sparkline = ratatui::widgets::Sparkline::default()
            .data(data.as_slice())
            .style(Color::Green);
        frame.render_widget(sparkline, vertical_chunks[1]);

        let rate_paragraph = Paragraph::new(format!(
            "Rate: {} events/s",
            status.event_counter.get_last_history()
        ));
        frame.render_widget(rate_paragraph, vertical_chunks[2]);
    }

    fn render_status_set(
        title: &str,
        statuses: &HashMap<String, TerminalStatus>,
        colors: &HashMap<String, Color>,
        frame: &mut Frame,
        area: Rect,
    ) {
        let color = if statuses
            .values()
            .all(|status| status.color == statuses.values().next().unwrap().color)
        {
            statuses
                .values()
                .next()
                .map_or(Color::Reset, |status| status.color)
        } else {
            Color::Reset
        };

        let status_block = Block::default()
            .title(title)
            .borders(ratatui::widgets::Borders::ALL)
            .style(color);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (0..statuses.len())
                    .map(|_i| Constraint::Length(7))
                    .collect::<Vec<_>>(),
            )
            .split(status_block.inner(area));

        frame.render_widget(status_block, area);

        for (i, (id, status)) in statuses.iter().enumerate() {
            let color = colors.get(id).unwrap_or(&Color::Black);
            Self::render_status_with_sparkline_and_color(id, status, color, frame, chunks[i]);
        }
    }

    fn ui(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(7),
                    Constraint::Length((self.light_status.len() * 7 + 2) as u16),
                ]
                .as_ref(),
            )
            .split(frame.area());

        Self::render_status("App", &self.app_status, frame, chunks[0]);
        Self::render_status_with_sparkline("sACN", &self.sacn_status, frame, chunks[1]);
        Self::render_status_set(
            "Lights",
            &self.light_status,
            &self.light_color,
            frame,
            chunks[2],
        );
    }
}
