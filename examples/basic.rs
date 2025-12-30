use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::{Buffer, Constraint, Layout, Rect, StatefulWidget, Stylize, Widget},
    widgets::{Block, Paragraph},
};
use tui_menu::{Menu, MenuEvent, MenuItem, MenuState};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    ratatui::run(|t| App::new().run(t))?;
    Ok(())
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
                    "File",
                    vec![
                        MenuItem::item("New", Action::FileNew),
                        MenuItem::item("Open", Action::FileOpen),
                        MenuItem::group(
                            "Open recent",
                            ["file_1.txt", "file_2.txt"]
                                .iter()
                                .map(|&f| MenuItem::item(f, Action::FileOpenRecent(f.into())))
                                .collect(),
                        ),
                        MenuItem::item("Save as", Action::FileSaveAs),
                        MenuItem::item("Exit", Action::Exit),
                    ],
                ),
                MenuItem::group(
                    "Edit",
                    vec![
                        MenuItem::item("Copy", Action::EditCopy),
                        MenuItem::item("Cut", Action::EditCut),
                        MenuItem::item("Paste", Action::EditPaste),
                    ],
                ),
                MenuItem::group(
                    "About",
                    vec![
                        MenuItem::item("Author", Action::AboutAuthor),
                        MenuItem::item("Help", Action::AboutHelp),
                    ],
                ),
            ]),
        }
    }
}

#[derive(Debug, Clone)]
enum Action {
    FileNew,
    FileOpen,
    FileOpenRecent(String),
    FileSaveAs,
    Exit,
    EditCopy,
    EditCut,
    EditPaste,
    AboutAuthor,
    AboutHelp,
}

impl App {
    fn run(mut self, terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

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
                        Action::FileNew => {
                            self.content.clear();
                        }
                        Action::FileOpenRecent(file) => {
                            self.content = format!("content of {file}");
                        }
                        action => {
                            self.content = format!("{action:?} not implemented");
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

        "tui-menu"
            .bold()
            .blue()
            .into_centered_line()
            .render(top, buf);

        // draw menu last, so it renders on top of other content
        Menu::new().render(top, buf, &mut self.menu);
    }
}
