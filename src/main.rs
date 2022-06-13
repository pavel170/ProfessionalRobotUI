use std::{error::Error, io, thread, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

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

fn ui<B: Backend>(f: &mut Frame<B>, ingrid: &mut InputGrid) {
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
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(chunks[1]);

    //input grid

    let input_columns = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
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
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
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
        let mut text = "You are about to enter unsafe rust!\n Press 'Enter' to proceed.";
        if ingrid.is_started == true {
            text = "You did it. Now the robot is running!";
        }
        let jumpscare = Paragraph::new(text)
            .block(will_start)
            .alignment(tui::layout::Alignment::Center);
        f.render_widget(jumpscare, left_chunk[1]);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture,)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut ingrid = InputGrid::new();

    loop {
        terminal.draw(|f| ui(f, &mut ingrid))?;
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
