use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use reqwest::blocking::Client;
use std::{cmp, fmt, io, time::Duration};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

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

struct DisplayString {
    value: String,
    edit_mode: bool,
}

impl DisplayString {
    fn new(value: String) -> Self {
        Self {
            value: value,
            edit_mode: false,
        }
    }

    fn add_char(&mut self, ch: char) {
        self.value.push(ch);
    }

    fn remove_last_char(&mut self) {
        self.value.pop();
    }

    fn update_value(&mut self, value: String) {
        self.value = value;
    }

    fn toggle_mode(&mut self) {
        self.edit_mode = !self.edit_mode;
    }
}

#[derive(EnumIter, EnumString)]
enum RequestType {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl fmt::Display for RequestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let request_type = match self {
            RequestType::GET => "GET",
            RequestType::POST => "POST",
            RequestType::PUT => "PUT",
            RequestType::PATCH => "PATCH",
            RequestType::DELETE => "DELETE",
        };
        write!(f, "{}", request_type)
    }
}

fn parse_into_https(url: &str) -> String {
    if url.starts_with("http") || url.starts_with("https") {
        return url.to_string();
    }

    format!("https://{}", url)
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
    let placeholder_url_value = "<Enter URL here>";
    let placeholder_request_body = "<Provide request body if applicable>";

    let mut url = DisplayString::new(placeholder_url_value.to_string());
    let mut request_body = DisplayString::new(placeholder_request_body.to_string());
    let mut response = DisplayString::new("".to_string());

    let vertical_constraints = [
        Constraint::Percentage(7),
        Constraint::Percentage(45),
        Constraint::Percentage(45),
        Constraint::Percentage(2),
    ];

    let horizontal_constraints = [Constraint::Percentage(30), Constraint::Percentage(70)];

    let mut active_chunk: usize = 0;
    let chunk_size = vertical_constraints.len() + 1; // Add one for status bar

    let request_types = RequestType::iter().map(|r| r.to_string()).collect();
    let mut request_type_dropdown = Dropdown::new(request_types);
    let mut request_type_val = String::from(RequestType::GET.to_string());
    let client = Client::new();

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vertical_constraints.as_ref())
                .split(frame.area());

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(horizontal_constraints.as_ref())
                .split(chunks[0]);

            let url_display_string = if url.edit_mode {
                format!("{}█", url.value)
            } else {
                url.value.clone()
            };

            let url_block_title = if url.edit_mode {
                "Request URL - editing"
            } else {
                "Request URL"
            };
            let url_block = Paragraph::new(Span::styled(
                &url_display_string,
                Style::default().fg(Color::White),
            ))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(url_block_title)
                    .title_style(
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(if active_chunk == 1 {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::LightBlue)
                    }),
            );

            let request_body_display_string = if request_body.edit_mode {
                format!("{}█", request_body.value)
            } else {
                request_body.value.clone()
            };

            let request_body_block_title = if request_body.edit_mode {
                "Request Body - editing"
            } else {
                "Request Body"
            };
            let request_body_block = Paragraph::new(Span::styled(
                &request_body_display_string,
                Style::default().fg(Color::White),
            ))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(request_body_block_title)
                    .title_style(
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(if active_chunk == 2 {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::LightBlue)
                    }),
            );
            let response_block = Paragraph::new(Span::styled(
                &response.value,
                Style::default().fg(Color::White),
            ))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Response")
                    .title_style(
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(if active_chunk == 3 {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::LightBlue)
                    }),
            );

            let request_type_block_title = if request_type_dropdown.open {
                "Request Type - editing"
            } else {
                "Request Type"
            };
            if request_type_dropdown.open {
                let items: Vec<ListItem> = request_type_dropdown
                    .items
                    .iter()
                    .map(|item| ListItem::new(Span::raw(item.clone())))
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(request_type_block_title),
                    )
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
                        .title(request_type_block_title)
                        .title_style(
                            Style::default()
                                .fg(Color::LightYellow)
                                .add_modifier(Modifier::BOLD),
                        )
                        .border_style(if active_chunk == 0 {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::LightBlue)
                        }),
                );

                frame.render_widget(selected_request_type, horizontal_chunks[0]);
            }

            let status_bar = Paragraph::new(Span::styled(
                "[e] Edit [enter] Save/Exit edit mode [r] Request [q] Quit",
                Style::default().bg(Color::Green).fg(Color::Black),
            ))
            .block(Block::default().bg(Color::Green).borders(Borders::NONE));

            frame.render_widget(url_block, horizontal_chunks[1]);
            frame.render_widget(request_body_block, chunks[1]);
            frame.render_widget(response_block, chunks[2]);
            frame.render_widget(status_bar, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => {
                        if url.edit_mode {
                            url.add_char(c);
                        }

                        if request_body.edit_mode {
                            request_body.add_char(c);
                        }

                        if c == 'e' {
                            if active_chunk == 0 {
                                request_type_dropdown.toggle()
                            }

                            if active_chunk == 1 && !url.edit_mode {
                                url.toggle_mode();
                                if url.value == placeholder_url_value {
                                    url.update_value(String::from(""));
                                }
                            }

                            if active_chunk == 2 && !request_body.edit_mode {
                                request_body.toggle_mode();
                                if request_body.value == placeholder_request_body {
                                    request_body.update_value(String::from(""));
                                }
                            }
                        }

                        if c == 'r'
                            && !url.edit_mode
                            && !request_body.edit_mode
                            && !request_type_dropdown.open
                        {
                            let url_path = parse_into_https(&url.value);
                            let res = match request_type_val.parse::<RequestType>().unwrap() {
                                RequestType::GET => client.get(url_path).send().unwrap(),
                                RequestType::POST => {
                                    if !request_body.value.contains(placeholder_request_body) {
                                        client
                                            .post(&url_path)
                                            .header("Content-Type", "application/json")
                                            .body(request_body.value.clone())
                                            .send()
                                            .unwrap()
                                    } else {
                                        client.post(&url_path).send().unwrap()
                                    }
                                }
                                RequestType::PUT => {
                                    if !request_body.value.contains(placeholder_request_body) {
                                        client
                                            .put(&url_path)
                                            .header("content-type", "application/json")
                                            .body(request_body.value.clone())
                                            .send()
                                            .unwrap()
                                    } else {
                                        client.put(&url_path).send().unwrap()
                                    }
                                }
                                RequestType::PATCH => {
                                    if !request_body.value.contains(placeholder_request_body) {
                                        client
                                            .patch(&url_path)
                                            .header("content-type", "application/json")
                                            .body(request_body.value.clone())
                                            .send()
                                            .unwrap()
                                    } else {
                                        client.patch(&url_path).send().unwrap()
                                    }
                                }
                                RequestType::DELETE => {
                                    if !request_body.value.contains(placeholder_request_body) {
                                        client
                                            .delete(&url_path)
                                            .header("content-type", "application/json")
                                            .body(request_body.value.clone())
                                            .send()
                                            .unwrap()
                                    } else {
                                        client.delete(&url_path).send().unwrap()
                                    }
                                }
                            };

                            if res.status().is_success() {
                                response.update_value(res.text().unwrap());
                            } else {
                                panic!("Received error from request")
                            }
                        }

                        if c == 'q' {
                            break;
                        }
                    }
                    KeyCode::Backspace => {
                        if url.edit_mode {
                            url.remove_last_char();
                        }

                        if request_body.edit_mode {
                            request_body.remove_last_char();
                        }
                    }
                    KeyCode::Down | KeyCode::Right => {
                        if request_type_dropdown.open {
                            request_type_dropdown.next();
                        } else if url.edit_mode || request_body.edit_mode {
                        } else {
                            active_chunk = cmp::min(active_chunk + 1, chunk_size - 2);
                        }
                    }
                    KeyCode::Up | KeyCode::Left => {
                        if request_type_dropdown.open {
                            request_type_dropdown.previous();
                        } else if url.edit_mode || request_body.edit_mode {
                        } else {
                            active_chunk = if active_chunk == 0 {
                                0
                            } else {
                                active_chunk - 1
                            }
                        }
                    }
                    KeyCode::Enter | KeyCode::Esc => {
                        if request_type_dropdown.open {
                            if let Some(i) = request_type_dropdown.state.selected() {
                                request_type_val = request_type_dropdown.items[i].clone();
                            }
                            request_type_dropdown.toggle();
                        }

                        if url.edit_mode {
                            url.toggle_mode();
                        }

                        if request_body.edit_mode {
                            request_body.toggle_mode();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
