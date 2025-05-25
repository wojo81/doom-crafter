mod convert;
mod doom;
mod minecraft;

use crate::convert::*;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarState, Table, TableState},
};

use ratatui::style::{Color, Modifier, Style, palette::tailwind};

use tui_prompts::prelude::*;

struct Theme {
    bg: Color,
    fg: Color,
    accent: Color,
    header_bg: Color,
    header_fg: Color,
    selected_bg: Color,
    selected_fg: Color,
    border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: tailwind::SLATE.c800,
            fg: tailwind::SLATE.c300,
            accent: tailwind::CYAN.c400,
            header_bg: tailwind::SLATE.c700,
            header_fg: tailwind::CYAN.c500,
            selected_bg: tailwind::CYAN.c500,
            selected_fg: Color::White,
            border: tailwind::CYAN.c300,
        }
    }
}

trait Context {
    fn handle_event(self: Box<Self>, app: &mut App, event: Event) -> Option<Box<dyn Context>>;
    fn draw(&mut self, theme: &Theme, frame: &mut Frame);
}

#[derive(Default)]
struct MainContext {
    table: TableState,
    scroll: ScrollbarState,
}

impl MainContext {
    fn handle_event(&mut self, app: &mut App, event: Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.subcontext = Some(Box::new(QuitConfirm));
                    }
                    KeyCode::Char('j') | KeyCode::Down => self.table.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.table.select_previous(),
                    KeyCode::Char('a') => app.subcontext = Some(Box::new(ItemPrompt::add())),
                    KeyCode::Char('e') => {
                        if let Some(i) = self.table.selected() {
                            app.subcontext = Some(Box::new(ItemPrompt::edit(&app.items[i], i)));
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(i) = self.table.selected() {
                            app.items.remove(i);
                        }
                    }
                    KeyCode::Enter if !app.items.is_empty() => {
                        app.subcontext = Some(Box::new(SubmitPrompt::default()))
                    }
                    _ => (),
                }
            }
        }
    }

    fn draw(&mut self, theme: &Theme, items: &Vec<SkinInfo>, frame: &mut Frame) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(3)]);
        let areas = vertical.split(frame.area());

        self.draw_table(theme, items, frame, areas[0]);
        self.draw_scrollbar(theme, frame, areas[0]);
        self.draw_footer(theme, frame, areas[1]);
    }

    fn draw_table(&mut self, theme: &Theme, items: &Vec<SkinInfo>, frame: &mut Frame, area: Rect) {
        let header = ["Name", "Path", "Sprite"]
            .into_iter()
            .map(|h| {
                Cell::from(h).style(
                    Style::default()
                        .fg(theme.header_fg)
                        .bg(theme.header_bg)
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect::<Row>()
            .height(1);
        let rows = items.iter().enumerate().map(|(i, data)| {
            let mut row = data
                .ref_array()
                .into_iter()
                .map(|content| {
                    Cell::from(Text::from(content.clone()))
                        .style(Style::default().fg(theme.fg).bg(theme.bg))
                })
                .collect::<Row>()
                .height(1);

            if Some(i) == self.table.selected() {
                row = row.style(
                    Style::default()
                        .fg(theme.selected_fg)
                        .bg(theme.selected_bg)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        });
        let table = Table::new(
            rows,
            [
                Constraint::Min(10),
                Constraint::Min(10),
                Constraint::Length(10),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg)),
        );

        frame.render_stateful_widget(table, area, &mut self.table);
    }

    fn draw_scrollbar(&mut self, theme: &Theme, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .thumb_style(Style::default().bg(theme.accent)),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll,
        );
    }

    fn draw_footer(&mut self, theme: &Theme, frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Text::from(
            "(A) Add (E) Edit (Delete) (J) Next (K) Prev (Q) Quit",
        ))
        .centered()
        .style(Style::default().fg(theme.accent).bg(theme.header_bg))
        .block(
            Block::bordered()
                .border_type(ratatui::widgets::BorderType::Double)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.header_bg)),
        );
        frame.render_widget(info_footer, area);
    }
}

struct QuitConfirm;

impl Context for QuitConfirm {
    fn handle_event(self: Box<Self>, app: &mut App, event: Event) -> Option<Box<dyn Context>> {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Esc => {
                        app.quit = true;
                        return None;
                    }
                    KeyCode::Char('n') | KeyCode::Down => return None,
                    _ => return Some(self),
                }
            }
        }
        Some(self)
    }

    fn draw(&mut self, theme: &Theme, frame: &mut Frame) {
        let popup = Paragraph::new("Are you ready to quit?")
            .centered()
            .block(Block::bordered());
        let areas = Layout::vertical([Constraint::Min(1), Constraint::Length(1)])
            .split(popup_area(frame.area(), 30, 30));
        frame.render_widget(Clear, areas[0]);
        frame.render_widget(Clear, areas[1]);
        frame.render_widget(popup, areas[0]);
        frame.render_widget(Line::from("(Y) Yes (N) No").right_aligned(), areas[1]);
    }
}

#[derive(Default)]
struct ItemPrompt {
    name: TextState<'static>,
    path: TextState<'static>,
    sprite: TextState<'static>,
    item_field: ItemField,
    name_error: String,
    path_error: String,
    sprite_error: String,
    edit: Option<usize>,
}

impl Context for ItemPrompt {
    fn handle_event(mut self: Box<Self>, app: &mut App, event: Event) -> Option<Box<dyn Context>> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Tab => self.advance_field(),
                KeyCode::BackTab => self.retreat_field(),
                KeyCode::Esc => return None,
                KeyCode::Enter => {
                    if self.name.status() == Status::Done
                        && self.path.status() == Status::Done
                        && self.sprite.status() == Status::Done
                    {
                        self.submit_item_prompt(app);
                        return None;
                    }
                }
                _ => {
                    self.field().handle_key_event(key);
                    self.validate_field();
                }
            }
        }
        Some(self)
    }

    fn draw(&mut self, theme: &Theme, frame: &mut Frame) {
        let popup = Block::bordered();
        let areas = Layout::vertical([Constraint::Min(1), Constraint::Length(1)])
            .split(popup_area(frame.area(), 50, 50));
        frame.render_widget(Clear, areas[0]);
        frame.render_widget(popup, areas[0]);
        frame.render_widget(Clear, areas[1]);
        frame.render_widget(
            Line::from("(Tab) Next (Enter) Submit").right_aligned(),
            areas[1],
        );

        let areas = Layout::vertical(vec![Constraint::Length(1); 6])
            .margin(2)
            .split(areas[0]);
        TextPrompt::from("Name").draw(frame, areas[0], &mut self.name);
        TextPrompt::from("Path").draw(frame, areas[2], &mut self.path);
        TextPrompt::from("Sprite").draw(frame, areas[4], &mut self.sprite);

        frame.render_widget(Line::from(self.name_error.clone()).red(), areas[1]);
        frame.render_widget(Line::from(self.path_error.clone()).red(), areas[3]);
        frame.render_widget(Line::from(self.sprite_error.clone()).red(), areas[5]);
    }
}

impl ItemPrompt {
    fn add() -> Self {
        Self {
            name: TextState::default().with_focus(FocusState::Focused),
            ..Default::default()
        }
    }

    fn edit(item: &SkinInfo, index: usize) -> Self {
        Self {
            name: TextState::default()
                .with_value(item.name.clone())
                .with_focus(FocusState::Focused)
                .with_status(Status::Done),
            path: TextState::default()
                .with_value(item.path.clone())
                .with_status(Status::Done),
            sprite: TextState::default()
                .with_value(item.sprite.clone())
                .with_status(Status::Done),
            edit: Some(index),
            ..Default::default()
        }
    }

    fn field(&mut self) -> &mut TextState<'static> {
        match self.item_field {
            ItemField::Name => &mut self.name,
            ItemField::Path => &mut self.path,
            ItemField::Sprite => &mut self.sprite,
        }
    }

    fn advance_field(&mut self) {
        self.field().blur();
        self.item_field = match self.item_field {
            ItemField::Name => ItemField::Path,
            ItemField::Path => ItemField::Sprite,
            ItemField::Sprite => ItemField::Name,
        };
        self.field().focus();
    }

    fn retreat_field(&mut self) {
        self.field().blur();
        self.item_field = match self.item_field {
            ItemField::Name => ItemField::Sprite,
            ItemField::Path => ItemField::Name,
            ItemField::Sprite => ItemField::Path,
        };
        self.field().focus();
    }

    fn validate_field(&mut self) {
        match self.item_field {
            ItemField::Name => {
                *self.name.status_mut() = Status::Aborted;
                if self.name.value().is_empty() {
                    self.name_error = "Cannot be empty!".into();
                } else {
                    *self.name.status_mut() = Status::Done;
                    self.name_error.clear();
                }
            }
            ItemField::Path => {
                *self.path.status_mut() = Status::Aborted;
                let path = self.path.value();
                if !path.ends_with(".png") {
                    self.path_error = "Must be a png file!".into();
                } else if !AsRef::<std::path::Path>::as_ref(path).exists() {
                    self.path_error = "Does not exist!".into();
                } else {
                    *self.path.status_mut() = Status::Done;
                    self.path_error.clear();
                }
            }
            ItemField::Sprite => {
                *self.sprite.status_mut() = Status::Aborted;
                let sprite = self.sprite.value();
                if sprite.len() != 4 {
                    self.sprite_error = "Must be 4 characters long!".into();
                } else if !validate_sprite(sprite) {
                    self.sprite_error =
                        "Must only contain alphabetic characters or ('[', ']', '\\')".into();
                } else {
                    *self.sprite.status_mut() = Status::Done;
                    self.sprite_error.clear();
                }
            }
        }
    }

    fn submit_item_prompt(&mut self, app: &mut App) {
        let item = SkinInfo {
            name: self.name.value().into(),
            path: self.path.value().into(),
            sprite: self.sprite.value().into(),
        };
        if let Some(index) = self.edit {
            let _ = std::mem::replace(&mut app.items[index], item);
        } else {
            app.items.push(item);
        }
    }
}

#[derive(Default)]
struct SubmitPrompt {
    file_name: TextState<'static>,
    error: String,
}

impl Context for SubmitPrompt {
    fn handle_event(mut self: Box<Self>, app: &mut App, event: Event) -> Option<Box<dyn Context>> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => return None,
                KeyCode::Enter => {
                    if self.file_name.status().is_done() {
                        let _gag = gag::Gag::stdout().unwrap();
                        convert_all(&app.items, self.file_name.value().into()).unwrap();
                        return Some(Box::new(Success));
                    }
                }
                _ => {
                    self.file_name.handle_key_event(key);
                    self.validate();
                }
            }
        }
        Some(self)
    }

    fn draw(&mut self, theme: &Theme, frame: &mut Frame) {
        let popup = Block::bordered();
        let areas = Layout::vertical([Constraint::Min(1), Constraint::Length(1)])
            .split(popup_area(frame.area(), 30, 30));
        frame.render_widget(Clear, areas[0]);
        frame.render_widget(popup, areas[0]);
        frame.render_widget(Clear, areas[1]);
        frame.render_widget(Line::from("(Enter) Submit").right_aligned(), areas[1]);

        let areas = Layout::vertical(vec![Constraint::Length(1); 2])
            .margin(2)
            .split(areas[0]);
        TextPrompt::from("File name").draw(frame, areas[0], &mut self.file_name);
        frame.render_widget(Line::from(self.error.clone()), areas[1]);
    }
}

impl SubmitPrompt {
    fn validate(&mut self) {
        *self.file_name.status_mut() = Status::Aborted;
        if !self.file_name.value().ends_with(".wad") {
            self.error = "Must be a wad file!".into();
        } else {
            *self.file_name.status_mut() = Status::Done;
            self.error.clear();
        }
    }
}

struct Success;

impl Context for Success {
    fn handle_event(self: Box<Self>, app: &mut App, event: Event) -> Option<Box<dyn Context>> {
        if let Event::Key(_) = event {
            app.quit = true;
        }
        Some(self)
    }

    fn draw(&mut self, theme: &Theme, frame: &mut Frame) {
        let popup = Block::bordered();
        let areas = Layout::vertical([Constraint::Min(1), Constraint::Length(1)])
            .split(popup_area(frame.area(), 30, 30));

        frame.render_widget(Clear, areas[0]);
        frame.render_widget(popup, areas[0]);
        frame.render_widget(Clear, areas[1]);
        frame.render_widget(Line::from("(Any) Quit").right_aligned(), areas[1]);

        let areas = Layout::vertical([Constraint::Length(1)])
            .margin(2)
            .split(areas[0]);
        frame.render_widget(
            Line::from("File created successfully!").centered(),
            areas[0],
        );
    }
}

#[derive(Default)]
struct App {
    quit: bool,
    items: Vec<SkinInfo>,
    subcontext: Option<Box<dyn Context>>,
    theme: Theme,
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let mut context = MainContext::default();

        while !self.quit {
            if let Some(mut subcontext) = self.subcontext.take() {
                terminal.draw(|frame| {
                    context.draw(&self.theme, &self.items, frame);
                    subcontext.draw(&self.theme, frame);
                })?;
                self.subcontext = subcontext.handle_event(&mut self, event::read()?);
            } else {
                terminal.draw(|frame| context.draw(&self.theme, &self.items, frame))?;
                context.handle_event(&mut self, event::read()?);
            }
        }
        Ok(())
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

#[derive(Default)]
enum ItemField {
    #[default]
    Name,
    Path,
    Sprite,
}

fn main() {
    color_eyre::install().unwrap();
    let terminal = ratatui::init();
    let result = App::default().run(terminal);
    ratatui::restore();
}

fn validate_sprite(sprite: &str) -> bool {
    for c in sprite.chars() {
        if !c.is_ascii() || !c.is_alphabetic() && c != '[' && c != ']' && c != '\\' {
            return false;
        }
    }
    true
}
