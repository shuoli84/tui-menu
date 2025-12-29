/*! Drop-down main menu for Ratatui.

Ratatui immediate mode split visual elements in a Widget and a WidgetState.
The first holds the configuration, and never changes.
The second holds the parts that can be affected
by user actions.

In this case the style is found in the Widget [Menu]
and the menu tree is found in WidgetState [MenuState].

The menu tree is built with one [MenuItem] per possible selection.
A MenuItem with children is called a group.

When a menu item is selected, this generates an event which
will be stored in MenuState.events.

To define a menu, see examples in [MenuState].
*/

use ratatui_core::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};
use ratatui_widgets::{block::Block, borders::Borders, clear::Clear};
use std::{borrow::Cow, marker::PhantomData};

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
        let mut root_item = MenuItem::group("root", items);
        // the root item marked as always highlight
        // this makes highlight logic more consistent
        root_item.is_highlight = true;

        Self {
            root_item,
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
        self.root_item.highlight_next();
    }

    /// Check if menu is active
    pub fn is_active(&self) -> bool {
        self.root_item.highlight().is_some()
    }

    /// trigger up movement
    /// NOTE: this action tries to do intuitive movement,
    /// which means logically it is not consistent, e.g:
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
                Some(0) => {
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

    /// trigger down movement
    ///
    /// NOTE: this action tries to do intuitive movement,
    /// which means logicially it is not consistent, e.g:
    /// case 1:
    ///    group 1      > group 2        group 3
    ///                   sub item 1
    ///                   sub item 2
    /// down is enter, which enter the sub group of group 2
    ///
    /// case 2:
    ///    group 1        group 2        group 3
    ///                   sub item 1
    ///                 > sub item 2
    /// down does nothing
    ///
    /// case 3:
    ///    group 1        group 2
    ///                 > sub item 1
    ///                   sub item 2
    ///
    /// down highlights "sub item 2"
    pub fn down(&mut self) {
        if self.active_depth() == 1 {
            self.push();
        } else {
            self.next();
        }
    }

    /// trigger left movement
    ///
    /// NOTE: this action tries to do intuitive movement,
    /// which means logicially it is not consistent, e.g:
    /// case 1:
    ///    group 1      > group 2        group 3
    ///                   sub item 1
    ///                   sub item 2
    /// left highlights "group 1"
    ///
    /// case 2:
    ///    group 1        group 2        group 3
    ///                   sub item 1
    ///                 > sub item 2
    /// left first pop "sub item group", then highlights "group 1"
    ///
    /// case 3:
    ///    group 1        group 2
    ///                 > sub item 1    sub sub item 1
    ///                   sub item 2  > sub sub item 2
    ///
    /// left pop "sub sub group"
    pub fn left(&mut self) {
        if self.active_depth() == 0 {
            // do nothing
        } else if self.active_depth() == 1 {
            self.prev();
        } else if self.active_depth() == 2 {
            self.pop();
            self.prev();
        } else {
            self.pop();
        }
    }

    /// trigger right movement
    ///
    /// NOTE: this action tries to do intuitive movement,
    /// which means logicially it is not consistent, e.g:
    /// case 1:
    ///    group 1      > group 2        group 3
    ///                   sub item 1
    ///                   sub item 2
    /// right highlights "group 3"
    ///
    /// case 2:
    ///    group 1        group 2        group 3
    ///                   sub item 1
    ///                 > sub item 2
    /// right pop group "sub item *", then highlights "group 3"
    ///
    /// case 3:
    ///    group 1        group 2        group 3
    ///                   sub item 1
    ///                 > sub item 2 +
    /// right pushes "sub sub item 2". this differs from case 2 that
    /// current highlighted item can be expanded
    pub fn right(&mut self) {
        if self.active_depth() == 0 {
            // do nothing
        } else if self.active_depth() == 1 {
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

    /// highlight the prev item in current group
    /// if already the first, then do nothing
    fn prev(&mut self) {
        if let Some(item) = self.root_item.highlight_last_but_one() {
            item.highlight_prev();
        } else {
            self.root_item.highlight_prev();
        }
    }

    /// highlight the next item in current group
    /// if already the last, then do nothing
    fn next(&mut self) {
        if let Some(item) = self.root_item.highlight_last_but_one() {
            item.highlight_next();
        } else {
            self.root_item.highlight_next();
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

    /// How many dropdown to render, including preview
    /// NOTE: If current group contains sub-group, in order to keep ui consistent,
    ///   even the sub-group not selected, its space is counted
    fn dropdown_count(&self) -> u16 {
        let mut node = &self.root_item;
        let mut count = 0;
        loop {
            match node.highlight_child() {
                None => {
                    return count;
                }
                Some(highlight_child) => {
                    if highlight_child.is_group() {
                        // highlighted child is a group, then it's children is previewed
                        count += 1;
                    } else if node.children.iter().any(|c| c.is_group()) {
                        // if highlighted item is not a group, but if sibling contains group
                        // in order to keep ui consistency, also count it
                        count += 1;
                    }

                    node = highlight_child;
                }
            }
        }
    }

    /// select current highlight item, if it has children
    /// then push
    pub fn select(&mut self) {
        if let Some(item) = self.root_item.highlight_mut() {
            if !item.children.is_empty() {
                self.push();
            } else if let Some(ref data) = item.data {
                self.events.push(MenuEvent::Selected(data.clone()));
            }
        }
    }

    /// dive into sub menu if applicable.
    /// Return: Some if entered deeper level
    ///         None if nothing happen
    pub fn push(&mut self) -> Option<()> {
        self.root_item.highlight_mut()?.highlight_first_child()
    }

    /// pop the current menu group. move one layer up
    pub fn pop(&mut self) {
        if let Some(item) = self.root_item.highlight_mut() {
            item.clear_highlight();
        }
    }

    /// clear all highlighted items. This is useful
    /// when the menu bar lose focus
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

/// MenuItem is the node in menu tree. If children is not
/// empty, then this item is the group item.
pub struct MenuItem<T> {
    name: Cow<'static, str>,
    pub data: Option<T>,
    children: Vec<MenuItem<T>>,
    is_highlight: bool,
}

impl<T> MenuItem<T> {
    /// helper function to create a non group item.
    pub fn item(name: impl Into<Cow<'static, str>>, data: T) -> Self {
        Self {
            name: name.into(),
            data: Some(data),
            is_highlight: false,
            children: vec![],
        }
    }

    /// helper function to create a group item.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_menu::MenuItem;
    ///
    /// let item = MenuItem::<&'static str>::group("group", vec![
    ///     MenuItem::item("foo", "label_foo"),
    /// ]);
    ///
    /// assert!(item.is_group());
    ///
    /// ```
    pub fn group(name: impl Into<Cow<'static, str>>, children: Vec<Self>) -> Self {
        Self {
            name: name.into(),
            data: None,
            is_highlight: false,
            children,
        }
    }

    #[cfg(test)]
    fn with_highlight(mut self, highlight: bool) -> Self {
        self.is_highlight = highlight;
        self
    }

    /// whether this item is group
    pub fn is_group(&self) -> bool {
        !self.children.is_empty()
    }

    /// get current item's name
    fn name(&self) -> &str {
        &self.name
    }

    /// highlight first child
    fn highlight_first_child(&mut self) -> Option<()> {
        if !self.children.is_empty() {
            if let Some(it) = self.children.get_mut(0) {
                it.is_highlight = true;
            }
            Some(())
        } else {
            None
        }
    }

    /// highlight prev item in this node
    fn highlight_prev(&mut self) {
        // if no child selected, then
        let Some(current_index) = self.highlight_child_index() else {
            self.highlight_first_child();
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

    /// highlight prev item in this node
    fn highlight_next(&mut self) {
        // if no child selected, then
        let Some(current_index) = self.highlight_child_index() else {
            self.highlight_first_child();
            return;
        };

        let index_to_highlight = (current_index + 1).min(self.children.len() - 1);
        self.children[current_index].clear_highlight();
        self.children[index_to_highlight].is_highlight = true;
    }

    /// return highlighted child index
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
    fn clear_highlight(&mut self) {
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
    fn highlight_mut(&mut self) -> Option<&mut Self> {
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
    fn highlight_last_but_one(&mut self) -> Option<&mut Self> {
        // if self is not highlighted or there is no highlighted child, return None
        if !self.is_highlight || self.highlight_child_mut().is_none() {
            return None;
        }

        let mut last_but_one = self;
        while last_but_one
            .highlight_child_mut()
            .and_then(|x| x.highlight_child_mut())
            .is_some()
        {
            last_but_one = last_but_one.highlight_child_mut().unwrap();
        }
        Some(last_but_one)
    }
}

/// Widget focus on display/render
pub struct Menu<T> {
    /// style for default item style
    default_item_style: Style,
    /// style for highlighted item
    highlight_item_style: Style,
    /// width for drop down panel
    drop_down_width: u16,
    /// style for drop down panel
    drop_down_style: Style,
    _priv: PhantomData<T>,
}

impl<T> Menu<T> {
    pub fn new() -> Self {
        Self {
            highlight_item_style: Style::default().fg(Color::White).bg(Color::LightBlue),
            default_item_style: Style::default().fg(Color::White),
            drop_down_width: 20,
            drop_down_style: Style::default().bg(Color::DarkGray),
            _priv: Default::default(),
        }
    }

    /// update with highlight style
    pub fn default_style(mut self, style: Style) -> Self {
        self.default_item_style = style;
        self
    }

    /// update with highlight style
    pub fn highlight(mut self, style: Style) -> Self {
        self.highlight_item_style = style;
        self
    }

    /// update drop_down_width
    pub fn dropdown_width(mut self, width: u16) -> Self {
        self.drop_down_width = width;
        self
    }

    /// update drop_down fill style
    pub fn dropdown_style(mut self, style: Style) -> Self {
        self.drop_down_style = style;
        self
    }

    /// render an item group in drop down
    /* Each menu item is rendered like this
    .|.NameString.|.
      ^^^^^^^^^^^^ ------ this area will be highlighted
    */
    fn render_dropdown(
        &self,
        x: u16,
        y: u16,
        group: &[MenuItem<T>],
        buf: &mut Buffer,
        dropdown_count_to_go: u16, // including current, it is not drawn yet
    ) {
        // Compute width of all menu items
        let child_max_width = group
            .iter()
            .map(|menu_item| Span::from(menu_item.name.clone()).width())
            .max()
            .unwrap_or(0) as u16;

        // Compute minimum size needed after border is added
        // Border is 3 chars wide and 1 char high, on both sides.
        let min_drop_down_width: u16 = child_max_width + 3 + 3;
        let min_drop_down_height: u16 = (group.len() as u16) + 1 + 1;

        // prevent calculation issue if canvas is narrow
        let drop_down_width = self.drop_down_width.min(buf.area.width);

        // calculate the maximum x, leaving enough space for deeper items
        // drawing area:
        // |  a |  b   |            c                |        d       |
        // | .. |  me  |  child_1  |  child_of_child |  nothing here  |
        // x_max is the x when d is 0
        let b_plus_c = dropdown_count_to_go * drop_down_width;
        let x_max = buf.area().right().saturating_sub(b_plus_c);

        let x = x.min(x_max);

        let area = Rect::new(x, y, min_drop_down_width, min_drop_down_height);

        // clamp to ensure we draw in areas
        let area = area.clamp(*buf.area());

        Clear.render(area, buf);

        buf.set_style(area, self.default_item_style);

        // Render menu border
        let border = Block::default()
            .borders(Borders::ALL)
            .style(self.default_item_style);
        border.render(
            area.inner(Margin {
                vertical: 0,
                horizontal: 1,
            }),
            buf,
        );

        // Render menu items
        let mut active_group: Option<_> = None;
        for (idx, item) in group.iter().enumerate() {
            let item_x = x + 2;
            let item_y = y + 1 + idx as u16;
            let is_active = item.is_highlight;

            let item_name = item.name();

            // make style apply to whole line by make name whole line
            let mut item_name =
                format!(" {: <width$} ", item_name, width = child_max_width as usize);

            if !item.children.is_empty() {
                item_name.pop();
                item_name.push('>');
            }

            buf.set_span(
                item_x,
                item_y,
                &Span::styled(
                    item_name,
                    if is_active {
                        self.highlight_item_style
                    } else {
                        self.default_item_style
                    },
                ),
                child_max_width + 2,
            );

            if is_active && !item.children.is_empty() {
                active_group = Some((item_x + child_max_width, item_y, item));
            }
        }

        // draw at the end to ensure its content above all items in current level
        if let Some((x, y, item)) = active_group {
            self.render_dropdown(x, y, &item.children, buf, dropdown_count_to_go - 1);
        }
    }
}

impl<T> Default for Menu<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> StatefulWidget for Menu<T> {
    type State = MenuState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = area.clamp(*buf.area());

        let mut spans = vec![];
        let mut x_pos = area.x;
        let y_pos = area.y;

        let dropdown_count = state.dropdown_count();

        // Skip top left char
        spans.push(Span::raw(" ").style(self.default_item_style));

        for item in state.root_item.children.iter() {
            let is_highlight = item.is_highlight;
            let item_style = if is_highlight {
                self.highlight_item_style
            } else {
                self.default_item_style
            };
            let has_children = !item.children.is_empty();

            let group_x_pos = x_pos;
            let span = Span::styled(format!(" {} ", item.name()), item_style);
            x_pos += span.width() as u16;
            spans.push(span);

            if has_children && is_highlight {
                self.render_dropdown(group_x_pos, y_pos + 1, &item.children, buf, dropdown_count);
            }
        }
        buf.set_line(area.x, area.y, &Line::from(spans), area.width);
    }
}

#[cfg(test)]
mod tests {
    use crate::MenuState;

    type MenuItem = super::MenuItem<i32>;

    #[test]
    fn test_active_depth() {
        {
            let menu_state = MenuState::new(vec![MenuItem::item("item1", 0)]);
            assert_eq!(menu_state.active_depth(), 0);
        }

        {
            let menu_state = MenuState::new(vec![MenuItem::item("item1", 0).with_highlight(true)]);
            assert_eq!(menu_state.active_depth(), 1);
        }

        {
            let menu_state = MenuState::new(vec![MenuItem::group("layer1", vec![])]);
            assert_eq!(menu_state.active_depth(), 0);
        }

        {
            let menu_state =
                MenuState::new(vec![MenuItem::group("layer1", vec![]).with_highlight(true)]);
            assert_eq!(menu_state.active_depth(), 1);
        }

        {
            let menu_state = MenuState::new(vec![MenuItem::group(
                "layer_1",
                vec![MenuItem::item("item_layer_2", 0)],
            )
            .with_highlight(true)]);
            assert_eq!(menu_state.active_depth(), 1);
        }

        {
            let menu_state = MenuState::new(vec![MenuItem::group(
                "layer_1",
                vec![MenuItem::item("item_layer_2", 0).with_highlight(true)],
            )
            .with_highlight(true)]);
            assert_eq!(menu_state.active_depth(), 2);
        }
    }

    #[test]
    fn test_dropdown_count() {
        {
            // only item in menu bar
            let menu_state = MenuState::new(vec![MenuItem::item("item1", 0)]);
            assert_eq!(menu_state.dropdown_count(), 0);
        }

        {
            // group in menu bar,
            let menu_state = MenuState::new(vec![MenuItem::group(
                "menu bar",
                vec![MenuItem::item("item layer 1", 0)],
            )
            .with_highlight(true)]);
            assert_eq!(menu_state.dropdown_count(), 1);
        }

        {
            // group in menu bar,
            let menu_state = MenuState::new(vec![MenuItem::group(
                "menu bar 1",
                vec![
                    MenuItem::group("dropdown 1", vec![MenuItem::item("item layer 2", 0)])
                        .with_highlight(true),
                    MenuItem::item("item layer 1", 0),
                ],
            )
            .with_highlight(true)]);
            assert_eq!(menu_state.dropdown_count(), 2);
        }

        {
            // *menu bar 1
            // *dropdown 1   >  item layer 2
            // item layer 1    group layer 2 >
            let menu_state = MenuState::new(vec![MenuItem::group(
                "menu bar 1",
                vec![
                    MenuItem::group(
                        "dropdown 1",
                        vec![
                            MenuItem::item("item layer 2", 0),
                            MenuItem::group(
                                "group layer 2",
                                vec![MenuItem::item("item layer 3", 0)],
                            ),
                        ],
                    )
                    .with_highlight(true),
                    MenuItem::item("item layer 1", 0),
                ],
            )
            .with_highlight(true)]);
            assert_eq!(menu_state.dropdown_count(), 2);
        }

        {
            // *menu bar 1
            // *dropdown 1   >  *item layer 2
            // item layer 1    group layer 2 > item layer 3
            let menu_state = MenuState::new(vec![MenuItem::group(
                "menu bar 1",
                vec![
                    MenuItem::group(
                        "dropdown 1",
                        vec![
                            MenuItem::item("item layer 2", 0).with_highlight(true),
                            MenuItem::group(
                                "group layer 2",
                                vec![MenuItem::item("item layer 3", 0)],
                            ),
                        ],
                    )
                    .with_highlight(true),
                    MenuItem::item("item layer 1", 0),
                ],
            )
            .with_highlight(true)]);
            assert_eq!(menu_state.dropdown_count(), 3);
        }
    }
}
