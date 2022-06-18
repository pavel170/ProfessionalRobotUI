use std::{
    error::Error,
    io::{self, Write},
    string::ToString,
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{self, Backend, CrosstermBackend},
    layout::{self, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal,
    text::{Span, Spans},
    widgets::{BarChart, Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};

enum errors {
    Mechanical_Error,
    Sensor_Error,
    Motor_Error,
    Wiring_Error,
}

struct Animation {
    position: u16,
    speed: u16,
    frame_counter: u16,
}

impl Animation {
    fn new() -> Self {
        Animation {
            position: 0,
            speed: 10,
            frame_counter: 0,
        }
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

    fn is_full(&self) -> bool {
        let mut is_full = true;
        for row in self.matrix {
            for entry in row {
                if entry == 0 {
                    is_full = false;
                }
            }
        }
        is_full
    }
}

fn output_array(ingrid: &InputGrid) -> [[u8; 3]; 3] {
    //TODO start the robot
    ingrid.matrix
}

fn print_progress_matrix<B: Backend>(
    f: &mut Frame<B>,
    state_progress_chunk: Vec<Rect>,
    progress: Vec<Vec<usize>>,
) {
    //current progress chart
    let progress_padding = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(state_progress_chunk[1]);

    let border_progress = Block::default()
        .title("Current progress")
        .borders(Borders::ALL);
    f.render_widget(border_progress, state_progress_chunk[1]);
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
        .split(progress_padding[0]);
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
            match progress[i][j] {
                1 => {
                    progress_blocks[i][j] =
                        Block::default().style(Style::default().bg(Color::White))
                }
                2 => {
                    progress_blocks[i][j] =
                        Block::default().style(Style::default().bg(Color::Black))
                }
                _ => {}
            }
            f.render_widget(
                progress_blocks[i][j].clone(),
                //TODO render the colors of the progress
                horizontal_progress_split_vec[i][j],
            );
        }
    }
}

fn print_input<B: Backend>(
    f: &mut Frame<B>,
    ingrid: &mut InputGrid,
    padding_input_matrix: &mut Vec<Rect>,
) {
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
        .split(padding_input_matrix[0]);
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
            if ingrid.is_started == true {
                block = block.border_style(Style::default())
            } else {
                if ingrid.row == i && ingrid.col == j {
                    block = block.border_style(Style::default().fg(Color::Blue));
                }
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
}

fn print_chart<B: Backend>(f: &mut Frame<B>, bar_center_chunk: Vec<Rect>, right_chunk: Vec<Rect>) {
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
}

fn print_animation<B: Backend>(
    f: &mut Frame<B>,
    ingrid: &mut InputGrid,
    left_chunk: Vec<Rect>,
    anime: &mut Animation,
) {
    if ingrid.allowed_started == true {
        let will_start = Block::default()
            .title("Attention")
            .title_alignment(tui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        let text = "You are about to enter unsafe rust!\n Press 'Enter' to proceed.";
        if ingrid.is_started == true {
            //animate the current position of the disk on the belt
            let padding = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .margin(1)
                .split(left_chunk[1]);
            let animation_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(anime.position),
                        Constraint::Length(2 * padding[0].height),
                        Constraint::Length(
                            left_chunk[1].width - 2 * padding[0].height - anime.position,
                        ),
                    ]
                    .as_ref(),
                )
                .split(padding[0]);
            let disk = Block::default()
                .style(Style::default().bg(Color::White))
                .borders(Borders::NONE);
            f.render_widget(disk, animation_chunk[1]);
            if padding[0].width - (2 * padding[0].height) > anime.position {
                if anime.frame_counter < anime.speed {
                    anime.frame_counter += 1;
                } else if anime.frame_counter == anime.speed {
                    anime.frame_counter = 0;
                    anime.position += 1;
                }
            } else {
                anime.position = 0;
            }
            let animation_border = Block::default()
                .title("Disk position")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);
            f.render_widget(animation_border, left_chunk[1]);
        } else {
            let jumpscare = Paragraph::new(text)
                .block(will_start)
                .alignment(tui::layout::Alignment::Center);
            f.render_widget(jumpscare, left_chunk[1]);
        }
    }
}

fn print_state<B: Backend>(f: &mut Frame<B>, state_padding: Vec<Rect>, state: String) {
    let mut message = String::from("[State] \t");
    message.push_str(state.as_str());

    let p = Paragraph::new(message).alignment(layout::Alignment::Center);
    f.render_widget(p, state_padding[0]);
}

fn print_text<B: Backend>(f: &mut Frame<B>, _messages: Vec<String>, print_padding: Vec<Rect>) {
    let mut messages = vec![];
    let mut i = 0;
    for message in _messages {
        messages.push(Spans::from(Span::styled(
            format!("[message {}] \t {} \n", i, message),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        i += 1;
    }
    //let text = Spans::from(messages);
    let p = Paragraph::new(messages);
    f.render_widget(p, print_padding[0]);
}

fn ui<B: Backend>(f: &mut Frame<B>, ingrid: &mut InputGrid, anime: &mut Animation) {
    //layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let left_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(chunks[0]);

    //add the border to the input matrix
    let mut padding_input_matrix = Layout::default()
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(left_chunk[0]);

    let border_input_matrix = Block::default()
        .title("Input matrix")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    f.render_widget(border_input_matrix, left_chunk[0]);

    let right_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(chunks[1]);

    let bar_center_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(right_chunk[0]);

    //splitting the rigt chunk
    let state_prints_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(right_chunk[1]);

    //add padding to the prints
    let print_padding = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(state_prints_chunk[1]);

    let print_border = Block::default()
        .title("Machine output")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(print_border, state_prints_chunk[1]);

    //split the state from progress
    let state_progress_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(state_prints_chunk[0]);

    //add the border to the state
    let state_padding = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(state_progress_chunk[0]);

    let state_border = Block::default()
        .title("Current state")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(state_border, state_progress_chunk[0]);

    print_progress_matrix(f, state_progress_chunk, vec![vec![2; 3]; 3]);
    print_chart(f, bar_center_chunk, right_chunk);
    print_input(f, ingrid, &mut padding_input_matrix);
    print_state(f, state_padding, "State".to_string());
    print_animation(f, ingrid, left_chunk, anime);
    print_text(
        f,
        vec![
            "a".to_string(),
            "a".to_string(),
            "a".to_string(),
            "a".to_string(),
            "a".to_string(),
        ],
        print_padding,
    )
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
        if event::poll(Duration::from_millis(1))? == true {
            if ingrid.is_started == false {
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
                            if ingrid.is_started == false {
                                ingrid.matrix[ingrid.row as usize][ingrid.col as usize] = 1;
                            }
                        }

                        KeyCode::Char('b') => {
                            if ingrid.is_started == false {
                                ingrid.matrix[ingrid.row as usize][ingrid.col as usize] = 2;
                            }
                        }
                        KeyCode::Enter => {
                            if ingrid.is_full() == true {
                                if ingrid.allowed_started == true {
                                    output_array(&ingrid);
                                    ingrid.is_started = true;
                                }
                                ingrid.allowed_started = true;
                            }
                        }
                        KeyCode::Tab => {
                            //terminal.current_buffer_mut().reset();
                            main()?;
                            break;
                        }
                        _ => {}
                    }
                }
            } else {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab => {
                            //terminal.current_buffer_mut().reset();
                            main()?;
                            break;
                        }
                        _ => {}
                    }
                }
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
