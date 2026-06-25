use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant};

use chrono::Local;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text, ToSpan};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Padding, Paragraph};
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
        accent_color: Color::Gray,
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
    timer_duration: Duration,
    work_category: String,
    work_time_duration: Duration,
    break_time_duration: Duration,
    is_timer_running: bool,
    should_start_break_timer: bool,
    stats_path: String,
    should_add_entry: bool,
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
            timer_duration: Duration::from_secs(0),
            work_category: "Basic Work".to_owned(),
            work_time_duration: Duration::from_mins(25), //1500
            break_time_duration: Duration::from_mins(5), //300
            is_timer_running: false,
            should_start_break_timer: false,
            stats_path: "/".to_owned(),
            should_add_entry: false,
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

    fn tick_timer(&mut self, elapsed: Duration) {
        if self.is_timer_running {
            if self.timer_duration > elapsed {
                self.timer_duration -= elapsed;
            } else {
                if self.should_start_break_timer {
                    self.timer_duration = self.break_time_duration;
                    self.should_start_break_timer = false;
                    self.should_add_entry = true;
                } else {
                    self.timer_duration = Duration::from_secs(0);
                    self.is_timer_running = false;
                }
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
            match self.selected {
                1 => {
                    if self.is_timer_running == false {
                        self.timer_duration =
                            Duration::from_secs(self.work_time_duration.as_secs());
                        self.is_timer_running = true;
                        self.should_start_break_timer = true;
                    }
                }
                _ => {}
            }
        } else {
            match self.selected {
                5 => match self.inner_selected {
                    4 => {
                        let new_theme_index: u8 = self.theme_idx + 1;
                        self.change_theme(new_theme_index);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn change_theme(&mut self, new_theme_index: u8) {
        let max_theme_idx: u8 = THEMES.len() as u8;
        let mut theme: u8 = new_theme_index;
        if new_theme_index > max_theme_idx - 1 {
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
    create_pomdoro_stats_file()?;
    create_config_file()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut app = App::new();
    initialize_from_settings(&mut app)?;
    let mut last_tick = Instant::now();
    let duration: Duration = Duration::from_millis(100);
    loop {
        terminal.draw(|f| render(f, &mut app))?;
        if event::poll(duration)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        event::KeyCode::Char('q') => {
                            if app.mode == Mode::INNER {
                                app.mode = Mode::OUTER;
                                app.inner_selected = 0;
                                if app.selected == 5 {
                                    save_settings(&mut app)?;
                                }
                                ();
                            } else {
                                break Ok(());
                            }
                        }
                        event::KeyCode::Char('s') => {
                            if app.is_timer_running {
                                app.should_start_break_timer = true;
                                app.is_timer_running = false;
                                app.timer_duration = app.work_time_duration;
                            }
                            // TODO: Render popup when timer is stopped
                        }
                        event::KeyCode::Up => app.previous(),
                        event::KeyCode::Down => app.next(),
                        event::KeyCode::Enter => app.handle_enter(),
                        _ => {}
                    }
                }
            }
        }

        let elapsed = last_tick.elapsed();
        last_tick = Instant::now();
        app.tick_timer(elapsed);
        if app.should_add_entry {
            app.should_add_entry = false;
            add_entry(&mut app)?;
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

fn create_pomdoro_stats_file() -> Result<()> {
    match File::open("pomodoro_stats.csv") {
        Ok(_) => Ok(()),
        Err(_) => {
            let mut new_file = OpenOptions::new()
                .append(true)
                .write(true)
                .read(true)
                .create(true)
                .open("pomodoro_stats.csv")?;
            return Ok(new_file.write_all(b"date;category;work_minutes;break_minutes\n")?);
        }
    }
}

fn create_config_file() -> Result<()> {
    match OpenOptions::new().read(true).open("pomodoro_config") {
        Ok(_) => Ok(()),
        Err(_) => {
            let mut new_config_file = OpenOptions::new()
                .append(true)
                .write(true)
                .read(true)
                .create(true)
                .open("pomodoro_config")?;
            return Ok(new_config_file.write_all(b"THEME=0\nSTATS_PATH=/\nDEFAULT_CATEGORY=Basic Work\nDEFAULT_WORK_MINUTES=25\nDEFAULT_BREAK_MINUTES=5")?);
        }
    }
}

fn save_settings(app: &mut App) -> Result<()> {
    let theme_index = app.theme_idx;
    let working_minutes = app.work_time_duration.as_secs() / 60;
    let break_minutes = app.break_time_duration.as_secs() / 60;
    let stats_path = app.stats_path.clone();
    let work_category = app.work_category.clone();
    let config_content = format!(
        "THEME={}\nSTATS_PATH={}\nDEFAULT_CATEGORY={}\nDEFAULT_WORK_MINUTES={}\nDEFAULT_BREAK_MINUTES={}",
        theme_index, stats_path, work_category, working_minutes, break_minutes
    );
    let mut config_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("pomodoro_config")?;
    return Ok(config_file.write_all(config_content.as_bytes())?);
}

fn initialize_from_settings(app: &mut App) -> Result<()> {
    let config_file_content = std::fs::read_to_string("pomodoro_config")?;
    let lines = config_file_content.lines();
    for line in lines {
        if let Some((key, value)) = line.split_once("=") {
            match key {
                "THEME" => {
                    app.theme_idx = value.parse()?;
                    app.change_theme(app.theme_idx);
                }
                "nSTATS_PATH" => {
                    app.stats_path = value.to_owned();
                }
                "DEFAULT_WORK_MINUTES" => {
                    app.work_time_duration = Duration::from_mins(value.parse()?);
                }
                "DEFAULT_BREAK_MINUTES" => {
                    app.break_time_duration = Duration::from_mins(value.parse()?);
                }
                "DEFAULT_CATEGORY" => {
                    app.work_category = value.to_owned();
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn add_entry(app: &mut App) -> Result<()> {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let category = app.work_category.clone();
    let work_minutes = app.work_time_duration.as_secs() / 60;
    let break_minutes = app.break_time_duration.as_secs() / 60;
    let entry_content = format!("{};{};{};{}\n", date, category, work_minutes, break_minutes);
    return Ok(OpenOptions::new()
        .append(true)
        .open("pomodoro_stats.csv")?
        .write_all(entry_content.as_bytes())?);
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

fn create_gauge(
    gauge_percentage: f64,
    label: &str,
    fill_color: Color,
    back_color: Color,
) -> Gauge<'_> {
    return Gauge::default()
        .style(Modifier::BOLD)
        .gauge_style(Style::new().fg(fill_color).bg(back_color))
        .label(label)
        .ratio(gauge_percentage);
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
    s -> stops the timer

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
        .constraints(vec![
            Constraint::Min(7),
            Constraint::Percentage(75),
            Constraint::Length(3),
        ])
        .split(area);

    // Timer text creation
    let duration_seconds_remaining: u64 = app.timer_duration.as_secs();
    let m: u64 = duration_seconds_remaining / 60;
    let s: u64 = duration_seconds_remaining % 60;
    let duration_string: Line;
    if app.should_start_break_timer {
        duration_string = Line::from(vec![
            Span::styled("Doing ", Style::default().fg(app.main_color)),
            Span::styled(
                format!("{:02}", app.work_category),
                Style::default().fg(app.accent_color),
            ),
            Span::styled(" for ", Style::default().fg(app.main_color)),
            Span::styled(
                format!("{:02}:{:02}", m, s),
                Style::default().fg(app.accent_color),
            ),
        ]);
    } else if app.should_start_break_timer == false && app.timer_duration.as_secs() > 0 {
        duration_string = Line::from(vec![
            Span::styled(
                "Break time! Having a break for ",
                Style::default().fg(app.main_color),
            ),
            Span::styled(
                format!("{:02}:{:02}", m, s),
                Style::default().fg(app.accent_color),
            ),
        ]);
    } else {
        duration_string = Line::from(vec![Span::styled(
            "All done!",
            Style::default().fg(app.main_color),
        )]);
    }

    // Progress Cauge creation
    let ratio: f64 = if app.work_time_duration.as_secs_f64() > 0.0 {
        1.0 - app.timer_duration.as_secs_f64() / app.work_time_duration.as_secs_f64()
    } else {
        0.0
    };
    let progress_gauge: Gauge = create_gauge(ratio, "", app.accent_color, app.main_color);
    frame.render_widget(ascii_art(app.accent_color), pomodoro_timer_layout[0]);
    frame.render_widget(Paragraph::new(duration_string), pomodoro_timer_layout[1]);
    frame.render_widget(progress_gauge, pomodoro_timer_layout[2]);
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
    let stats_path: String = format!("Stats Path: {}", app.stats_path);
    let category: String = format!("Default Category: {}", app.work_category);
    let work_minutes: String = format!(
        "Default Work Minutes: {}",
        app.work_time_duration.as_secs() / 60,
    );
    let break_minutes: String = format!(
        "Default Break Minutes: {}",
        app.break_time_duration.as_secs() / 60,
    );
    let settings_actions: Vec<String> = vec![
        stats_path,
        category,
        work_minutes,
        break_minutes,
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
