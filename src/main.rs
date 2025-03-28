use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use std::{io, time::Duration};

struct Dropdown {
    items: Vec<String>,
    state: ListState,
    open: bool,
}

impl Dropdown {
    fn new(items: Vec<String>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            items,
            state,
            open: false,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 && self.items.len() > 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn toggle(&mut self) {
        self.open = !self.open
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let out = run_app(&mut terminal);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    out
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut url = String::from("https://google.com");
    let mut request_body = String::from("{key: value}");
    let mut response = String::from("{key: value}");

    let mut request_type_dropdown = Dropdown::new(vec![
        "POST".to_string(),
        "GET".to_string(),
        "PUT".to_string(),
        "UPDATE".to_string(),
        "DELETE".to_string(),
    ]);
    let mut request_type_val = String::from("GET");

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(7),
                        Constraint::Percentage(46),
                        Constraint::Percentage(46),
                    ]
                    .as_ref(),
                )
                .split(frame.area());

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[0]);

            let url_block = Paragraph::new(Span::styled(&url, Style::default().fg(Color::White)))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("URL")
                        .title_style(
                            Style::default()
                                .fg(Color::LightYellow)
                                .add_modifier(Modifier::BOLD),
                        )
                        .border_style(Style::default().fg(Color::LightBlue)),
                );

            let request_body_block = Paragraph::new(Span::styled(
                &request_body,
                Style::default().fg(Color::White),
            ))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Body")
                    .title_style(
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(Color::LightBlue)),
            );
            let response_block =
                Paragraph::new(Span::styled(&response, Style::default().fg(Color::White))).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Response")
                        .title_style(
                            Style::default()
                                .fg(Color::LightYellow)
                                .add_modifier(Modifier::BOLD),
                        )
                        .border_style(Style::default().fg(Color::LightBlue)),
                );

            if request_type_dropdown.open {
                let items: Vec<ListItem> = request_type_dropdown
                    .items
                    .iter()
                    .map(|item| ListItem::new(Span::raw(item.clone())))
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Request Type"))
                    .highlight_style(Style::default().fg(Color::Yellow));

                frame.render_stateful_widget(
                    list,
                    horizontal_chunks[0],
                    &mut request_type_dropdown.state,
                );
            } else {
                let selected_request_type = Paragraph::new(Span::styled(
                    &request_type_val,
                    Style::default().fg(Color::White),
                ))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Request Type")
                        .title_style(
                            Style::default()
                                .fg(Color::LightYellow)
                                .add_modifier(Modifier::BOLD),
                        )
                        .border_style(Style::default().fg(Color::LightBlue)),
                );

                frame.render_widget(selected_request_type, horizontal_chunks[0]);
            }

            frame.render_widget(url_block, horizontal_chunks[1]);
            frame.render_widget(request_body_block, chunks[1]);
            frame.render_widget(response_block, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(' ') => request_type_dropdown.toggle(),
                    KeyCode::Down => request_type_dropdown.next(),
                    KeyCode::Up => request_type_dropdown.previous(),
                    KeyCode::Enter => {
                        if request_type_dropdown.open {
                            if let Some(i) = request_type_dropdown.state.selected() {
                                request_type_val = request_type_dropdown.items[i].clone();
                            }
                            request_type_dropdown.toggle();
                        }
                    }
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => {
                        if request_type_dropdown.open {
                            request_type_dropdown.toggle();
                        } else {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
