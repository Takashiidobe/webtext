use html2text::from_read;
use std::{
    error::Error,
    io::{self},
    time::{Duration, Instant},
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

#[derive(Default)]
struct App {
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub horizontal_scroll: usize,
    pub show_popup: bool,
    pub length: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::default();
    run_app(&mut terminal, app, tick_rate)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn vec_to_num(vec: &[usize]) -> usize {
    let res = vec.iter().fold(0, |acc, elem| acc * 10 + elem);
    if res == 0 {
        1
    } else {
        res
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut html = None;
    let mut last_tick = Instant::now();
    let mut buffer = vec![];
    loop {
        if html.is_some() {
            terminal.draw(|f| {
                app.length = f.size().width;
                ui(f, &mut app, html.clone())
            })?;
        }

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('j') => {
                        let num = vec_to_num(&buffer);
                        app.vertical_scroll = app.vertical_scroll.saturating_add(num);
                        app.vertical_scroll_state =
                            app.vertical_scroll_state.position(app.vertical_scroll);
                        buffer.clear();
                    }
                    KeyCode::Char('k') => {
                        let num = vec_to_num(&buffer);
                        app.vertical_scroll = app.vertical_scroll.saturating_sub(num);
                        app.vertical_scroll_state =
                            app.vertical_scroll_state.position(app.vertical_scroll);
                        buffer.clear();
                    }
                    KeyCode::Char('h') => {
                        let num = vec_to_num(&buffer);
                        app.horizontal_scroll = app.horizontal_scroll.saturating_sub(num);
                        app.horizontal_scroll_state =
                            app.horizontal_scroll_state.position(app.horizontal_scroll);
                        buffer.clear();
                    }
                    KeyCode::Char('l') => {
                        let num = vec_to_num(&buffer);
                        app.horizontal_scroll = app.horizontal_scroll.saturating_add(num);
                        app.horizontal_scroll_state =
                            app.horizontal_scroll_state.position(app.horizontal_scroll);
                        buffer.clear();
                    }
                    KeyCode::Char('g') => {
                        app.vertical_scroll = 0;
                        app.vertical_scroll_state =
                            app.vertical_scroll_state.position(app.vertical_scroll);
                    }
                    KeyCode::Char('G') => {
                        if let Some(ref h) = html {
                            app.vertical_scroll = h.lines().count();
                            app.vertical_scroll_state =
                                app.vertical_scroll_state.position(app.vertical_scroll);
                        }
                    }
                    KeyCode::Char('s') => {
                        use std::io::{stdout, Write};
                        use tui_input::backend::crossterm as backend;

                        app.show_popup = !app.show_popup;
                        let mut input: Input = "".into();
                        let stdout = stdout();
                        let mut stdout = stdout.lock();
                        backend::write(
                            &mut stdout,
                            input.value(),
                            input.cursor(),
                            (0, 0),
                            app.length,
                        )?;
                        stdout.flush()?;

                        loop {
                            let event = read()?;

                            if let Event::Key(KeyEvent { code, .. }) = event {
                                match code {
                                    KeyCode::Esc | KeyCode::Enter => {
                                        break;
                                    }
                                    _ => {
                                        if input.handle_event(&event).is_some() {
                                            backend::write(
                                                &mut stdout,
                                                input.value(),
                                                input.cursor(),
                                                (0, 0),
                                                15,
                                            )?;
                                            stdout.flush()?;
                                        }
                                    }
                                }
                            }
                        }
                        let client = reqwest::blocking::Client::new();
                        let response =
                        client.get(input.to_string()).header("User-Agent","User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0").header("Accept", "text/html")
                                    .send()
                                    .unwrap().text().unwrap().trim().to_string();
                        let mut response_html = String::default();
                        for line in response.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }
                            response_html.push_str(line.trim());
                            response_html.push('\n');
                        }
                        html = Some(response_html);
                    }
                    KeyCode::Char(c) => {
                        if c.is_ascii_digit() {
                            buffer.push(c.to_digit(10).unwrap() as usize);
                        }
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &mut App, html: Option<String>) {
    let Some(h) = html else { return };
    let size = f.size();
    let terminal_width = size.width;

    let parsed = from_read(h.as_bytes(), terminal_width.saturating_sub(3).into());

    let block = Block::default().black();
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Percentage(100)])
        .split(size);

    let text: Vec<_> = parsed.lines().map(Line::from).collect();
    app.vertical_scroll_state = app.vertical_scroll_state.content_length(text.len());
    app.horizontal_scroll_state = app
        .horizontal_scroll_state
        .content_length(size.height.into());

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .gray()
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    let title = Block::default()
        .title("Use h j k l to scroll ◄ ▲ ▼ ►")
        .title_alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let paragraph = Paragraph::new(text.clone())
        .gray()
        .block(create_block(
            h.lines()
                .find(|l| l.starts_with("<title>"))
                .unwrap_or("<title>Unknown Title</title>")
                .strip_prefix("<title>")
                .unwrap()
                .strip_suffix("</title>")
                .unwrap(),
        ))
        .scroll((app.vertical_scroll as u16, 0));
    f.render_widget(paragraph, chunks[1]);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        chunks[1],
        &mut app.vertical_scroll_state,
    );
}
