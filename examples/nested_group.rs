use color_eyre::config::HookBuilder;
use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    widgets::{Block, Paragraph},
};
use std::io::{self, stdout, Stdout};
use tui_menu::{Menu, MenuEvent, MenuItem, MenuState};

fn main() -> color_eyre::Result<()> {
    let mut terminal = init_terminal()?;
    App::new().run(&mut terminal)?;
    restore_terminal()?;
    Ok(())
}

/// Install panic and error hooks that restore the terminal before printing the error.
pub fn init_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();

    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal(); // ignore failure to restore terminal
        panic(info);
    }));
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal(); // ignore failure to restore terminal
        error(e)
    }))?;

    Ok(())
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen,)
}

struct App {
    content: String,
    menu: MenuState<Action>,
}

impl App {
    fn new() -> Self {
        Self {
            content: String::new(),
            menu: MenuState::new(vec![
                MenuItem::group(
                    "Group 1",
                    vec![
                        MenuItem::group(
                            "Nested",
                            vec![MenuItem::group(
                                "Nested 1",
                                vec![MenuItem::group(
                                    "Nested 2",
                                    vec![
                                        MenuItem::group(
                                            "Nested 3",
                                            vec![MenuItem::group(
                                                "Nested 4",
                                                vec![
                                                    MenuItem::group(
                                                        "Nested 5",
                                                        vec![MenuItem::group("Nested 6", vec![])],
                                                    ),
                                                    MenuItem::item("Item 5", Action::Exit),
                                                ],
                                            )],
                                        ),
                                        MenuItem::item("Item 3a", Action::Exit),
                                    ],
                                )],
                            )],
                        ),
                        MenuItem::item("Exit", Action::Exit),
                    ],
                ),
                MenuItem::item("Exit", Action::Exit),
            ]),
        }
    }
}

#[derive(Debug, Clone)]
enum Action {
    Exit,
}

impl App {
    fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.size()))?;

            if event::poll(std::time::Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    self.on_key_event(key);
                }
            }

            for e in self.menu.drain_events() {
                match e {
                    MenuEvent::Selected(item) => match item {
                        Action::Exit => {
                            return Ok(());
                        }
                    },
                }
                self.menu.reset();
            }
        }
    }

    fn on_key_event(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => self.menu.left(),
            KeyCode::Char('l') | KeyCode::Right => self.menu.right(),
            KeyCode::Char('j') | KeyCode::Down => self.menu.down(),
            KeyCode::Char('k') | KeyCode::Up => self.menu.up(),
            KeyCode::Esc => self.menu.reset(),
            KeyCode::Enter => self.menu.select(),
            _ => {}
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;
        let [top, main] = Layout::vertical([Length(1), Fill(1)]).areas(area);

        Paragraph::new(self.content.as_str())
            .block(Block::bordered().title("Content").on_black())
            .render(main, buf);

        let [log, menu] = Layout::horizontal([Length(10), Fill(1)]).areas(area);
        "tui-menu"
            .bold()
            .blue()
            .into_centered_line()
            .render(log, buf);

        // draw menu last, so it renders on top of other content
        Menu::new().render(menu, buf, &mut self.menu);
    }
}
