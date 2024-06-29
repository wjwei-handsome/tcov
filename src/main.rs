use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
};
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    widgets::{Block, Paragraph, Sparkline},
};

#[derive(Clone)]
struct RandomSignal {
    distribution: Uniform<u64>,
    rng: ThreadRng,
}

impl RandomSignal {
    fn new(lower: u64, upper: u64) -> Self {
        Self {
            distribution: Uniform::new(lower, upper),
            rng: rand::thread_rng(),
        }
    }
}

impl Iterator for RandomSignal {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        Some(self.distribution.sample(&mut self.rng))
    }
}

struct App {
    data1: Vec<u64>,
}

impl App {
    fn new() -> Self {
        let mut signal = RandomSignal::new(0, 100);
        let data1 = signal.by_ref().take(200).collect::<Vec<u64>>();

        Self { data1 }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let full = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(92), // 92% for sparkline
                Constraint::Percentage(8),  // 8% for label
            ]
            .as_ref(),
        )
        .split(full);
    let sparkline = Sparkline::default()
        .block(Block::new().title("Data1"))
        .data(&app.data1)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(sparkline, chunks[0]);

    // a test case
    let formatted_label = generate_and_format_label(2078687, 2079869, chunks[1].width as usize);

    let label_paragraph = Paragraph::new(formatted_label).style(Style::default().fg(Color::White));
    f.render_widget(label_paragraph, chunks[1]);
}

fn generate_and_format_label(start: i64, end: i64, axis_width: usize) -> String {
    // gap number = 10 -1 = 9
    let step = (end - start) / 9;
    let labels = (0..10)
        .map(|i| format!("{:09}", start + step * i))
        .collect::<Vec<String>>();
    // 30 + 9w = axis_width
    // w = (axis_width - 30) / 9
    let space_width = (axis_width - 30) / 9;
    // println!("space_width: {}", space_width);
    let space = " ".repeat(space_width);
    // horizontal axis labels
    let mut labels_display = vec![String::new(); 3]; // three lines
    for label in labels.iter() {
        let parts: Vec<&str> = label
            .as_bytes()
            .chunks(3)
            .map(std::str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()
            .unwrap();

        for (i, part) in parts.into_iter().enumerate() {
            labels_display[i].push_str(&space); // add width=space
            labels_display[i].push_str(part);
        }
    }

    // remove first space
    let mut new_labels_display = Vec::new();
    for label in labels_display {
        // remove first space
        let label = label.chars().skip(space_width).collect::<String>();
        new_labels_display.push(label);
    }

    // join three lines
    new_labels_display.join("\n")
}
