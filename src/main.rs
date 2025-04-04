use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::{DefaultTerminal, prelude::*, widgets::*};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::{
    cmp,
    collections::HashMap,
    fmt::{self},
    io,
    str::FromStr,
    time::Duration,
};
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

    fn get_selected_value(&mut self) -> String {
        if let Some(i) = self.state.selected() {
            return self.items[i].clone();
        }

        self.items[0].clone()
    }

    fn toggle(&mut self) {
        self.open = !self.open
    }
}

#[derive(Clone)]
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

    if url.contains("localhost") {
        return format!("http://{}", url);
    }

    format!("https://{}", url)
}

fn build_headers(headers: &str) -> Result<HeaderMap> {
    let map: HashMap<String, String> = serde_json::from_str(headers)?;
    let mut out = HeaderMap::new();
    for (key, value) in map {
        out.insert(HeaderName::from_str(&key)?, HeaderValue::from_str(&value)?);
    }

    Ok(out)
}

static PLACEHOLDER_URL_VALUE: &str = "<Enter URL here>";
static PLACEHOLDER_REQUEST_BODY: &str = "<Provide request body here>";
static PLACEHOLDER_HEADERS: &str = r#"{"content-type": "application/json"}"#;

fn main() -> Result<()> {
    // enable_raw_mode()?;
    // let mut stdout = io::stdout();
    // execute!(stdout, EnterAlternateScreen)?;
    // let backend = CrosstermBackend::new(stdout);
    // let mut terminal = Terminal::new(backend)?;
    // let out = run_app(&mut terminal);
    // disable_raw_mode()?;
    // execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    // out

    color_eyre::install()?;
    let terminal = ratatui::init();
    let request_types = RequestType::iter().map(|r| r.to_string()).collect();

    let app = App::new(
        request_types,
        PLACEHOLDER_URL_VALUE,
        PLACEHOLDER_REQUEST_BODY,
        PLACEHOLDER_HEADERS,
        "",
    );
    let result = app.run(terminal);
    ratatui::restore();
    result
}

// fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
//     let placeholder_url_value = "<Enter URL here>";
//     let placeholder_request_body = "<Provide request body if applicable>";
//     let placeholder_header = r#"{"content-type": "application/json"}"#;

//     let mut url = DisplayString::new(placeholder_url_value.to_string());
//     let mut request_body = DisplayString::new(placeholder_request_body.to_string());
//     let mut response = DisplayString::new("".to_string());
//     let mut headers = DisplayString::new(placeholder_header.to_string());

//     let vertical_constraints = [
//         Constraint::Percentage(7),
//         Constraint::Percentage(45),
//         Constraint::Percentage(45),
//         Constraint::Percentage(2),
//     ];

//     let request_horizontal_constraints = [Constraint::Percentage(20), Constraint::Percentage(80)];
//     let body_horizontal_contraints = [Constraint::Percentage(50), Constraint::Percentage(50)];

//     let mut active_chunk: usize = 0;
//     let chunk_size = 5;
//     let request_types = RequestType::iter().map(|r| r.to_string()).collect();
//     let mut request_type_dropdown = Dropdown::new(request_types);
//     let mut request_type_val = String::from(RequestType::GET.to_string());
//     let client = Client::new();

//     loop {
//         terminal.draw(|frame| {
//             let chunks = Layout::default()
//                 .direction(Direction::Vertical)
//                 .constraints(vertical_constraints.as_ref())
//                 .split(frame.area());

//             let request_horizontal_chunks = Layout::default()
//                 .direction(Direction::Horizontal)
//                 .constraints(request_horizontal_constraints.as_ref())
//                 .split(chunks[0]);

//             let body_horizontal_chunks = Layout::default()
//                 .direction(Direction::Horizontal)
//                 .constraints(body_horizontal_contraints)
//                 .split(chunks[1]);

//             let url_display_string = if url.edit_mode {
//                 format!("{}█", url.value)
//             } else {
//                 url.value.clone()
//             };

//             let url_block_title = if url.edit_mode {
//                 "Request URL - editing"
//             } else {
//                 "Request URL"
//             };
//             let url_block = Paragraph::new(Span::styled(
//                 &url_display_string,
//                 Style::default().fg(Color::White),
//             ))
//             .block(
//                 Block::default()
//                     .borders(Borders::ALL)
//                     .title(url_block_title)
//                     .title_style(
//                         Style::default()
//                             .fg(Color::LightYellow)
//                             .add_modifier(Modifier::BOLD),
//                     )
//                     .border_style(if active_chunk == 1 {
//                         Style::default()
//                             .fg(Color::White)
//                             .add_modifier(Modifier::BOLD)
//                     } else {
//                         Style::default().fg(Color::LightBlue)
//                     }),
//             );

//             let request_body_display_string = if request_body.edit_mode {
//                 format!("{}█", request_body.value)
//             } else {
//                 request_body.value.clone()
//             };

//             let request_body_block_title = if request_body.edit_mode {
//                 "Request Body - editing"
//             } else {
//                 "Request Body"
//             };
//             let request_body_block = Paragraph::new(Span::styled(
//                 &request_body_display_string,
//                 Style::default().fg(Color::White),
//             ))
//             .wrap(Wrap { trim: true })
//             .block(
//                 Block::default()
//                     .borders(Borders::ALL)
//                     .title(request_body_block_title)
//                     .title_style(
//                         Style::default()
//                             .fg(Color::LightYellow)
//                             .add_modifier(Modifier::BOLD),
//                     )
//                     .border_style(if active_chunk == 2 {
//                         Style::default()
//                             .fg(Color::White)
//                             .add_modifier(Modifier::BOLD)
//                     } else {
//                         Style::default().fg(Color::LightBlue)
//                     }),
//             );

//             let headers_display_string = if headers.edit_mode {
//                 format!("{}█", headers.value)
//             } else {
//                 headers.value.clone()
//             };

//             let headers_block_title = if headers.edit_mode {
//                 "Request Body - editing"
//             } else {
//                 "Request Body"
//             };
//             let headers_body_block = Paragraph::new(Span::styled(
//                 &headers_display_string,
//                 Style::default().fg(Color::White),
//             ))
//             .wrap(Wrap { trim: true })
//             .block(
//                 Block::default()
//                     .borders(Borders::ALL)
//                     .title(headers_block_title)
//                     .title_style(
//                         Style::default()
//                             .fg(Color::LightYellow)
//                             .add_modifier(Modifier::BOLD),
//                     )
//                     .border_style(if active_chunk == 3 {
//                         Style::default()
//                             .fg(Color::White)
//                             .add_modifier(Modifier::BOLD)
//                     } else {
//                         Style::default().fg(Color::LightBlue)
//                     }),
//             );

//             let response_block = Paragraph::new(Span::styled(
//                 &response.value,
//                 Style::default().fg(Color::White),
//             ))
//             .wrap(Wrap { trim: true })
//             .block(
//                 Block::default()
//                     .borders(Borders::ALL)
//                     .title("Response")
//                     .title_style(
//                         Style::default()
//                             .fg(Color::LightYellow)
//                             .add_modifier(Modifier::BOLD),
//                     )
//                     .border_style(if active_chunk == 4 {
//                         Style::default()
//                             .fg(Color::White)
//                             .add_modifier(Modifier::BOLD)
//                     } else {
//                         Style::default().fg(Color::LightBlue)
//                     }),
//             );

//             let request_type_block_title = if request_type_dropdown.open {
//                 "Request Type - editing"
//             } else {
//                 "Request Type"
//             };
//             if request_type_dropdown.open {
//                 let items: Vec<ListItem> = request_type_dropdown
//                     .items
//                     .iter()
//                     .map(|item| ListItem::new(Span::raw(item.clone())))
//                     .collect();

//                 let list = List::new(items)
//                     .block(
//                         Block::default()
//                             .borders(Borders::ALL)
//                             .title(request_type_block_title),
//                     )
//                     .highlight_style(Style::default().fg(Color::Yellow));

//                 frame.render_stateful_widget(
//                     list,
//                     request_horizontal_chunks[0],
//                     &mut request_type_dropdown.state,
//                 );
//             } else {
//                 let selected_request_type = Paragraph::new(Span::styled(
//                     &request_type_val,
//                     Style::default().fg(Color::White),
//                 ))
//                 .block(
//                     Block::default()
//                         .borders(Borders::ALL)
//                         .title(request_type_block_title)
//                         .title_style(
//                             Style::default()
//                                 .fg(Color::LightYellow)
//                                 .add_modifier(Modifier::BOLD),
//                         )
//                         .border_style(if active_chunk == 0 {
//                             Style::default()
//                                 .fg(Color::White)
//                                 .add_modifier(Modifier::BOLD)
//                         } else {
//                             Style::default().fg(Color::LightBlue)
//                         }),
//                 );

//                 frame.render_widget(selected_request_type, request_horizontal_chunks[0]);
//             }

//             let status_bar = Paragraph::new(Span::styled(
//                 "[e] Edit [enter] Save/Exit edit mode [r] Request [q] Quit",
//                 Style::default().bg(Color::Green).fg(Color::Black),
//             ))
//             .block(Block::default().bg(Color::Green).borders(Borders::NONE));

//             frame.render_widget(url_block, request_horizontal_chunks[1]);
//             frame.render_widget(request_body_block, body_horizontal_chunks[0]);
//             frame.render_widget(headers_body_block, body_horizontal_chunks[1]);
//             frame.render_widget(response_block, chunks[2]);
//             frame.render_widget(status_bar, chunks[3]);
//         })?;

//         if event::poll(Duration::from_millis(100))? {
//             if let event::Event::Key(key) = event::read()? {
//                 match key.code {
//                     KeyCode::Char(c) => {
//                         if url.edit_mode {
//                             url.add_char(c);
//                         }

//                         if request_body.edit_mode {
//                             request_body.add_char(c);
//                         }

//                         if headers.edit_mode {
//                             headers.add_char(c);
//                         }

//                         if c == 'e' {
//                             if active_chunk == 0 {
//                                 request_type_dropdown.toggle()
//                             }

//                             if active_chunk == 1 && !url.edit_mode {
//                                 url.toggle_mode();
//                                 if url.value == placeholder_url_value {
//                                     url.update_value(String::from(""));
//                                 }
//                             }

//                             if active_chunk == 2 && !request_body.edit_mode {
//                                 request_body.toggle_mode();
//                                 if request_body.value == placeholder_request_body {
//                                     request_body.update_value(String::from(""));
//                                 }
//                             }

//                             if active_chunk == 3 && !headers.edit_mode {
//                                 headers.toggle_mode();
//                             }
//                         }

//                         if c == 'r'
//                             && !url.edit_mode
//                             && !request_body.edit_mode
//                             && !request_type_dropdown.open
//                         {
//                             let url_path = parse_into_https(&url.value);
//                             let parsed_headers = build_headers(&headers.value).unwrap();
//                             let res = match request_type_val.parse::<RequestType>().unwrap() {
//                                 RequestType::GET => {
//                                     client.get(url_path).headers(parsed_headers).send()
//                                 }
//                                 RequestType::POST => {
//                                     if !request_body.value.contains(placeholder_request_body) {
//                                         client
//                                             .post(&url_path)
//                                             .headers(parsed_headers)
//                                             .body(request_body.value.clone())
//                                             .send()
//                                     } else {
//                                         client.post(&url_path).headers(parsed_headers).send()
//                                     }
//                                 }
//                                 RequestType::PUT => {
//                                     if !request_body.value.contains(placeholder_request_body) {
//                                         client
//                                             .put(&url_path)
//                                             .headers(parsed_headers)
//                                             .body(request_body.value.clone())
//                                             .send()
//                                     } else {
//                                         client.put(&url_path).headers(parsed_headers).send()
//                                     }
//                                 }
//                                 RequestType::PATCH => {
//                                     if !request_body.value.contains(placeholder_request_body) {
//                                         client
//                                             .patch(&url_path)
//                                             .headers(parsed_headers)
//                                             .body(request_body.value.clone())
//                                             .send()
//                                     } else {
//                                         client.patch(&url_path).headers(parsed_headers).send()
//                                     }
//                                 }
//                                 RequestType::DELETE => {
//                                     if !request_body.value.contains(placeholder_request_body) {
//                                         client
//                                             .delete(&url_path)
//                                             .headers(parsed_headers)
//                                             .body(request_body.value.clone())
//                                             .send()
//                                     } else {
//                                         client.delete(&url_path).headers(parsed_headers).send()
//                                     }
//                                 }
//                             };

//                             match res {
//                                 Ok(output) => {
//                                     if output.status().is_success() {
//                                         response.update_value(output.text().unwrap());
//                                     } else {
//                                         response.update_value(format!(
//                                             "Status code: {}, Error message: {}",
//                                             output.status(),
//                                             output
//                                                 .text()
//                                                 .unwrap_or_else(|_| "No response body".to_string()),
//                                         ));
//                                     }
//                                 }
//                                 Err(e) => {
//                                     response
//                                         .update_value(format!("Error while making request: {}", e));
//                                 }
//                             }
//                         }

//                         if c == 'q' {
//                             break;
//                         }
//                     }
//                     KeyCode::Backspace => {
//                         if url.edit_mode {
//                             url.remove_last_char();
//                         }

//                         if request_body.edit_mode {
//                             request_body.remove_last_char();
//                         }

//                         if headers.edit_mode {
//                             headers.remove_last_char();
//                         }
//                     }
//                     KeyCode::Down | KeyCode::Right => {
//                         if request_type_dropdown.open {
//                             request_type_dropdown.next();
//                         } else if url.edit_mode || request_body.edit_mode || headers.edit_mode {
//                         } else {
//                             active_chunk = cmp::min(active_chunk + 1, chunk_size - 2);
//                         }
//                     }
//                     KeyCode::Up | KeyCode::Left => {
//                         if request_type_dropdown.open {
//                             request_type_dropdown.previous();
//                         } else if url.edit_mode || request_body.edit_mode || headers.edit_mode {
//                         } else {
//                             active_chunk = if active_chunk == 0 {
//                                 0
//                             } else {
//                                 active_chunk - 1
//                             }
//                         }
//                     }
//                     KeyCode::Enter | KeyCode::Esc => {
//                         if request_type_dropdown.open {
//                             if let Some(i) = request_type_dropdown.state.selected() {
//                                 request_type_val = request_type_dropdown.items[i].clone();
//                             }
//                             request_type_dropdown.toggle();
//                         }

//                         if url.edit_mode {
//                             url.toggle_mode();
//                         }

//                         if request_body.edit_mode {
//                             request_body.toggle_mode();
//                         }

//                         if headers.edit_mode {
//                             headers.toggle_mode();
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     }

//     Ok(())
// }

struct App {
    should_exit: bool,
    active_block: usize,
    block_size: usize,
    request_type: Dropdown,
    request_url: DisplayString,
    request_body: DisplayString,
    headers: DisplayString,
    response: DisplayString,
    client: Client,
}

impl App {
    fn new(
        request_types: Vec<String>,
        default_request_url: &str,
        default_request_body: &str,
        default_headers: &str,
        default_response: &str,
    ) -> Self {
        Self {
            should_exit: false,
            active_block: 0,
            block_size: 4,
            request_type: Dropdown::new(request_types),
            request_url: DisplayString::new(default_request_url.to_string()),
            request_body: DisplayString::new(default_request_body.to_string()),
            headers: DisplayString::new(default_headers.to_string()),
            response: DisplayString::new(default_response.to_string()),
            client: Client::new(),
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(100))? {
            let mut display_strings = vec![
                &mut self.request_url,
                &mut self.request_body,
                &mut self.headers,
            ];
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => {
                        for display_string in display_strings.iter_mut() {
                            if display_string.edit_mode {
                                display_string.add_char(c);
                                break;
                            }
                        }

                        if c == 'e' {
                            if self.active_block == 0 {
                                self.request_type.toggle()
                            }

                            if self.active_block == 1 && !self.request_url.edit_mode {
                                self.request_url.toggle_mode();
                                if self.request_url.value == PLACEHOLDER_URL_VALUE.to_string() {
                                    self.request_url.update_value(String::from(""));
                                }
                            }

                            if self.active_block == 2 && !self.request_body.edit_mode {
                                self.request_body.toggle_mode();
                                if self.request_body.value == PLACEHOLDER_REQUEST_BODY.to_string() {
                                    self.request_body.update_value(String::from(""));
                                }
                            }

                            if self.active_block == 3 && !self.headers.edit_mode {
                                self.headers.toggle_mode();
                            }
                        }

                        if c == 'r'
                            && !self.request_url.edit_mode
                            && !self.request_body.edit_mode
                            && !self.request_type.open
                        {
                            let url_path = parse_into_https(&self.request_url.value);
                            let parsed_headers = build_headers(&self.headers.value).unwrap();
                            let res = match self
                                .request_type
                                .get_selected_value()
                                .parse::<RequestType>()
                                .unwrap()
                            {
                                RequestType::GET => {
                                    self.client.get(url_path).headers(parsed_headers).send()
                                }
                                RequestType::POST => {
                                    if !self
                                        .request_body
                                        .value
                                        .contains(&PLACEHOLDER_REQUEST_BODY.to_string())
                                    {
                                        self.client
                                            .post(&url_path)
                                            .headers(parsed_headers)
                                            .body(self.request_body.value.clone())
                                            .send()
                                    } else {
                                        self.client.post(&url_path).headers(parsed_headers).send()
                                    }
                                }
                                RequestType::PUT => {
                                    if !self
                                        .request_body
                                        .value
                                        .contains(&PLACEHOLDER_REQUEST_BODY.to_string())
                                    {
                                        self.client
                                            .put(&url_path)
                                            .headers(parsed_headers)
                                            .body(self.request_body.value.clone())
                                            .send()
                                    } else {
                                        self.client.put(&url_path).headers(parsed_headers).send()
                                    }
                                }
                                RequestType::PATCH => {
                                    if !self
                                        .request_body
                                        .value
                                        .contains(&PLACEHOLDER_REQUEST_BODY.to_string())
                                    {
                                        self.client
                                            .patch(&url_path)
                                            .headers(parsed_headers)
                                            .body(self.request_body.value.clone())
                                            .send()
                                    } else {
                                        self.client.patch(&url_path).headers(parsed_headers).send()
                                    }
                                }
                                RequestType::DELETE => {
                                    if !self
                                        .request_body
                                        .value
                                        .contains(&PLACEHOLDER_REQUEST_BODY.to_string())
                                    {
                                        self.client
                                            .delete(&url_path)
                                            .headers(parsed_headers)
                                            .body(self.request_body.value.clone())
                                            .send()
                                    } else {
                                        self.client.delete(&url_path).headers(parsed_headers).send()
                                    }
                                }
                            };

                            match res {
                                Ok(output) => {
                                    if output.status().is_success() {
                                        self.response.update_value(output.text().unwrap());
                                    } else {
                                        self.response.update_value(format!(
                                            "Status code: {}, Error message: {}",
                                            output.status(),
                                            output
                                                .text()
                                                .unwrap_or_else(|_| "No response body".to_string()),
                                        ));
                                    }
                                }
                                Err(e) => {
                                    self.response
                                        .update_value(format!("Error while making request: {}", e));
                                }
                            }
                        }

                        if c == 'q' {
                            self.should_exit = true;
                        }
                    }
                    KeyCode::Backspace => {
                        for display_string in display_strings.iter_mut() {
                            if display_string.edit_mode {
                                display_string.remove_last_char();
                                break;
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Right => {
                        if self.request_type.open {
                            self.request_type.next();
                        } else if self.request_url.edit_mode
                            || self.request_body.edit_mode
                            || self.headers.edit_mode
                        {
                        } else {
                            self.active_block =
                                cmp::min(self.active_block + 1, self.block_size - 1);
                        }
                    }
                    KeyCode::Up | KeyCode::Left => {
                        if self.request_type.open {
                            self.request_type.previous();
                        } else if self.request_url.edit_mode
                            || self.request_body.edit_mode
                            || self.headers.edit_mode
                        {
                        } else {
                            self.active_block = if self.active_block == 0 {
                                0
                            } else {
                                self.active_block - 1
                            }
                        }
                    }
                    KeyCode::Enter | KeyCode::Esc => {
                        if self.request_type.open {
                            self.request_type.toggle();
                        }

                        for display_string in display_strings.iter_mut() {
                            if display_string.edit_mode {
                                display_string.toggle_mode();
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical_constraints = [
            Constraint::Percentage(7),
            Constraint::Percentage(45),
            Constraint::Percentage(45),
            Constraint::Percentage(2),
        ];

        let request_horizontal_constraints =
            [Constraint::Percentage(20), Constraint::Percentage(80)];
        let body_horizontal_contraints = [Constraint::Percentage(50), Constraint::Percentage(50)];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints.as_ref())
            .split(frame.area());

        let request_horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(request_horizontal_constraints.as_ref())
            .split(chunks[0]);

        let body_horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(body_horizontal_contraints)
            .split(chunks[1]);

        let request_type_block_title = if self.request_type.open {
            "Request Type - editing"
        } else {
            "Request Type"
        };
        if self.request_type.open {
            let items: Vec<ListItem> = self
                .request_type
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
                request_horizontal_chunks[0],
                &mut self.request_type.state,
            );
        } else {
            let selected_value = DisplayString::new(self.request_type.get_selected_value());
            let request_type_block = generate_paragraph(
                &selected_value,
                "Request Method".to_string(),
                self.active_block == 0,
            );

            frame.render_widget(request_type_block, request_horizontal_chunks[0]);
        }

        let url_block =
            generate_paragraph(&self.request_url, "URL".to_string(), self.active_block == 1);
        frame.render_widget(url_block, request_horizontal_chunks[1]);

        let request_body_block = generate_paragraph(
            &self.request_body,
            "Request Body".to_string(),
            self.active_block == 2,
        );
        frame.render_widget(request_body_block, body_horizontal_chunks[0]);

        let headers_block =
            generate_paragraph(&self.headers, "Headers".to_string(), self.active_block == 3);
        frame.render_widget(headers_block, body_horizontal_chunks[1]);

        let response_body_block = generate_paragraph(
            &self.response,
            "Response".to_string(),
            self.active_block == 4,
        );
        frame.render_widget(response_body_block, chunks[2]);

        let status_bar = Paragraph::new(Span::styled(
            "[e] Edit [enter] Save/Exit edit mode [r] Request [q] Quit",
            Style::default().bg(Color::Green).fg(Color::Black),
        ))
        .block(Block::default().bg(Color::Green).borders(Borders::NONE));
        frame.render_widget(status_bar, chunks[3]);
    }
}

fn generate_paragraph(
    display_string: &DisplayString,
    title: String,
    chunk_active: bool,
) -> Paragraph {
    let mut display_value = display_string.value.to_string();
    let mut display_title = title;

    if display_string.edit_mode {
        display_value = format!("{}█", display_value);
        display_title = format!("{} - Editing", display_title);
    }

    Paragraph::new(Span::styled(
        display_value,
        Style::default().fg(Color::White),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(display_title)
            .title_style(
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(if chunk_active {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightBlue)
            }),
    )
}
