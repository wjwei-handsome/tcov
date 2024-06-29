mod cov;
use std::{
    error::Error,
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

use cov::{DefaultReadFilter, OnlyDepthProcessor};
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    widgets::{Block, Paragraph, Sparkline},
};

struct App {
    data: Vec<u64>,
    legend: String,
    view_start: usize,
    view_end: usize,
    label_start: usize,
    label_end: usize,
    max_width: u16,
}

impl App {
    fn new(data: Vec<u64>, legend: String, width: u16, label_start: usize) -> Self {
        let view_end = if data.len() > width.into() {
            width as usize
        } else {
            data.len()
        };
        let label_end = label_start + width as usize;
        Self {
            data,
            legend,
            view_start: 0,
            view_end,
            label_start,
            label_end,
            max_width: width,
        }
    }

    // 添加方法来更新 view_start 和 view_end
    fn move_view(&mut self, direction: i32) {
        let label_view_diff = self.label_start as i32 - self.view_start as i32;
        let max_view_size = self.max_width as usize;
        let data_len = self.data.len();
        if direction < 0 && self.view_start > 0 {
            self.view_start = self.view_start.saturating_sub(10);
            self.view_end = usize::min(self.view_start + max_view_size, data_len);
        } else if direction > 0 && self.view_end < data_len {
            if self.view_end + 10 > data_len {
                let sub = data_len - self.view_end;
                self.view_end = data_len;
                self.view_start = self.view_start.saturating_sub(sub);
            } else {
                self.view_end += 10;
                self.view_start = self.view_end - max_view_size;
            }
        }
        self.label_start = self.view_start + label_view_diff as usize;
        self.label_end = self.view_end + label_view_diff as usize;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let size = terminal.size()?;
    let width = size.width;

    let read_filter = DefaultReadFilter::new(0, 0, 0);
    let bam_path = PathBuf::from("test.bam"); // check it
    let depth_processer = OnlyDepthProcessor::new(bam_path, read_filter);
    let res = depth_processer.process_region("2", 2078887, 2079669)?;

    let data = res.iter().map(|x| x.depth as u64).collect();

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new(data, "aaaaa".to_string(), width, 2078887);
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
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => app.move_view(-10), // set flexible view size
                    KeyCode::Right => app.move_view(10),
                    _ => {}
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
        .block(Block::new().title(app.legend.as_str()))
        .data(&app.data[app.view_start..app.view_end])
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(sparkline, chunks[0]);

    // a test case
    // let formatted_label = generate_and_format_label(2078687, 2079869, chunks[1].width as usize);

    let fmt_label =
        generate_and_format_dynamic_label(app.label_start, app.label_end, chunks[1].width as usize);

    let label_paragraph = Paragraph::new(fmt_label).style(Style::default().fg(Color::White));
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

fn generate_and_format_dynamic_label(
    label_start: usize,
    label_end: usize,
    axis_width: usize,
) -> String {
    let start_label = format!("{:09}", label_start);
    let end_label = format!("{:09}", label_end);

    // compute the space between start and end
    let space = " ".repeat(axis_width - start_label.len() - end_label.len());
    format!("{}{}{}", start_label, space, end_label)
}
