use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Position},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Terminal,
};

use std::{
    io::{stdout, Stdout},
    time::{Duration, Instant},
};

pub struct ChatUI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    input: String,
    messages: Vec<(String, String)>,
    input_mode: InputMode,
    spinner: Spinner,
    input_width: u16,
    horizontal_scroll_state: ScrollbarState,
    horizontal_scroll: usize,
    vertical_scroll_state: ScrollbarState,
    list_state: ListState,
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

        let mut chat_ui = ChatUI {
            terminal,
            input: String::new(),
            messages: Vec::new(),
            input_mode: InputMode::Normal,
            spinner: Spinner::new(),
            input_width: 0,
            horizontal_scroll_state: ScrollbarState::new(0),
            horizontal_scroll: 0,
            vertical_scroll_state: ScrollbarState::default(),
            list_state: ListState::default(),
        };
        chat_ui.list_state.select(Some(0));
        Ok(chat_ui)
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
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            self.draw()?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if self.input_mode == InputMode::Waiting {
                if event::poll(timeout)? {
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
                        KeyCode::Up => {
                            let current = self.list_state.selected().unwrap_or(0);
                            let next = current.saturating_sub(1);
                            self.list_state.select(Some(next));
                            self.vertical_scroll_state = self.vertical_scroll_state.position(next);
                        }
                        KeyCode::Down => {
                            let current = self.list_state.selected().unwrap_or(0);
                            let next = (current + 1).min(self.messages.len().saturating_sub(1));
                            self.list_state.select(Some(next));
                            self.vertical_scroll_state = self.vertical_scroll_state.position(next);
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

                            // reset horizontal scroll when starting a new input
                            self.horizontal_scroll = 0;
                            self.horizontal_scroll_state = ScrollbarState::default();

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
                        KeyCode::Left => {
                            self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
                        }
                        KeyCode::Right => {
                            let max_scroll =
                                self.input.len().saturating_sub(self.input_width as usize);
                            self.horizontal_scroll = (self.horizontal_scroll + 1).min(max_scroll);
                        }
                        _ => {}
                    },
                    InputMode::Waiting => {}
                }
            }
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
    }

    pub fn add_response(&mut self, response: String) {
        if self.input_mode == InputMode::Waiting {
            self.messages.pop();
        }
        self.messages.push(("assistant".to_string(), response));
        self.input_mode = InputMode::Normal;

        // scroll to the bottom
        self.list_state.select(Some(self.messages.len() - 1));
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.messages.len() - 1);

        // reset horizontal scroll when switching back to normal mode
        self.horizontal_scroll = 0;
        self.horizontal_scroll_state = ScrollbarState::default();
    }

    fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                .split(f.area());

            let messages_area = chunks[0];
            let messages_inner_area = messages_area.inner(Margin::new(1, 1));

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

                    let wrapped_content =
                        wrap_text(&content, messages_inner_area.width as usize - prefix.len());
                    let mut lines = Vec::new();

                    for (i, line) in wrapped_content.into_iter().enumerate() {
                        if i == 0 {
                            lines.push(Line::from(vec![
                                Span::styled(prefix, style),
                                Span::raw(line),
                            ]));
                        } else {
                            lines.push(Line::from(vec![
                                Span::raw(" ".repeat(prefix.len())),
                                Span::raw(line),
                            ]));
                        }
                    }

                    ListItem::new(lines)
                })
                .collect();

            let messages = List::new(messages)
                .block(Block::default().title("ChatTUI").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray));

            self.vertical_scroll_state = self
                .vertical_scroll_state
                .content_length(self.messages.len())
                .viewport_content_length(messages_area.height as usize);

            f.render_stateful_widget(messages, messages_area, &mut self.list_state);

            f.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None),
                messages_area.inner(Margin::new(0, 1)),
                &mut self.vertical_scroll_state,
            );

            // store the available width for the input
            self.input_width = chunks[1].width.saturating_sub(2);

            let input = Paragraph::new(self.input.as_str())
                .style(match self.input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                    InputMode::Waiting => Style::default().fg(Color::DarkGray),
                })
                .block(Block::default().borders(Borders::ALL))
                .scroll((0, self.horizontal_scroll as u16));
            f.render_widget(input, chunks[1]);

            // only update scroll state and render scrollbar if necessary
            if self.input.len() as u16 > self.input_width {
                let content_length = self.input.len();
                let viewport_content_length = self.input_width as usize;

                self.horizontal_scroll_state = self
                    .horizontal_scroll_state
                    .content_length(content_length)
                    .viewport_content_length(viewport_content_length)
                    .position(self.horizontal_scroll);

                f.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                        .thumb_symbol("🬋")
                        .begin_symbol(None)
                        .end_symbol(None),
                    chunks[1].inner(Margin {
                        vertical: 0,
                        horizontal: 1,
                    }),
                    &mut self.horizontal_scroll_state,
                );
            }

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

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        if current_line.len() + word.len() + 1 > max_width {
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
            if word.len() > max_width {
                let mut remaining = word;
                while !remaining.is_empty() {
                    let (chunk, rest) =
                        remaining.split_at(std::cmp::min(remaining.len(), max_width));
                    lines.push(chunk.to_string());
                    remaining = rest;
                }
            } else {
                current_line = word.to_string();
            }
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

impl Drop for ChatUI {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        stdout().execute(LeaveAlternateScreen).unwrap();
    }
}
