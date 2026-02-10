use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Modules,
    Config,
    Status,
    Audit,
}

struct App {
    current_tab: Tab,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            current_tab: Tab::Modules,
            should_quit: false,
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Modules => Tab::Config,
            Tab::Config => Tab::Status,
            Tab::Status => Tab::Audit,
            Tab::Audit => Tab::Modules,
        };
    }

    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Modules => Tab::Audit,
            Tab::Config => Tab::Modules,
            Tab::Status => Tab::Config,
            Tab::Audit => Tab::Status,
        };
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Main loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Create layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(size);

            // Header
            let header = Paragraph::new("S1bCr4ft TUI - Declarative System Configuration")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // Tabs
            let tabs = vec![
                ("1", "Modules"),
                ("2", "Config"),
                ("3", "Status"),
                ("4", "Audit"),
            ];

            let tab_titles: Vec<Line> = tabs
                .iter()
                .enumerate()
                .map(|(i, (key, title))| {
                    let style = if i == app.current_tab as usize {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    Line::from(vec![
                        Span::styled(format!("[{}] ", key), Style::default().fg(Color::Green)),
                        Span::styled(*title, style),
                        Span::raw("  "),
                    ])
                })
                .collect();

            let tabs_widget = Paragraph::new(tab_titles)
                .block(Block::default().borders(Borders::ALL).title("Navigation"));

            // Content area
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(chunks[1]);

            f.render_widget(tabs_widget, content_chunks[0]);

            // Render current tab content
            match app.current_tab {
                Tab::Modules => render_modules_tab(f, content_chunks[1]),
                Tab::Config => render_config_tab(f, content_chunks[1]),
                Tab::Status => render_status_tab(f, content_chunks[1]),
                Tab::Audit => render_audit_tab(f, content_chunks[1]),
            }

            // Footer
            let footer = Paragraph::new(
                "q: Quit | Tab/Shift+Tab: Switch tabs | ↑↓: Navigate | Enter: Select",
            )
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.previous_tab(),
                    KeyCode::Char('1') => app.current_tab = Tab::Modules,
                    KeyCode::Char('2') => app.current_tab = Tab::Config,
                    KeyCode::Char('3') => app.current_tab = Tab::Status,
                    KeyCode::Char('4') => app.current_tab = Tab::Audit,
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn render_modules_tab(f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = vec![
        "core/base-system",
        "core/bootloader",
        "development/languages/rust",
        "linux-optimization/window-managers/hyprland-config",
        "red-team/c2-frameworks/sliver-c2",
        "malware-analysis/static-analysis/ghidra-setup",
        "ai-ml/ollama-setup",
    ]
    .iter()
    .map(|m| ListItem::new(*m).style(Style::default().fg(Color::White)))
    .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Available Modules (57 total)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

fn render_config_tab(f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let config_text = vec![
        Line::from("version: \"1.0\""),
        Line::from("name: \"my-arch-setup\""),
        Line::from(""),
        Line::from("modules:"),
        Line::from("  - core/base-system"),
        Line::from("  - linux-optimization/window-managers/hyprland-config"),
        Line::from(""),
        Line::from("options:"),
        Line::from("  auto_backup: true"),
        Line::from("  parallel_install: true"),
    ];

    let paragraph = Paragraph::new(config_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Configuration"),
        )
        .style(Style::default().fg(Color::Green));

    f.render_widget(paragraph, area);
}

fn render_status_tab(f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let status_text = vec![
        Line::from(Span::styled(
            "✓ System Status: OK",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Installed Modules: 12"),
        Line::from("Total Packages: 847"),
        Line::from("Last Sync: 2 hours ago"),
        Line::from("Backups: 5"),
        Line::from(""),
        Line::from(Span::styled(
            "Package Manager: paru",
            Style::default().fg(Color::Cyan),
        )),
    ];

    let paragraph = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("System Status"),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

fn render_audit_tab(f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let audit_entries = vec![
        "[2024-02-04 20:15:32] sync - SUCCESS - Installed 15 packages",
        "[2024-02-04 19:45:12] backup_create - SUCCESS - Backup created: abc123",
        "[2024-02-04 18:30:45] module_add - SUCCESS - Added hyprland-config",
        "[2024-02-04 17:22:18] sync - SUCCESS - Installed 3 packages",
        "[2024-02-04 16:10:05] config_change - SUCCESS - Updated config.yml",
    ];

    let items: Vec<ListItem> = audit_entries
        .iter()
        .map(|entry| ListItem::new(*entry).style(Style::default().fg(Color::Yellow)))
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Audit Log (Recent Entries)"),
    );

    f.render_widget(list, area);
}
