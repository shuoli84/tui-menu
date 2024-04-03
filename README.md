# tui-menu

A menu widget for [Ratatui](https://crates.io/crates/ratatui).

![Demo](https://vhs.charm.sh/vhs-3pN84WzBTmPTbU2SCtvUiV.gif)

## Features

- Sub menu groups.
- Intuitive movement.
- Item's data is generic as long as it ```Clone```able.

## Try

``` bash
cargo run --example basic
```

## Example

take a look at examples/basic.rs

### Render

```rust
// menu should be draw at last, so it can stay on top of other content
let menu = Menu::new();
frame.render_stateful_widget(menu, chunks[0], &mut app.menu);
```

### Create nested menu tree

Note: MenuItems can be created from any type that implements `Clone`. Using an enum is just one
option which can work. You could use strings or your own state types.

```rust
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

let menu = MenuState::new(vec![
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
```

### Consume events

``` rust
for e in menu.drain_events() {
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
    // close the menu once the event has been handled.
    menu.reset();
}
```
