use library::{config, obsidian::print_sprint_balance};
use notify::{self, RecursiveMode, Watcher};
use time;

use crossterm;
use ratatui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        // crossterm::event::EnableMouseCapture
    )?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Run the whole thing
    run(&mut terminal)?;

    // Bye!
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(render_app)?;
        if should_quit()? {
            break;
        }
    }
    Ok(())
}

fn render_app(frame: &mut ratatui::Frame) {
    let greeting = ratatui::widgets::Paragraph::new("Youu what the hell are you trying");
    frame.render_widget(greeting, frame.size());
}

fn should_quit() -> Result<bool, Box<dyn std::error::Error>> {
    if crossterm::event::poll(std::time::Duration::from_millis(250))? {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            return Ok(key.code == crossterm::event::KeyCode::Char('q'));
        }
    }
    Ok(false)
}

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
