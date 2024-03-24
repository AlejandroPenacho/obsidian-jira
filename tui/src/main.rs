use library::{config, obsidian::TaskTimeData};
use notify::{self, RecursiveMode, Watcher};
use time;

use crossterm;
use ratatui::{self, prelude::Constraint};

type Terminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

struct App {
    data: library::obsidian::SprintTimeBalance,
}

impl App {
    pub fn new() -> Self {
        let iso_week = config::CONFIG
            .get_week()
            .unwrap_or_else(|| time::OffsetDateTime::now_local().unwrap().date().iso_week());

        Self {
            data: library::obsidian::SprintTimeBalance::new(2024, iso_week),
        }
    }
    pub fn reload(&mut self) {
        let iso_week = config::CONFIG
            .get_week()
            .unwrap_or_else(|| time::OffsetDateTime::now_local().unwrap().date().iso_week());

        self.data = library::obsidian::SprintTimeBalance::new(2024, iso_week);
    }
}

enum Event {
    Quit,
    Reload,
    Other,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let mut terminal = setup_terminal()?;
    // Run the whole thing

    let mut app = App::new();
    run(&mut app, &mut terminal)?;

    // Bye!
    restore_terminal(terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal, Box<dyn std::error::Error>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        // crossterm::event::EnableMouseCapture
    )?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let terminal = ratatui::Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal) -> Result<(), Box<dyn std::error::Error>> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run(
    app: &mut App,
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| render_app(&app, f))?;
        let event = get_event()?;
        match event {
            Event::Quit => break,
            Event::Reload => app.reload(),
            Event::Other => {}
        };
    }
    Ok(())
}

fn render_app(app: &App, frame: &mut ratatui::Frame) {
    let layout = ratatui::layout::Layout::horizontal([
        ratatui::layout::Constraint::Percentage(70),
        ratatui::layout::Constraint::Percentage(30),
    ])
    .split(frame.size());

    let n_rows_in_sprint = app.data.tasks.iter().filter(|x| x.in_sprint).count();

    let right_block = ratatui::widgets::Block::bordered().title("Greeting");
    let left_block = ratatui::widgets::Block::bordered()
        .title("Table")
        .padding(ratatui::widgets::Padding::symmetric(2, 1));

    let left_inner = left_block.inner(layout[0]);
    let right_inner = right_block.inner(layout[1]);

    let top_left_inner = ratatui::layout::Layout::vertical([
        Constraint::Length(app.data.tasks.len() as u16 + 3),
        Constraint::Length(5),
    ])
    .spacing(3)
    .split(left_inner);

    let schedule_rows =
        ratatui::layout::Layout::vertical([Constraint::Length(1)].repeat(app.data.tasks.len() + 3))
            .split(top_left_inner[0]);

    let full_schedule_layout: Vec<_> = (0..(app.data.tasks.len() + 3))
        .map(|row_index| {
            ratatui::layout::Layout::horizontal([
                ratatui::layout::Constraint::Length(60),
                ratatui::layout::Constraint::Length(5),
                ratatui::layout::Constraint::Length(5),
                ratatui::layout::Constraint::Fill(1),
                ratatui::layout::Constraint::Length(6),
            ])
            .spacing(1)
            .split(schedule_rows[row_index])
        })
        .collect();

    // Executed time | Planned time | Remaining time
    let max_seconds: f64 = app
        .data
        .tasks
        .iter()
        .map(|x| x.remaining_time.max(x.uncompleted_time + x.completed_time))
        .max()
        .unwrap()
        .as_seconds_f64();

    let important_times: [time::Duration; 3] = [
        time::Duration::hours(40),
        app.data
            .tasks
            .iter()
            .map(|x| (x.uncompleted_time + x.completed_time))
            .sum(),
        app.data
            .tasks
            .iter()
            .filter(|x| !x.in_sprint)
            .map(|x| (x.uncompleted_time + x.completed_time))
            .sum(),
    ];

    frame.render_widget(
        ratatui::widgets::Paragraph::new("Name"),
        full_schedule_layout[0][0],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new("Plan"),
        full_schedule_layout[0][1],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new("Alloc"),
        full_schedule_layout[0][2],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new("Diff"),
        full_schedule_layout[0][4],
    );

    for (task_index, row_index) in (2..n_rows_in_sprint + 2)
        .chain(n_rows_in_sprint + 3..app.data.tasks.len() + 3)
        .enumerate()
    {
        let task = &app.data.tasks[task_index];
        let name = task.name.clone();

        frame.render_widget(
            ratatui::widgets::Paragraph::new(name),
            full_schedule_layout[row_index][0],
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format_time(task.remaining_time, false)),
            full_schedule_layout[row_index][1],
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format_time(
                task.completed_time + task.uncompleted_time,
                false,
            )),
            full_schedule_layout[row_index][2],
        );
        render_time_bar(task, max_seconds, frame, full_schedule_layout[row_index][3]);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format_time(
                task.completed_time + task.uncompleted_time - task.remaining_time,
                true,
            )),
            full_schedule_layout[row_index][4],
        );
    }

    let greeting = ratatui::widgets::Paragraph::new("Youu what the hell are you trying");
    frame.render_widget(left_block, layout[0]);
    frame.render_widget(right_block, layout[1]);
    frame.render_widget(greeting.clone(), right_inner);

    let general_table_layout = ratatui::layout::Layout::vertical([Constraint::Length(1)].repeat(5))
        .split(top_left_inner[1]);

    let general_table_layout: Vec<_> = std::iter::repeat([
        ratatui::layout::Constraint::Length(60),
        ratatui::layout::Constraint::Length(5),
        ratatui::layout::Constraint::Length(5),
    ])
    .take(5)
    .enumerate()
    .map(|(i, x)| {
        ratatui::layout::Layout::horizontal(x)
            .spacing(1)
            .split(general_table_layout[i])
    })
    .collect();

    frame.render_widget(
        ratatui::widgets::Paragraph::new("Class").right_aligned(),
        general_table_layout[0][0],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new("Planned"),
        general_table_layout[0][1],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new("Allocated"),
        general_table_layout[0][2],
    );

    frame.render_widget(
        ratatui::widgets::Paragraph::new("Tasks").right_aligned(),
        general_table_layout[2][0],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new(format_time(
            app.data
                .tasks
                .iter()
                .filter(|x| x.in_sprint)
                .map(|x| x.remaining_time)
                .sum(),
            false,
        )),
        general_table_layout[2][1],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new(format_time(
            app.data
                .tasks
                .iter()
                .filter(|x| x.in_sprint)
                .map(|x| x.uncompleted_time + x.completed_time)
                .sum(),
            false,
        )),
        general_table_layout[2][2],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new("Additionals").right_aligned(),
        general_table_layout[3][0],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new(format_time(
            app.data
                .tasks
                .iter()
                .filter(|x| !x.in_sprint)
                .map(|x| x.uncompleted_time + x.completed_time)
                .sum(),
            false,
        )),
        general_table_layout[3][1],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new(format_time(
            app.data
                .tasks
                .iter()
                .filter(|x| !x.in_sprint)
                .map(|x| x.uncompleted_time + x.completed_time)
                .sum(),
            false,
        )),
        general_table_layout[3][2],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new("Total").right_aligned(),
        general_table_layout[4][0],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new(format_time(
            app.data.tasks.iter().map(|x| x.remaining_time).sum(),
            false,
        )),
        general_table_layout[4][1],
    );
    frame.render_widget(
        ratatui::widgets::Paragraph::new(format_time(
            app.data
                .tasks
                .iter()
                .map(|x| x.uncompleted_time + x.completed_time)
                .sum(),
            false,
        )),
        general_table_layout[4][2],
    );
}

fn render_time_bar(
    task: &TaskTimeData,
    max_time: f64,
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
) {
    use ratatui::style::Color::*;
    let second_time = (task.completed_time + task.uncompleted_time).min(task.remaining_time);
    let third_time = task
        .remaining_time
        .max(task.completed_time + task.uncompleted_time);

    let last_color: ratatui::style::Color;

    if task.remaining_time > task.completed_time + task.uncompleted_time {
        last_color = LightRed;
    } else {
        last_color = LightGreen;
    };

    let values = [
        100.0 * third_time.as_seconds_f64() / max_time,
        100.0 * second_time.as_seconds_f64() / max_time,
        100.0 * task.completed_time.as_seconds_f64() / max_time,
    ];

    let bar = ratatui::widgets::canvas::Canvas::default()
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 1.0])
        .marker(ratatui::symbols::Marker::Braille)
        .paint(|ctx| {
            for y in [0.0, 0.333, 0.666, 1.0] {
                for (color, value) in [last_color, LightBlue, White].iter().zip(values) {
                    ctx.draw(&ratatui::widgets::canvas::Line {
                        x1: 0.0,
                        y1: y,
                        x2: value,
                        y2: y,
                        color: *color,
                    })
                }
            }
        });

    frame.render_widget(bar, area);
}

fn format_time(time: time::Duration, signed: bool) -> String {
    let positive = !time.is_negative();
    let time = time.abs();
    let hours = time.whole_hours();
    let minutes = time.whole_minutes() - 60 * hours;
    let sign = if signed {
        if positive {
            "+"
        } else {
            "-"
        }
    } else {
        ""
    };
    format!("{}{:0>2}:{:0>2}", sign, hours, minutes)
}

fn get_event() -> Result<Event, Box<dyn std::error::Error>> {
    let crossterm::event::Event::Key(key) = crossterm::event::read()? else {
        return Ok(Event::Other);
    };

    match key.code {
        crossterm::event::KeyCode::Char('q') => Ok(Event::Quit),
        crossterm::event::KeyCode::Char('r') => Ok(Event::Reload),
        _ => Ok(Event::Other),
    }
}

/*
fn run_time_loop() -> notify::Result<()> {
    let iso_week = config::CONFIG
        .get_week()
        .unwrap_or_else(|| time::OffsetDateTime::now_local().unwrap().date().iso_week());

    print_sprint_balance(2024, iso_week);

    let watch_path: std::path::PathBuf = [
        config::CONFIG.get_vault_path(),
        config::CONFIG.get_daily_notes_path(),
    ]
    .iter()
    .collect();

    let (tx, rx) = std::sync::mpsc::channel();

    println!("Watching {:?}", watch_path);

    let mut watcher = notify::RecommendedWatcher::new(tx, notify::Config::default()).unwrap();

    watcher.watch(&watch_path, RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(_) => {
                println!("\n\n");
                print_sprint_balance(2024, iso_week);
            }
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}
*/
