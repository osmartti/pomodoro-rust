use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Text, ToSpan};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph};
use ratatui::{DefaultTerminal, Frame};

#[derive(PartialEq, Eq)]
enum Mode {
    OUTER,
    INNER,
}

struct Theme {
    name: &'static str,
    main_color: Color,
    accent_color: Color,
}

const THEMES: [Theme; 5] = [
    Theme {
        name: "BASIC",
        main_color: Color::White,
        accent_color: Color::White,
    },
    Theme {
        name: "MIAMI",
        main_color: Color::Magenta,
        accent_color: Color::Yellow,
    },
    Theme {
        name: "MATRIX",
        main_color: Color::Green,
        accent_color: Color::LightGreen,
    },
    Theme {
        name: "SEABREEZE",
        main_color: Color::LightBlue,
        accent_color: Color::LightYellow,
    },
    Theme {
        name: "DRACULA",
        main_color: Color::Red,
        accent_color: Color::LightRed,
    },
];

struct App {
    actions: Vec<&'static str>,
    inner_actions: Vec<String>,
    selected: usize,
    inner_selected: usize,
    current_tab: usize,
    theme_idx: u8,
    main_color: Color,
    accent_color: Color,
    mode: Mode,
}

impl App {
    fn new() -> Self {
        Self {
            actions: vec![
                "Home",
                "Start Pomodoro",
                "Daily Stats",
                "Weekly Stats",
                "Lifetime Stats",
                "Settings",
            ],
            inner_actions: vec![],
            selected: 0,
            inner_selected: 0,
            current_tab: 0,
            theme_idx: 0,
            main_color: Color::White,
            accent_color: Color::White,
            mode: Mode::OUTER,
        }
    }

    fn next(&mut self) {
        if self.mode == Mode::OUTER {
            if self.selected + 1 < self.actions.len() {
                self.selected += 1;
            }
        }
        if self.mode == Mode::INNER {
            if self.inner_selected + 1 < self.inner_actions.len() {
                self.inner_selected += 1;
            }
        }
    }

    fn previous(&mut self) {
        if self.mode == Mode::OUTER {
            if self.selected > 0 {
                self.selected -= 1;
            }
        }
        if self.mode == Mode::INNER {
            if self.inner_selected > 0 {
                self.inner_selected -= 1;
            }
        }
    }

    fn set_inner_actions(&mut self, new_actions: Vec<String>) {
        self.inner_actions = new_actions;
    }

    fn handle_enter(&mut self) {
        if self.mode == Mode::OUTER {
            if self.selected == 0 {
                self.current_tab = self.selected;
                return;
            }
            self.current_tab = self.selected;
            self.mode = Mode::INNER;
        } else {
            match self.inner_selected {
                2 => self.change_theme(),
                _ => {}
            }
        }
    }

    fn change_theme(&mut self) {
        let mut theme: u8 = self.theme_idx;
        let max_theme_idx: u8 = THEMES.len() as u8;
        theme = theme + 1;
        if theme > max_theme_idx - 1 {
            theme = 0
        }
        self.theme_idx = theme;
        self.apply_theme(
            THEMES[theme as usize].main_color,
            THEMES[theme as usize].accent_color,
        );
    }

    fn apply_theme(&mut self, mcolor: Color, acolor: Color) {
        self.main_color = mcolor;
        self.accent_color = acolor;
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut app = App::new();
    loop {
        terminal.draw(|f| render(f, &mut app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                event::KeyCode::Char('q') => {
                    if app.mode == Mode::INNER {
                        app.mode = Mode::OUTER;
                        app.inner_selected = 0;
                        ();
                    } else {
                        break Ok(());
                    }
                }
                event::KeyCode::Up => app.previous(),
                event::KeyCode::Down => app.next(),
                event::KeyCode::Enter => app.handle_enter(),
                _ => {}
            }
        }
    }
}

fn render(frame: &mut Frame, app: &mut App) {
    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(vec![Constraint::Percentage(100)])
        .split(frame.area());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(outer_layout[0]);

    frame.render_widget(
        Block::bordered()
            .fg(app.main_color)
            .title_bottom(" © osmartti 2026 ".to_span().into_centered_line())
            .title(" POMODORO ".to_span().into_centered_line()),
        outer_layout[0],
    );

    let items: Vec<ListItem> = app
        .actions
        .iter()
        .enumerate()
        .map(|(i, a)| {
            if i == app.current_tab {
                ListItem::new(*a).style(Style::default().fg(app.accent_color))
            } else {
                ListItem::new(*a)
            }
        })
        .collect();

    let is_active = app.mode == Mode::OUTER;
    let list = create_list(items, is_active, app, true);
    let mut list_state = if app.mode == Mode::OUTER {
        ListState::default().with_selected(Some(app.selected))
    } else {
        ListState::default()
    };

    frame.render_stateful_widget(list, inner_layout[0], &mut list_state);

    let inner_layout_block_color: Color = if app.mode == Mode::OUTER {
        app.main_color
    } else {
        app.accent_color
    };
    let inner_layout_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(inner_layout_block_color))
        .padding(Padding::new(1, 1, 0, 0));
    let inner_layout_block_area = inner_layout_block.inner(inner_layout[1]);
    frame.render_widget(inner_layout_block, inner_layout[1]);

    match app.current_tab {
        0 => render_home_screen(frame, inner_layout_block_area, app),
        1 => render_start_pomodoro(frame, inner_layout_block_area, app),
        2 => render_daily_stats(frame, inner_layout_block_area),
        3 => render_weekly_stats(frame, inner_layout_block_area),
        4 => render_lifetime_stats(frame, inner_layout_block_area),
        5 => render_settings(frame, inner_layout_block_area, app),
        _ => {}
    }
}

// HELPER FUNCTIONS

fn create_list<'a>(
    list_items: Vec<ListItem<'a>>,
    active: bool,
    app: &mut App,
    has_borders: bool,
) -> List<'a> {
    let accent_color: Color = app.accent_color;
    let main_color: Color = app.main_color;
    let borders = if has_borders {
        Borders::ALL
    } else {
        Borders::NONE
    };
    let color: Color = if active { accent_color } else { main_color };
    return List::new(list_items)
        .block(
            Block::new()
                .border_style(Style::default().fg(color))
                .borders(borders),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
}

fn ascii_art(color: Color) -> Paragraph<'static> {
    let pomodoro_ascii = r#"
██████╗  ██████╗ ███╗   ███╗ ██████╗ ██████╗  ██████╗ ██████╗  ██████╗ 
██╔══██╗██╔═══██╗████╗ ████║██╔═══██╗██╔══██╗██╔═══██╗██╔══██╗██╔═══██╗
██████╔╝██║   ██║██╔████╔██║██║   ██║██║  ██║██║   ██║██████╔╝██║   ██║
██╔═══╝ ██║   ██║██║╚██╔╝██║██║   ██║██║  ██║██║   ██║██╔══██╗██║   ██║
██║     ╚██████╔╝██║ ╚═╝ ██║╚██████╔╝██████╔╝╚██████╔╝██║  ██║╚██████╔╝
╚═╝      ╚═════╝ ╚═╝     ╚═╝ ╚═════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝
"#;
    let mut text = Text::default().fg(color);
    for line in pomodoro_ascii.lines() {
        text.lines.push(Line::from(line));
    }
    let paragraph = Paragraph::new(text);
    return paragraph;
}

// RENDER FUNCTIONS

fn render_home_screen(frame: &mut Frame, area: Rect, app: &mut App) {
    let home_screen_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vec![Constraint::Min(7), Constraint::Percentage(75)])
        .split(area);

    let readme_text: &str = r#"
    Welcome to Pomodoro application!

    Navigation:
    UP, DOWN -> navigate menu
    ENTER -> Select highlighted menu item
    q -> exit menu / quit the application

    © osmartti 2026
    "#;
    let mut text = Text::default();
    for line in readme_text.lines() {
        text.lines.push(Line::from(line));
    }
    let paragraph = Paragraph::new(text);

    frame.render_widget(ascii_art(app.accent_color), home_screen_layout[0]);
    frame.render_widget(paragraph, home_screen_layout[1]);
}

fn render_start_pomodoro(frame: &mut Frame, area: Rect, app: &mut App) {
    let pomodoro_timer_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vec![Constraint::Min(7), Constraint::Percentage(75)])
        .split(area);

    frame.render_widget(ascii_art(app.accent_color), pomodoro_timer_layout[0]);
    frame.render_widget(Paragraph::new("Start Pomodoro"), pomodoro_timer_layout[1]);
}

fn render_daily_stats(frame: &mut Frame, area: Rect) {
    frame.render_widget(Paragraph::new("Daily Stats"), area);
}

fn render_weekly_stats(frame: &mut Frame, area: Rect) {
    frame.render_widget(Paragraph::new("Weekly Stats"), area);
}

fn render_lifetime_stats(frame: &mut Frame, area: Rect) {
    frame.render_widget(Paragraph::new("Lifetime Stats"), area);
}

fn render_settings(frame: &mut Frame, area: Rect, app: &mut App) {
    let current_theme: &Theme = &THEMES[app.theme_idx as usize];
    let theme_string: String = "Theme: ".to_owned() + current_theme.name;
    let settings_actions: Vec<String> = vec![
        String::from("Default Work Minutes: 25"),
        String::from("Default Break Minutes: 5"),
        theme_string,
    ];
    app.set_inner_actions(settings_actions);
    let items: Vec<ListItem> = app
        .inner_actions
        .iter()
        .enumerate()
        .map(|(i, a)| {
            if i == app.inner_selected {
                ListItem::new(a.clone()).style(Style::default().fg(app.accent_color))
            } else {
                ListItem::new(a.clone())
            }
        })
        .collect();
    let is_active: bool = app.mode == Mode::INNER;
    let list = create_list(items, is_active, app, false);
    let mut list_state = ListState::default().with_selected(Some(app.inner_selected));
    frame.render_stateful_widget(list, area, &mut list_state);
}
