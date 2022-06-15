use std::{error::Error, io, thread, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{self, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal,
    widgets::{BarChart, Block, Borders, Paragraph},
    Frame, Terminal,
};

const FRAME_RATE_ANIME: u16 = 1000;

struct Animation {
    position: u16,
}

impl Animation {
    fn new() -> Self {
        Animation { position: 0 }
    }
}

struct InputGrid {
    matrix: [[u8; 3]; 3],
    row: u8,
    col: u8,
    allowed_started: bool,
    is_started: bool,
}

impl InputGrid {
    fn new() -> Self {
        InputGrid {
            matrix: [[0; 3]; 3],
            row: 0,
            col: 0,
            allowed_started: false,
            is_started: false,
        }
    }
}

fn output_array(ingrid: &InputGrid) -> [[u8; 3]; 3] {
    //TODO start the robot
    ingrid.matrix
}

fn ui<B: Backend>(f: &mut Frame<B>, ingrid: &mut InputGrid, anime: &mut Animation) {
    //layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let left_chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(chunks[0]);

    let right_chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(chunks[1]);

    let bar_center_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(right_chunk[0]);

    //splitting the rigt-down corner
    let state_prints_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(right_chunk[1]);

    let state_progress_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(state_prints_chunk[0]);

    //current progress chart
    let current_progress_block = Block::default()
        .title("Current progress")
        .borders(Borders::ALL);
    f.render_widget(current_progress_block, state_progress_chunk[1]);
    //split the grid
    let vertical_progress_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ]
            .as_ref(),
        )
        .split(state_progress_chunk[1]);
    let mut horizontal_progress_split_vec = vec![vec![Rect::default()]; 3];
    let mut progress_blocks = vec![vec![Block::default(); 3]; 3];
    for i in 0..3 {
        horizontal_progress_split_vec[i] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(vertical_progress_split[i]);
    }
    //place the blocks in the grid
    for i in 0..3 {
        for j in 0..3 {
            progress_blocks[i][j] = Block::default().borders(Borders::ALL);
            f.render_widget(
                progress_blocks[i][j].clone(),
                //TODO render the colors of the progress
                horizontal_progress_split_vec[i][j],
            );
        }
    }

    //RGB Chart
    let title_block = Block::default().title("RGB Chart").borders(Borders::ALL);
    f.render_widget(title_block, right_chunk[0]);
    let chart = BarChart::default()
        .bar_width(5)
        .bar_gap(3)
        .bar_style(Style::default().fg(Color::Yellow))
        .label_style(Style::default().fg(Color::White))
        .data(&[("R", 2), ("G", 4), ("B", 3)])
        .max(4);

    f.render_widget(chart, bar_center_chunk[1]);

    //input grid
    let input_columns = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ]
            .as_ref(),
        )
        .split(left_chunk[0]);
    let mut temp_array = vec![vec![Rect::default()]; 3];
    let mut block_array = vec![vec![Block::default(); 3]; 3];
    for i in 0..3 {
        temp_array[i] = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Ratio(2, 6),
                    Constraint::Ratio(2, 6),
                    Constraint::Ratio(2, 6),
                ]
                .as_ref(),
            )
            .split(input_columns[i]);
    }
    for i in 0..3 {
        for j in 0..3 {
            let mut block = Block::default().borders(Borders::ALL);
            if ingrid.row == i && ingrid.col == j {
                block = block.border_style(Style::default().fg(Color::Blue));
            }
            if ingrid.matrix[i as usize][j as usize] == 1 {
                block = block.style(Style::default().bg(Color::White));
            }

            if ingrid.matrix[i as usize][j as usize] == 2 {
                block = block.style(Style::default().bg(Color::Black));
            }

            f.render_widget(block.clone(), temp_array[j as usize][i as usize]);
            block_array[j as usize][i as usize] = block;
        }
    }

    if ingrid.allowed_started == true {
        let will_start = Block::default()
            .title("Attention")
            .title_alignment(tui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        let text = "You are about to enter unsafe rust!\n Press 'Enter' to proceed.";
        if ingrid.is_started == true {
            //animate the current position of the disk on the belt
            let animation_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(anime.position),
                        Constraint::Length(2 * left_chunk[1].height),
                        Constraint::Length(
                            left_chunk[1].width - 2 * left_chunk[1].height - anime.position,
                        ),
                    ]
                    .as_ref(),
                )
                .split(left_chunk[1]);
            let disk = Block::default()
                .style(Style::default().bg(Color::White))
                .borders(Borders::NONE);
            f.render_widget(disk, animation_chunk[1]);
            anime.position += 1;
        } else {
            let jumpscare = Paragraph::new(text)
                .block(will_start)
                .alignment(tui::layout::Alignment::Center);
            f.render_widget(jumpscare, left_chunk[1]);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture,)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut ingrid = InputGrid::new();
    let mut animation = Animation::new();

    loop {
        terminal.draw(|f| ui(f, &mut ingrid, &mut animation))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Left => {
                    if ingrid.col > 0 {
                        ingrid.col -= 1;
                    }
                }
                KeyCode::Right => {
                    if ingrid.col < 2 {
                        ingrid.col += 1;
                    }
                }
                KeyCode::Up => {
                    if ingrid.row > 0 {
                        ingrid.row -= 1;
                    }
                }
                KeyCode::Down => {
                    if ingrid.row < 2 {
                        ingrid.row += 1;
                    }
                }
                KeyCode::Char('w') => {
                    ingrid.matrix[ingrid.row as usize][ingrid.col as usize] = 1;
                }

                KeyCode::Char('b') => {
                    ingrid.matrix[ingrid.row as usize][ingrid.col as usize] = 2;
                }
                KeyCode::Enter => {
                    if ingrid.allowed_started == true {
                        output_array(&ingrid);
                        ingrid.is_started = true;
                    }
                    ingrid.allowed_started = true;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
