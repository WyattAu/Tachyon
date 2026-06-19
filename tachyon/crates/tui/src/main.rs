use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use reqwest::Client;
use serde_json::Value;
use std::io;

#[derive(Parser, Debug)]
#[command(name = "tachyon-tui")]
#[command(about = "Terminal UI for Tachyon knowledge management")]
struct Args {
    /// Server URL
    #[arg(short, long, default_value = "http://localhost:8080")]
    server: String,

    /// Username
    #[arg(short, long, default_value = "admin")]
    username: String,

    /// Password
    #[arg(short, long, default_value = "admin123")]
    password: String,
}

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    Login,
    Documents,
    DocumentView,
    Graph,
    Search,
    Help,
}

struct App {
    mode: AppMode,
    token: Option<String>,
    documents: Vec<Document>,
    selected: usize,
    search_query: String,
    search_results: Vec<Document>,
    status_message: String,
    server_url: String,
    client: Client,
}

#[derive(Debug, Clone)]
struct Document {
    id: String,
    title: String,
    content: String,
    tags: Vec<String>,
}

impl App {
    fn new(server_url: String) -> Self {
        Self {
            mode: AppMode::Login,
            token: None,
            documents: Vec::new(),
            selected: 0,
            search_query: String::new(),
            search_results: Vec::new(),
            status_message: String::new(),
            server_url,
            client: Client::new(),
        }
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let resp = self
            .client
            .post(format!("{}/api/v1/auth/login", self.server_url))
            .json(&serde_json::json!({
                "username": username,
                "password": password
            }))
            .send()
            .await?;

        let data: Value = resp.json().await?;
        if data["success"].as_bool().unwrap_or(false) {
            self.token = data["access_token"].as_str().map(String::from);
            self.status_message = "Login successful".to_string();
            self.mode = AppMode::Documents;
            self.load_documents().await?;
        } else {
            self.status_message = "Login failed".to_string();
        }
        Ok(())
    }

    async fn load_documents(&mut self) -> Result<()> {
        if let Some(token) = &self.token {
            let resp = self
                .client
                .get(format!("{}/api/v1/documents", self.server_url))
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;

            let data: Value = resp.json().await?;
            if let Some(docs) = data["results"].as_array() {
                self.documents = docs
                    .iter()
                    .map(|d| Document {
                        id: d["id"].as_str().unwrap_or("").to_string(),
                        title: d["title"].as_str().unwrap_or("").to_string(),
                        content: d["content"].as_str().unwrap_or("").to_string(),
                        tags: d["tags"]
                            .as_array()
                            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                    })
                    .collect();
            }
        }
        Ok(())
    }

    async fn search(&mut self, query: &str) -> Result<()> {
        if let Some(token) = &self.token {
            let resp = self
                .client
                .get(format!("{}/api/v1/search?q={}", self.server_url, query))
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;

            let data: Value = resp.json().await?;
            if let Some(results) = data["results"].as_array() {
                self.search_results = results
                    .iter()
                    .map(|d| Document {
                        id: d["id"].as_str().unwrap_or("").to_string(),
                        title: d["title"].as_str().unwrap_or("").to_string(),
                        content: d["content"].as_str().unwrap_or("").to_string(),
                        tags: Vec::new(),
                    })
                    .collect();
                self.status_message = format!("Found {} results", self.search_results.len());
            }
        }
        Ok(())
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let header = Block::default()
        .title("Tachyon TUI")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    // Main content
    match app.mode {
        AppMode::Login => render_login(f, app, chunks[1]),
        AppMode::Documents => render_documents(f, app, chunks[1]),
        AppMode::DocumentView => render_document_view(f, app, chunks[1]),
        AppMode::Graph => render_graph(f, app, chunks[1]),
        AppMode::Search => render_search(f, app, chunks[1]),
        AppMode::Help => render_help(f, app, chunks[1]),
    }

    // Status bar
    let status = Paragraph::new(Line::from(vec![
        Span::raw(&app.status_message),
        Span::raw(" | h:Help q:Quit"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}

fn render_login(f: &mut Frame, _app: &App, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from("  Tachyon TUI Client"),
        Line::from(""),
        Line::from("  Press Enter to login with default credentials"),
        Line::from("  or 'q' to quit"),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Login").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_documents(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .documents
        .iter()
        .enumerate()
        .map(|(i, doc)| {
            let style = if i == app.selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(doc.title.as_str(), style)))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Documents").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let mut state = ListState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_document_view(f: &mut Frame, app: &App, area: Rect) {
    if let Some(doc) = app.documents.get(app.selected) {
        let text = vec![
            Line::from(Span::styled(
                &doc.title,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(doc.content.as_str()),
        ];
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(format!("Document: {}", doc.title))
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }
}

fn render_graph(f: &mut Frame, _app: &App, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from("  Knowledge Graph View"),
        Line::from(""),
        Line::from("  (Graph visualization coming soon)"),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Graph").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_search(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let input = Paragraph::new(Line::from(vec![
        Span::styled("Search: ", Style::default().fg(Color::Yellow)),
        Span::raw(&app.search_query),
        Span::styled("_", Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(input, chunks[0]);

    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .map(|doc| ListItem::new(Line::from(doc.title.as_str())))
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!("Results ({})", app.search_results.len()))
            .borders(Borders::ALL),
    );
    f.render_widget(list, chunks[1]);
}

fn render_help(f: &mut Frame, _app: &App, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Tachyon TUI Help",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  j/k or arrows  - Move up/down"),
        Line::from("  Enter          - Select/Open"),
        Line::from("  Esc            - Go back"),
        Line::from(""),
        Line::from("Modes:"),
        Line::from("  d - Documents"),
        Line::from("  g - Graph"),
        Line::from("  / - Search"),
        Line::from("  h - Help"),
        Line::from("  q - Quit"),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(args.server.clone());

    // Auto-login
    app.login(&args.username, &args.password).await?;

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        AppMode::Login => match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Enter => {
                                app.login(&args.username, &args.password).await?;
                            }
                            _ => {}
                        },
                        AppMode::Documents => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('j') | KeyCode::Down => {
                                if app.selected < app.documents.len().saturating_sub(1) {
                                    app.selected += 1;
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                app.selected = app.selected.saturating_sub(1);
                            }
                            KeyCode::Enter => {
                                app.mode = AppMode::DocumentView;
                            }
                            KeyCode::Char('d') => {
                                app.mode = AppMode::Documents;
                                app.load_documents().await?;
                            }
                            KeyCode::Char('g') => {
                                app.mode = AppMode::Graph;
                            }
                            KeyCode::Char('/') => {
                                app.mode = AppMode::Search;
                            }
                            KeyCode::Char('h') => {
                                app.mode = AppMode::Help;
                            }
                            _ => {}
                        },
                        AppMode::DocumentView => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.mode = AppMode::Documents;
                            }
                            _ => {}
                        },
                        AppMode::Graph => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.mode = AppMode::Documents;
                            }
                            _ => {}
                        },
                        AppMode::Search => match key.code {
                            KeyCode::Esc => {
                                app.mode = AppMode::Documents;
                            }
                            KeyCode::Enter => {
                                app.search(&app.search_query.clone()).await?;
                            }
                            KeyCode::Char(c) => {
                                app.search_query.push(c);
                            }
                            KeyCode::Backspace => {
                                app.search_query.pop();
                            }
                            _ => {}
                        },
                        AppMode::Help => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') => {
                                app.mode = AppMode::Documents;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
