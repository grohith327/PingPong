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
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread::sleep,
    time::Duration,
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, FromRepr};
use tokio::{runtime::Runtime, time::Instant};

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

    fn append_string(&mut self, value: String) {
        self.value = format!("{}\n{}", self.value, value);
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

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, PartialEq)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "Request/Reply")]
    RequestReply,
    #[strum(to_string = "Load Test")]
    LoadTest,
}

impl SelectedTab {
    fn all() -> &'static [SelectedTab] {
        use SelectedTab::*;
        &[RequestReply, LoadTest]
    }

    fn previous(self) -> Self {
        let current_index = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    fn description(self) -> String {
        match self {
            SelectedTab::RequestReply => "Send individual requests to your endpoint".to_string(),
            SelectedTab::LoadTest => "Load test your API".to_string(),
        }
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

struct App {
    should_exit: bool,
    active_block: usize,
    block_size: usize,
    request_type: Dropdown,
    request_url: DisplayString,
    request_body: DisplayString,
    headers: DisplayString,
    response: DisplayString,
    selected_tab: SelectedTab,
    load_test_url: DisplayString,
    load_test_result: Arc<Mutex<DisplayString>>,
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
            selected_tab: SelectedTab::RequestReply,
            load_test_url: DisplayString::new("".to_string()),
            load_test_result: Arc::new(Mutex::new(DisplayString::new("".to_string()))),
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
            let any_block_in_edit_mode = self.request_body.edit_mode.clone()
                || self.request_url.edit_mode.clone()
                || self.headers.edit_mode.clone()
                || self.load_test_url.edit_mode.clone();

            let mut display_strings = vec![
                &mut self.request_url,
                &mut self.request_body,
                &mut self.headers,
            ];
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => {
                        if c == 'h' && !any_block_in_edit_mode {
                            self.selected_tab = self.selected_tab.previous();
                        }

                        if c == 'l' && !any_block_in_edit_mode {
                            self.selected_tab = self.selected_tab.next();
                        }

                        if c == 'q' && !any_block_in_edit_mode {
                            self.should_exit = true;
                        }

                        if self.selected_tab == SelectedTab::LoadTest {
                            if self.load_test_url.edit_mode {
                                self.load_test_url.add_char(c);
                            }

                            if c == 'e' && !self.load_test_url.edit_mode {
                                self.load_test_url.toggle_mode();
                            }

                            if c == 'r' && !self.load_test_url.edit_mode {
                                let url = self.load_test_url.clone();
                                let parsed_url = parse_into_https(&url.value);
                                App::run_load_test(parsed_url, self.load_test_result.clone());
                            }
                        } else {
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
                                    if self.request_body.value
                                        == PLACEHOLDER_REQUEST_BODY.to_string()
                                    {
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
                                            self.client
                                                .post(&url_path)
                                                .headers(parsed_headers)
                                                .send()
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
                                            self.client
                                                .put(&url_path)
                                                .headers(parsed_headers)
                                                .send()
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
                                            self.client
                                                .patch(&url_path)
                                                .headers(parsed_headers)
                                                .send()
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
                                            self.client
                                                .delete(&url_path)
                                                .headers(parsed_headers)
                                                .send()
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
                                                output.text().unwrap_or_else(|_| {
                                                    "No response body".to_string()
                                                }),
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        self.response.update_value(format!(
                                            "Error while making request: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if self.selected_tab == SelectedTab::LoadTest {
                            self.load_test_url.remove_last_char();
                        } else {
                            for display_string in display_strings.iter_mut() {
                                if display_string.edit_mode {
                                    display_string.remove_last_char();
                                    break;
                                }
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
                        if self.selected_tab == SelectedTab::LoadTest {
                            self.load_test_url.toggle_mode();
                        } else {
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
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let titles: Vec<Span> = SelectedTab::all()
            .iter()
            .map(|t| {
                Span::from(Span::styled(
                    t.to_string(),
                    Style::default().fg(Color::White),
                ))
            })
            .collect();
        let selected_tab_index = self.selected_tab as usize;
        let tabs = Tabs::new(titles)
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().fg(Color::Gray))
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ");
        use Constraint::{Length, Min};
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(frame.area());

        let horizontal = Layout::horizontal([Min(0), Length(50)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);
        frame.render_widget(self.selected_tab.description().bold(), title_area);
        frame.render_widget(tabs, tabs_area);
        match self.selected_tab {
            SelectedTab::RequestReply => {
                self.render_request_reply_tab(frame, inner_area);
            }
            SelectedTab::LoadTest => {
                self.render_load_test_tab(frame, inner_area);
            }
        }

        let footer_widget =
            Line::raw("[h] Previous tab [l] Next tab [e] Edit [enter] Save/Exit edit mode [r] Request [q] Quit").centered();
        frame.render_widget(footer_widget, footer_area);
    }

    fn render_request_reply_tab(&mut self, frame: &mut Frame, area: Rect) {
        let vertical_constraints = [
            Constraint::Percentage(7),
            Constraint::Percentage(45),
            Constraint::Percentage(45),
        ];

        let request_horizontal_constraints =
            [Constraint::Percentage(20), Constraint::Percentage(80)];
        let body_horizontal_contraints = [Constraint::Percentage(50), Constraint::Percentage(50)];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints.as_ref())
            .split(area);

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
    }

    fn render_load_test_tab(&mut self, frame: &mut Frame, area: Rect) {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Max(4),
                Constraint::Min(30),
                Constraint::Percentage(10),
            ])
            .split(area);

        let url = generate_paragraph(&self.load_test_url, "Load test url".to_string(), true);
        let load_test_result_clone = self.load_test_result.clone();
        let load_test_result_clone_lock = load_test_result_clone.lock().unwrap();
        let result = Paragraph::new(load_test_result_clone_lock.value.to_string())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Load test result")
                    .title_style(
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    ),
            );
        drop(load_test_result_clone_lock);

        frame.render_widget(url, vertical_chunks[1]);
        frame.render_widget(result, vertical_chunks[2]);
    }

    fn run_load_test(url: String, result: Arc<Mutex<DisplayString>>) {
        std::thread::spawn(move || {
            let runtime = Runtime::new().unwrap();
            let mut tps = 10;
            let mut result_lock = result.lock().unwrap();
            result_lock.append_string("Running load test...".to_string());
            drop(result_lock);

            loop {
                let success_count = Arc::new(AtomicUsize::new(0));
                let failure_count = Arc::new(AtomicUsize::new(0));
                let duration = Duration::from_secs(10);
                let failure_threshold = 20.0;

                runtime.block_on(async {
                    let mut tasks = Vec::with_capacity(tps);
                    let start_time = Instant::now();

                    while Instant::now() - start_time < duration {
                        for _ in 0..tps {
                            let client_clone = Arc::new(reqwest::Client::new()); // Need a non-blocking client
                            let endpoint_clone = url.clone();
                            let success_count_clone = success_count.clone();
                            let failure_count_clone = failure_count.clone();

                            tasks.push(tokio::spawn(async move {
                                let result = client_clone.get(endpoint_clone).send().await;

                                match result {
                                    Ok(response) => {
                                        if response.status().is_success() {
                                            success_count_clone.fetch_add(1, Ordering::SeqCst);
                                        } else {
                                            failure_count_clone.fetch_add(1, Ordering::SeqCst);
                                        }
                                    }
                                    _ => {
                                        failure_count_clone.fetch_add(1, Ordering::SeqCst);
                                    }
                                }
                            }));
                        }

                        sleep(Duration::from_secs(1));
                        tasks.retain(|task| !task.is_finished());
                    }

                    futures::future::join_all(tasks).await;
                });

                let successes = success_count.load(Ordering::SeqCst);
                let failures = failure_count.load(Ordering::SeqCst);
                let total = successes + failures;
                let failure_rate = failures as f64 / total as f64 * 100.0;

                let mut result_lock = result.lock().unwrap();
                result_lock
                    .append_string(format!("TPS: {}, Failure rate: {:.2}%", tps, failure_rate));

                if failure_rate > failure_threshold {
                    result_lock.append_string(format!(
                        "Breaking point reached! Failure rate exceeds {}% at {} TPS.",
                        failure_threshold, tps
                    ));
                    result_lock.append_string("Completed load test".to_string());
                    break;
                }

                drop(result_lock);
                tps += 10;
                sleep(Duration::from_secs(5));
            }
        });
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
