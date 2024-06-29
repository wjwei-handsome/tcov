use crate::cli;
use anyhow::Result;
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    widgets::{Block, Paragraph, Sparkline},
};
use std::{
    io,
    time::{Duration, Instant},
};

/// A struct to hold the data and view of the coverage data
struct CovView {
    data: Vec<u64>,
    legend: String,
    view_start: u32,
    view_end: u32,
    label_start: u32,
}

impl CovView {
    // create new instance
    fn new(data: Vec<u64>, legend: String, init_width: u16, label_start: u32) -> Self {
        let view_end = if data.len() > init_width.into() {
            init_width as u32
        } else {
            data.len() as u32
        };
        Self {
            data,
            legend,
            view_start: 0,
            view_end,
            label_start,
        }
    }

    // update `view_start`  `view_end` `label_start`
    fn move_view(&mut self, direction: i32, curr_view_size: u16) {
        // diff between label_start and view_start
        let label_view_diff = self.label_start as i32 - self.view_start as i32;
        let curr_view_size = curr_view_size as u32;
        let data_len = self.data.len() as u32;

        // get step size == abs(direction)
        let step_size = direction.unsigned_abs();

        if direction < 0 && self.view_start > 0 {
            // sub
            self.view_start = self.view_start.saturating_sub(step_size);
            // compare with data_len
            self.view_end = u32::min(self.view_start + curr_view_size, data_len);
        } else if direction > 0 && self.view_end < data_len {
            if self.view_end + step_size > data_len {
                // check boundary
                // compute the sub
                let sub = data_len - self.view_end;
                // touch the bottom
                self.view_end = data_len;
                // update view_start
                self.view_start = self.view_start.saturating_sub(sub);
            } else {
                self.view_end += step_size;
                self.view_start = self.view_end - curr_view_size;
            }
        }
        // update label_start
        self.label_start = self.view_start + label_view_diff as u32;
    }
}

/// main function to run the tui
pub fn tview(
    data: Vec<u64>,
    start: u32,
    legend: String,
    step: u8,
    color: cli::Color,
) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // get initial width
    let init_width = terminal.size()?.width;

    // parse color to crossterm color
    let color = color.to_string().parse::<Color>()?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = CovView::new(data, legend, init_width, start);
    let res = run_app(&mut terminal, app, tick_rate, step, color);

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

// run the app
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: CovView,
    tick_rate: Duration,
    size: u8,
    color: Color,
) -> Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app, color))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        // get current width in loop
        let curr_width = terminal.size()?.width;
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => {
                        // move view to left
                        let dir_step = -(size as i32);
                        app.move_view(dir_step, curr_width)
                    }
                    KeyCode::Right => {
                        // move view to right
                        let dir_step = size as i32;
                        app.move_view(dir_step, curr_width)
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

// draw the ui
fn ui(f: &mut Frame, app: &CovView, color: Color) {
    // get full size and split it to chunks
    let full = f.size();
    let width = full.width;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        // .margin(1)
        .constraints(
            [
                Constraint::Percentage(96), // 96% for sparkline
                Constraint::Percentage(2),  // 2% for label
                Constraint::Percentage(2),  // 2% for help text
            ]
            .as_ref(),
        )
        .split(full);

    // re-generate legend
    let curr_max = app.data[app.view_start as usize..app.view_end as usize]
        .iter()
        .max()
        .unwrap_or(&0);
    let legend = format!("{} (current max: {})", app.legend, curr_max);

    let sparkline = Sparkline::default()
        .block(
            Block::new()
                .title(legend)
                .title_alignment(Alignment::Center),
        )
        .data(&app.data[app.view_start as usize..app.view_end as usize])
        .style(Style::default().fg(color));

    f.render_widget(sparkline, chunks[0]);

    let label_end = app.label_start + width as u32;
    let fmt_label = generate_and_format_dynamic_label(app.label_start, label_end, chunks[1].width);

    let label_paragraph = Paragraph::new(fmt_label).style(Style::default().fg(Color::Cyan));
    f.render_widget(label_paragraph, chunks[1]);

    let help_text = "Press ◄ ► to scroll, 'q' to quit";
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(help_paragraph, chunks[2]);
}

// generate dynamic label
fn generate_and_format_dynamic_label(label_start: u32, label_end: u32, axis_width: u16) -> String {
    let start_label = format!("{:09}", label_start);
    let end_label = format!("{:09}", label_end);
    // compute the space between start and end
    let space = " ".repeat((axis_width - 18) as usize);
    format!("{}{}{}", start_label, space, end_label)
}
