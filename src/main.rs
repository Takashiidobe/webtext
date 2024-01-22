use html2text::from_read;
use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
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
}

fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::default();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('j') => {
                        app.vertical_scroll = app.vertical_scroll.saturating_add(1);
                        app.vertical_scroll_state =
                            app.vertical_scroll_state.position(app.vertical_scroll);
                    }
                    KeyCode::Char('k') => {
                        app.vertical_scroll = app.vertical_scroll.saturating_sub(1);
                        app.vertical_scroll_state =
                            app.vertical_scroll_state.position(app.vertical_scroll);
                    }
                    KeyCode::Char('h') => {
                        app.horizontal_scroll = app.horizontal_scroll.saturating_sub(1);
                        app.horizontal_scroll_state =
                            app.horizontal_scroll_state.position(app.horizontal_scroll);
                    }
                    KeyCode::Char('l') => {
                        app.horizontal_scroll = app.horizontal_scroll.saturating_add(1);
                        app.horizontal_scroll_state =
                            app.horizontal_scroll_state.position(app.horizontal_scroll);
                    }
                    KeyCode::Char('s') => app.show_popup = !app.show_popup,
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

use std::io::{stdout, Result, Write};
use tui_input::backend::crossterm as backend;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

// fn show_input() -> io::Result<String> {
//     let stdout = stdout();
//     let mut stdout = stdout.lock();
//     let mut input: Input = "https://".into();
//     backend::write(&mut stdout, input.value(), input.cursor(), (0, 0), 15)?;
//     stdout.flush()?;
//
//     loop {
//         let event = read()?;
//
//         if let Event::Key(KeyEvent { code, .. }) = event {
//             match code {
//                 KeyCode::Esc | KeyCode::Enter => {
//                     break;
//                 }
//                 _ => {
//                     if input.handle_event(&event).is_some() {
//                         backend::write(&mut stdout, input.value(), input.cursor(), (0, 0), 15)?;
//                         stdout.flush()?;
//                     }
//                 }
//             }
//         }
//     }
//     Ok(input.to_string())
// }

fn ui(f: &mut Frame, app: &mut App) {
    use std::env::args;
    let arguments: Vec<_> = args().collect();
    let url = arguments[1].to_string();
    let size = f.size();

    // if app.show_popup {
    // let new_url = show_input().unwrap();
    // url = new_url;
    //  app.show_popup = false;
    // }

    let html = reqwest::blocking::get(url.to_owned())
        .unwrap()
        .bytes()
        .unwrap();
    let parsed = from_read(&html[..], size.width.into());

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
        .block(create_block("Vertical scrollbar with arrows"))
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
