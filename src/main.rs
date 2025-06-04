mod convert;
mod doom;
mod fists;
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
                        app.subcontext = Some(Box::new(ConvertPrompt::default()))
                    }
                    _ => (),
                }
            }
        }
    }

    fn draw(&mut self, theme: &Theme, items: &Vec<SkinItem>, frame: &mut Frame) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(3)]);
        let areas = vertical.split(frame.area());

        self.draw_table(theme, items, frame, areas[0]);
        self.draw_scrollbar(theme, frame, areas[0]);
        self.draw_footer(theme, frame, areas[1]);
    }

    fn draw_table(&mut self, theme: &Theme, items: &Vec<SkinItem>, frame: &mut Frame, area: Rect) {
        let header = ["Name", "Path", "Sprite", "Mugshot"]
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
        let rows = items.iter().map(|data| {
            data.ref_array()
                .into_iter()
                .map(|content| {
                    Cell::from(Text::from(content.clone()))
                        .style(Style::default().fg(theme.fg).bg(theme.bg))
                })
                .collect::<Row>()
                .height(1)
        });
        let table = Table::new(
            rows,
            [
                Constraint::Min(10),
                Constraint::Min(10),
                Constraint::Length(8),
                Constraint::Length(8),
            ],
        )
        .header(header)
        .row_highlight_style(Style::default().fg(theme.selected_fg).bg(theme.selected_bg))
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
            "(A) Add (E) Edit (D) Delete (J) Next (K) Prev (Enter) Convert (Q) Quit",
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
                    KeyCode::Char('y') | KeyCode::Esc => app.quit = true,
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
    mugshot: TextState<'static>,
    item_field: ItemField,
    name_error: String,
    path_error: String,
    sprite_error: String,
    mugshot_error: String,
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
                    if self.name.status().is_done()
                        && self.path.status().is_done()
                        && self.sprite.status().is_done()
                        && self.mugshot.status().is_done()
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
            .split(popup_area(frame.area(), 70, 70));
        frame.render_widget(Clear, areas[0]);
        frame.render_widget(popup, areas[0]);
        frame.render_widget(Clear, areas[1]);
        frame.render_widget(
            Line::from("(Tab) Next (Enter) Submit").right_aligned(),
            areas[1],
        );

        let areas = Layout::vertical(vec![Constraint::Length(1); 8])
            .margin(2)
            .split(areas[0]);
        TextPrompt::from("Name").draw(frame, areas[0], &mut self.name);
        TextPrompt::from("Path").draw(frame, areas[2], &mut self.path);
        TextPrompt::from("Sprite").draw(frame, areas[4], &mut self.sprite);
        TextPrompt::from("Mugshot").draw(frame, areas[6], &mut self.mugshot);

        frame.render_widget(Line::from(self.name_error.clone()).red(), areas[1]);
        frame.render_widget(Line::from(self.path_error.clone()).red(), areas[3]);
        frame.render_widget(Line::from(self.sprite_error.clone()).red(), areas[5]);
        frame.render_widget(Line::from(self.mugshot_error.clone()).red(), areas[7]);
    }
}

impl ItemPrompt {
    fn add() -> Self {
        Self {
            name: TextState::default().with_focus(FocusState::Focused),
            ..Default::default()
        }
    }

    fn edit(item: &SkinItem, index: usize) -> Self {
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
            mugshot: TextState::default()
                .with_value(item.mugshot.clone())
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
            ItemField::Mugshot => &mut self.mugshot,
        }
    }

    fn advance_field(&mut self) {
        self.field().blur();
        self.item_field = match self.item_field {
            ItemField::Name => ItemField::Path,
            ItemField::Path => ItemField::Sprite,
            ItemField::Sprite => ItemField::Mugshot,
            ItemField::Mugshot => ItemField::Name,
        };
        self.field().focus();
    }

    fn retreat_field(&mut self) {
        self.field().blur();
        self.item_field = match self.item_field {
            ItemField::Name => ItemField::Mugshot,
            ItemField::Path => ItemField::Name,
            ItemField::Sprite => ItemField::Path,
            ItemField::Mugshot => ItemField::Sprite,
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
                } else if !std::path::Path::new(path).exists() {
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
            ItemField::Mugshot => {
                *self.mugshot.status_mut() = Status::Aborted;
                let mugshot = self.mugshot.value();
                if mugshot.len() != 3 {
                    self.mugshot_error = "Must be 3 character long!".into();
                } else if !validate_sprite(mugshot) {
                    self.mugshot_error =
                        "Must only contain alphabetic characters or ('[', ']', '\\')".into();
                } else {
                    *self.mugshot.status_mut() = Status::Done;
                    self.mugshot_error.clear();
                }
            }
        }
    }

    fn submit_item_prompt(&mut self, app: &mut App) {
        let item = SkinItem {
            name: self.name.value().into(),
            path: self.path.value().into(),
            sprite: self.sprite.value().to_uppercase(),
            mugshot: self.mugshot.value().to_uppercase(),
        };
        if let Some(index) = self.edit {
            let _ = std::mem::replace(&mut app.items[index], item);
        } else {
            app.items.push(item);
        }
    }
}

struct ConvertPrompt {
    file_name: TextState<'static>,
    error: String,
}

impl Default for ConvertPrompt {
    fn default() -> Self {
        Self {
            file_name: TextState::default().with_focus(FocusState::Focused),
            error: String::new(),
        }
    }
}

impl Context for ConvertPrompt {
    fn handle_event(mut self: Box<Self>, app: &mut App, event: Event) -> Option<Box<dyn Context>> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => return None,
                KeyCode::Enter => {
                    if self.file_name.status().is_done() {
                        if let Some(acc) = crate::fists::get_acc() {
                            return Some(Box::new(FistsConfirm::new(
                                self.file_name.value().into(),
                                acc,
                            )));
                        } else {
                            let _gag = gag::Gag::stdout().unwrap();
                            crate::convert::convert_all(
                                &app.items,
                                self.file_name.value().into(),
                                None,
                            )
                            .unwrap();
                            return Some(Box::new(Success::new(self.file_name.value().into())));
                        }
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
            .split(popup_area(frame.area(), 70, 70));
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

struct FistsConfirm {
    file_name: String,
    acc: std::path::PathBuf,
}

impl Context for FistsConfirm {
    fn handle_event(self: Box<Self>, app: &mut App, event: Event) -> Option<Box<dyn Context>> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => return None,
                KeyCode::Char('y') => {
                    let _gag = gag::Gag::stdout().unwrap();
                    crate::convert::convert_all(&app.items, self.file_name.clone(), Some(self.acc))
                        .unwrap();
                    return Some(Box::new(Success::new_with_fists(self.file_name.clone())));
                }
                KeyCode::Char('n') => {
                    let _gag = gag::Gag::stdout().unwrap();
                    crate::convert::convert_all(&app.items, self.file_name.clone(), None).unwrap();
                    return Some(Box::new(Success::new(self.file_name)));
                }
                _ => (),
            }
        }
        Some(self)
    }

    fn draw(&mut self, theme: &Theme, frame: &mut Frame) {
        let popup = Paragraph::new("ACC detected: Do you want to generate fists?")
            .centered()
            .block(Block::bordered());
        let areas = Layout::vertical([Constraint::Min(1), Constraint::Length(1)])
            .split(popup_area(frame.area(), 70, 70));
        frame.render_widget(Clear, areas[0]);
        frame.render_widget(Clear, areas[1]);
        frame.render_widget(popup, areas[0]);
        frame.render_widget(Line::from("(y) Yes (n) No").right_aligned(), areas[1]);
    }
}

impl FistsConfirm {
    fn new(file_name: String, acc: std::path::PathBuf) -> Self {
        Self { file_name, acc }
    }
}

impl ConvertPrompt {
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

struct Success {
    file_name: String,
    fists_file_name: Option<String>,
}

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
            .split(popup_area(frame.area(), 70, 70));

        frame.render_widget(Clear, areas[0]);
        frame.render_widget(popup, areas[0]);
        frame.render_widget(Clear, areas[1]);
        frame.render_widget(Line::from("(any) Quit").right_aligned(), areas[1]);

        let area = Rect::new(
            areas[0].x,
            areas[0].y + areas[0].height / 2,
            areas[0].width,
            1,
        );
        frame.render_widget(
            Line::from(format!("'{}' created successfully!", self.file_name)).centered(),
            area,
        );
        if let Some(fists_file_name) = self.fists_file_name.clone() {
            let area = Rect::new(area.x, area.y + 1, area.width, 1);
            frame.render_widget(
                Line::from(format!("'{}' created successfully!", fists_file_name)).centered(),
                area,
            );
        }
    }
}

impl Success {
    fn new(file_name: String) -> Self {
        Self {
            file_name,
            fists_file_name: None,
        }
    }

    fn new_with_fists(file_name: String) -> Self {
        Self {
            file_name: file_name.clone(),
            fists_file_name: Some(file_name.replace('.', "_fists.")),
        }
    }
}

#[derive(Default)]
struct App {
    quit: bool,
    items: Vec<SkinItem>,
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
    Mugshot,
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
