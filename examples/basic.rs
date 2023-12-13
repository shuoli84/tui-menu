use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame, Terminal,
};
use std::{borrow::Cow, error::Error, io};
use tui_menu::{Menu, MenuItem, MenuState};

struct App {
    menu: tui_menu::MenuState<Cow<'static, str>>,
}

impl App {
    fn new() -> App {
        App {
            menu: MenuState::new(vec![
                MenuItem::group(
                    "File",
                    vec![
                        MenuItem::item("New", "file.new".into()),
                        MenuItem::item("Open", "file.open".into()),
                        MenuItem::group(
                            "Open recent",
                            vec!["file_1.txt", "file_2.txt"]
                                .into_iter()
                                .map(|f| MenuItem::item(f, format!("file.recent:{f}").into()))
                                .collect(),
                        ),
                        MenuItem::item("Save as", "file.save_as".into()),
                        MenuItem::item("Exit", "exit".into()),
                    ],
                ),
                MenuItem::group(
                    "Edit",
                    vec![
                        MenuItem::item("Copy", "edit.new".into()),
                        MenuItem::item("Cut", "edit.cut".into()),
                        MenuItem::item("Paste", "edit.paste".into()),
                    ],
                ),
                MenuItem::group(
                    "About",
                    vec![
                        MenuItem::item("Author", "about.author".into()),
                        MenuItem::item("Help", "about.help".into()),
                    ],
                ),
            ]),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => app.menu.left(),
                    KeyCode::Char('l') | KeyCode::Right => app.menu.right(),
                    KeyCode::Char('j') | KeyCode::Down => app.menu.down(),
                    KeyCode::Char('k') | KeyCode::Up => app.menu.up(),
                    KeyCode::Esc => app.menu.reset(),
                    KeyCode::Enter => app.menu.select(),
                    _ => {}
                }
            }
        }

        for e in app.menu.drain_events() {
            match e {
                tui_menu::MenuEvent::Selected(item) => match item.as_ref() {
                    "exit" => {
                        return Ok(());
                    }
                    _ => {
                        // println!("{} selected", item);
                    }
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Percentage(80)].as_ref())
        .split(size);

    let block = Block::default()
        .title("Content")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    f.render_widget(block, chunks[1]);

    // menu should be draw at last, so it can stay on top of other content
    let menu = Menu::new();
    f.render_stateful_widget(menu, chunks[0], &mut app.menu);
}
