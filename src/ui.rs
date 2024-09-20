use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Position},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use std::{
    io::{stdout, Stdout},
    time::Duration,
};

pub struct ChatUI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    input: String,
    messages: Vec<(String, String)>,
    input_mode: InputMode,
    spinner: Spinner,
}

#[derive(PartialEq, Clone, Copy)]
enum InputMode {
    Normal,
    Editing,
    Waiting,
}

struct Spinner {
    frames: Vec<char>,
    current: usize,
}

impl Spinner {
    fn new() -> Self {
        Spinner {
            frames: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            current: 0,
        }
    }

    fn next(&mut self) -> char {
        let char = self.frames[self.current];
        self.current = (self.current + 1) % self.frames.len();

        char
    }
}

#[derive(PartialEq)]
pub enum Action {
    CancelRequest,
}

impl ChatUI {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        Ok(ChatUI {
            terminal,
            input: String::new(),
            messages: Vec::new(),
            input_mode: InputMode::Normal,
            spinner: Spinner::new(),
        })
    }

    pub fn update(&mut self) -> Result<Option<Action>> {
        self.draw()?;

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc && self.input_mode == InputMode::Waiting {
                    self.input_mode = InputMode::Normal;
                    return Ok(Some(Action::CancelRequest));
                }
            }
        }

        Ok(None)
    }

    pub fn run(&mut self) -> Result<Option<String>> {
        loop {
            self.draw()?;

            if self.input_mode == InputMode::Waiting {
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if key.code == KeyCode::Esc {
                            self.input_mode = InputMode::Normal;
                            self.messages.pop();
                        }
                    }
                }
                continue;
            }

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(None);
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            let message: String = self.input.drain(..).collect();

                            self.messages.push(("user".to_string(), message.clone()));
                            self.input_mode = InputMode::Waiting;
                            self.messages
                                .push(("system".to_string(), "Sending request...".to_string()));

                            return Ok(Some(message));
                        }
                        KeyCode::Char(c) => {
                            self.input.push(c);
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        KeyCode::Esc => {
                            self.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                    InputMode::Waiting => {}
                }
            }
        }
    }

    pub fn add_response(&mut self, response: String) {
        if self.input_mode == InputMode::Waiting {
            self.messages.pop();
        }
        self.messages.push(("assistant".to_string(), response));
        self.input_mode = InputMode::Normal;
    }

    fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                .split(f.area());

            let messages: Vec<ListItem> = self
                .messages
                .iter()
                .map(|(role, content)| {
                    let (style, prefix) = match role.as_str() {
                        "user" => (Style::default().fg(Color::Blue), "You: "),
                        "assistant" => (Style::default().fg(Color::Green), "AI: "),
                        "system" => (Style::default().fg(Color::Yellow), ""),
                        _ => (Style::default(), ""),
                    };

                    let content = if role == "system" && self.input_mode == InputMode::Waiting {
                        format!("{} {}", self.spinner.next(), content)
                    } else {
                        content.clone()
                    };

                    let content = Line::from(vec![Span::styled(prefix, style), Span::raw(content)]);
                    ListItem::new(content)
                })
                .collect();

            let messages =
                List::new(messages).block(Block::default().title("ChatTUI").borders(Borders::ALL));
            f.render_widget(messages, chunks[0]);

            let input = Paragraph::new(self.input.as_str())
                .style(match self.input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                    InputMode::Waiting => Style::default().fg(Color::DarkGray),
                })
                .block(Block::default().title("Input").borders(Borders::ALL));
            f.render_widget(input, chunks[1]);

            if self.input_mode == InputMode::Editing {
                f.set_cursor_position(Position::new(
                    chunks[1].x + self.input.len() as u16 + 1,
                    chunks[1].y + 1,
                ));
            }

            let (msg, style) = match self.input_mode {
                InputMode::Normal => (
                    vec![
                        "Press ".into(),
                        "q".bold(),
                        " to exit, ".into(),
                        "e".bold(),
                        " to start editing".into(),
                    ],
                    Style::default(),
                ),
                InputMode::Editing => (
                    vec![
                        "Press ".into(),
                        "Esc".bold(),
                        " to stop editing, ".into(),
                        "Enter".bold(),
                        " to send the message".into(),
                    ],
                    Style::default(),
                ),
                InputMode::Waiting => (
                    vec!["Press ".into(), "Esc".bold(), " to cancel request".into()],
                    Style::default(),
                ),
            };

            let text = Text::from(Line::from(msg)).patch_style(style);
            let help_message = Paragraph::new(text);

            f.render_widget(help_message, chunks[1]);
        })?;

        Ok(())
    }
}

impl Drop for ChatUI {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        stdout().execute(LeaveAlternateScreen).unwrap();
    }
}
