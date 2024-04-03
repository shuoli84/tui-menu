# tui-menu

A menu widget for [Ratatui](https://crates.io/crates/ratatui).

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
f.render_stateful_widget(menu, chunks[0], &mut app.menu);
```

### Create nested menu tree

```rust
let menu = MenuState::new(vec![
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
]);
```

### Consume events

``` rust
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
```
