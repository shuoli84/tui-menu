use std::marker::PhantomData;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Clear, StatefulWidget, Widget},
};

/// Events this widget produce
/// Now only emit Selected, may add few in future
#[derive(Debug)]
pub enum MenuEvent<T> {
    /// Item selected, with its data attached
    Selected(T),
}

/// The state for menu, keep track of runtime info
pub struct MenuState<T> {
    /// stores the menu tree
    root_item: MenuItem<T>,
    /// stores events generated in one frame
    events: Vec<MenuEvent<T>>,
}

impl<T: Clone> MenuState<T> {
    /// create with items
    /// # Example
    ///
    /// ```
    /// use tui_menu::{MenuState, MenuItem};
    ///
    /// let state = MenuState::<&'static str>::new(vec![
    ///     MenuItem::item("Foo", "label_foo"),
    ///     MenuItem::group("Group", vec![
    ///         MenuItem::item("Bar 1", "label_bar_1"),
    ///         MenuItem::item("Bar 2", "label_bar_1"),
    ///     ])
    /// ]);
    /// ```
    pub fn new(items: Vec<MenuItem<T>>) -> Self {
        Self {
            root_item: MenuItem {
                name: "root".into(),
                data: None,
                children: items,
                // the root item marked as always highlight
                // this makes highlight logic more consistent
                is_highlight: true,
            },
            events: Default::default(),
        }
    }

    /// active the menu, this will select the first item
    ///
    /// # Example
    ///
    /// ```
    /// use tui_menu::{MenuState, MenuItem};
    ///
    /// let mut state = MenuState::<&'static str>::new(vec![
    ///     MenuItem::item("Foo", "label_foo"),
    ///     MenuItem::group("Group", vec![
    ///         MenuItem::item("Bar 1", "label_bar_1"),
    ///         MenuItem::item("Bar 2", "label_bar_1"),
    ///     ])
    /// ]);
    ///
    /// state.activate();
    ///
    /// assert_eq!(state.highlight().unwrap().data.unwrap(), "label_foo");
    ///
    /// ```
    ///
    pub fn activate(&mut self) {
        self.root_item.select_next();
    }

    /// trigger up movement
    /// NOTE: this action tries to do intuitive movement,
    /// which means logicially it is not consistent, e.g:
    /// case 1:
    ///    group 1        group 2        group 3
    ///                 > sub item 1
    ///                   sub item 2
    /// up is pop, which closes the group 2
    ///
    /// case 2:
    ///    group 1        group 2        group 3
    ///                   sub item 1
    ///                 > sub item 2
    /// up is move prev
    ///
    /// case 3:
    ///
    ///    group 1        group 2   
    ///                   sub item 1
    ///                 > sub item 2  > sub sub item 1
    ///                                 sub sub item 2
    ///
    /// up does nothing
    pub fn up(&mut self) {
        match self.active_depth() {
            0 | 1 => {
                // do nothing
            }
            2 => match self
                .root_item
                .highlight_child()
                .and_then(|child| child.highlight_child_index())
            {
                // case 1
                Some(idx) if idx == 0 => {
                    self.pop();
                }
                _ => {
                    self.prev();
                }
            },
            _ => {
                self.prev();
            }
        }
    }

    pub fn down(&mut self) {
        if self.active_depth() == 1 {
            self.push();
        } else {
            self.next();
        }
    }

    pub fn left(&mut self) {
        if self.active_depth() == 1 {
            self.prev();
        } else if self.active_depth() == 2 {
            self.pop();
            self.prev();
        } else {
            self.pop();
        }
    }

    pub fn right(&mut self) {
        if self.active_depth() == 1 {
            self.next();
        } else if self.active_depth() == 2 {
            if self.push().is_none() {
                // special handling, make menu navigation
                // more productive
                self.pop();
                self.next();
            }
        } else {
            self.push();
        }
    }

    pub fn prev(&mut self) {
        if let Some(item) = self.root_item.highlight_last_but_one() {
            item.select_prev();
        } else {
            self.root_item.select_prev();
        }
    }

    pub fn next(&mut self) {
        if let Some(item) = self.root_item.highlight_last_but_one() {
            item.select_next();
        } else {
            self.root_item.select_next();
        }
    }

    /// active depth, how many levels dropdown/sub menus expanded.
    /// when no drop down, it is 1
    /// one drop down, 2
    fn active_depth(&self) -> usize {
        let mut item = self.root_item.highlight_child();
        let mut depth = 0;
        while let Some(inner_item) = item {
            depth += 1;
            item = inner_item.highlight_child();
        }
        depth
    }

    /// select current highlight item, if it has children
    /// then push
    pub fn select(&mut self) {
        if let Some(item) = self.root_item.highlight_mut() {
            if !item.children.is_empty() {
                self.push();
            } else {
                if let Some(ref data) = item.data {
                    self.events.push(MenuEvent::Selected(data.clone()));
                }
            }
        }
    }

    /// dive into sub menu if applicable.
    /// Return: Some if entered deeper level
    ///         None if nothing happen
    pub fn push(&mut self) -> Option<()> {
        self.root_item.highlight_mut()?.select_first_child()
    }

    pub fn pop(&mut self) {
        if let Some(item) = self.root_item.highlight_mut() {
            item.clear_highlight();
        }
    }

    pub fn reset(&mut self) {
        self.root_item
            .children
            .iter_mut()
            .for_each(|c| c.clear_highlight());
    }

    /// client should drain events each frame, otherwise user action
    /// will feel laggy
    pub fn drain_events(&mut self) -> impl Iterator<Item = MenuEvent<T>> {
        std::mem::take(&mut self.events).into_iter()
    }

    /// return current highlighted item's reference
    pub fn highlight(&self) -> Option<&MenuItem<T>> {
        self.root_item.highlight()
    }
}

pub struct MenuItem<T> {
    pub name: String,
    pub data: Option<T>,
    children: Vec<MenuItem<T>>,
    is_highlight: bool,
}

impl<T> MenuItem<T> {
    pub fn item(name: impl Into<String>, data: T) -> Self {
        Self {
            name: name.into(),
            data: Some(data),
            is_highlight: false,
            children: vec![],
        }
    }

    pub fn group(name: impl Into<String>, children: Vec<Self>) -> Self {
        Self {
            name: name.into(),
            data: None,
            is_highlight: false,
            children,
        }
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn select_first_child(&mut self) -> Option<()> {
        if !self.children.is_empty() {
            if let Some(it) = self.children.iter_mut().nth(0) {
                it.is_highlight = true;
            }
            Some(())
        } else {
            None
        }
    }

    /// select prev item in this node
    fn select_prev(&mut self) {
        // if no child selected, then
        let Some(current_index) = self.highlight_child_index() else {
            self.select_first_child();
            return;
        };

        let index_to_highlight = if current_index > 0 {
            current_index - 1
        } else {
            0
        };

        self.children[current_index].clear_highlight();
        self.children[index_to_highlight].is_highlight = true;
    }

    /// select prev item in this node
    pub fn select_next(&mut self) {
        // if no child selected, then
        let Some(current_index) = self.highlight_child_index() else {
            self.select_first_child();
            return;
        };

        let index_to_highlight = (current_index + 1).min(self.children.len() - 1);
        self.children[current_index].clear_highlight();
        self.children[index_to_highlight].is_highlight = true;
    }

    fn highlight_child_index(&self) -> Option<usize> {
        for (idx, child) in self.children.iter().enumerate() {
            if child.is_highlight {
                return Some(idx);
            }
        }

        None
    }

    /// if any child highlighted, then return its reference
    fn highlight_child(&self) -> Option<&Self> {
        self.children.iter().filter(|i| i.is_highlight).nth(0)
    }

    /// if any child highlighted, then return its reference
    fn highlight_child_mut(&mut self) -> Option<&mut Self> {
        self.children.iter_mut().filter(|i| i.is_highlight).nth(0)
    }

    /// clear is_highlight flag recursively.
    pub fn clear_highlight(&mut self) {
        self.is_highlight = false;
        for child in self.children.iter_mut() {
            child.clear_highlight();
        }
    }

    /// return deepest highlight item's reference
    pub fn highlight(&self) -> Option<&Self> {
        if !self.is_highlight {
            return None;
        }

        let mut highlight_item = self;
        while highlight_item.highlight_child().is_some() {
            highlight_item = highlight_item.highlight_child().unwrap();
        }

        Some(highlight_item)
    }

    /// mut version of highlight
    pub fn highlight_mut(&mut self) -> Option<&mut Self> {
        if !self.is_highlight {
            return None;
        }

        let mut highlight_item = self;
        while highlight_item.highlight_child_mut().is_some() {
            highlight_item = highlight_item.highlight_child_mut().unwrap();
        }

        Some(highlight_item)
    }

    /// last but one layer in highlight
    pub fn highlight_last_but_one(&mut self) -> Option<&mut Self> {
        // if self is not highlighted or there is no highlighed child, return None
        if !self.is_highlight || self.highlight_child_mut().is_none() {
            return None;
        }

        let mut last_but_one = self;
        while last_but_one
            .highlight_child_mut()
            .map(|x| x.highlight_child_mut())
            .flatten()
            .is_some()
        {
            last_but_one = last_but_one.highlight_child_mut().unwrap();
        }
        Some(last_but_one)
    }
}

pub struct Menu<T> {
    default_style: Style,
    highlight_style: Style,
    drop_down_width: u16,
    drop_down_style: Style,
    _priv: PhantomData<T>,
}

impl<T> Menu<T> {
    pub fn new() -> Self {
        Self {
            highlight_style: Style::default().fg(Color::White).bg(Color::LightBlue),
            default_style: Style::default().fg(Color::White),
            drop_down_width: 20,
            drop_down_style: Style::default().bg(Color::DarkGray),
            _priv: Default::default(),
        }
    }

    fn render_drop_down(
        &self,
        x: u16,
        y: u16,
        group: &[MenuItem<T>],
        buf: &mut tui::buffer::Buffer,
        depth: usize,
    ) {
        let area = Rect::new(x, y, self.drop_down_width, group.len() as u16);
        Clear.render(area, buf);
        buf.set_style(area, self.drop_down_style);

        for (idx, item) in group.iter().enumerate() {
            let item_y = y + idx as u16;
            let is_active = item.is_highlight;

            buf.set_span(
                x as u16,
                item_y,
                &Span::styled(
                    item.name(),
                    if is_active {
                        self.highlight_style
                    } else {
                        self.default_style
                    },
                ),
                self.drop_down_width,
            );

            // show children
            if is_active && !item.children.is_empty() {
                self.render_drop_down(
                    x + self.drop_down_width,
                    item_y,
                    &item.children,
                    buf,
                    depth + 1,
                );
            }
        }
    }
}

impl<T> StatefulWidget for Menu<T> {
    type State = MenuState<T>;

    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer, state: &mut Self::State) {
        let mut spans = vec![];
        let mut x_pos = 0;
        let y_pos = area.y;

        for (idx, item) in state.root_item.children.iter().enumerate() {
            let is_highlight = item.is_highlight;
            let item_style = if is_highlight {
                self.highlight_style
            } else {
                self.default_style
            };
            let has_children = !item.children.is_empty();

            let group_x_pos = x_pos;
            let span = Span::styled(&item.name, item_style);
            x_pos += span.width();
            spans.push(span);

            if has_children && is_highlight {
                self.render_drop_down(group_x_pos as u16, y_pos as u16 + 1, &item.children, buf, 1);
            }

            if idx < state.root_item.children.len() - 1 {
                let span = Span::raw(" | ");
                x_pos += span.width();
                spans.push(span);
            }
        }
        buf.set_spans(area.x, area.y, &Spans::from(spans), x_pos as u16);
    }
}
