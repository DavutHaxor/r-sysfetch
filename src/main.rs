#![allow(unused)]

mod collectors {
    pub mod cpu;
    pub mod gpu;
    pub mod mem;
}

use collectors::cpu::*;
use collectors::gpu::*;
use collectors::mem::*;

use ratatui::crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::{
    char,
    collections::HashMap,
    env,
    fmt::{Write, format},
    fs::{self, exists},
    path::PathBuf,
    process::exit,
    thread, time,
};

fn main() {
    let config_path = get_config_path();
    create_config(&config_path);
    let Ok((flags, gpu_ids)) = read_config(&config_path) else {
        eprintln!("Failed to read config");
        return;
    };
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() > 1 {
        let mut arg_count = HashMap::new();
        for arg in arguments.iter().skip(1) {
            *arg_count.entry(arg).or_insert(0) += 1;
        }
        if arg_count.contains_key(&"cpu".to_string()) {
            print_cpu();
        }
        if arg_count.contains_key(&"mem".to_string()) {
            print_mem();
        }
        if arg_count.contains_key(&"gpu".to_string()) {
            print_gpu();
        }
        if arg_count.contains_key(&"a".to_string()) {
            print_cpu();
            print_mem();
            print_gpu();
        }
        if arg_count.contains_key(&"t".to_string()) {
            print_cpu_from_config(&flags);
        }
    } else {
        run_tui().expect("TUI failed");
    }
}

fn run_tui() -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            // Outer vertical split: top row + bottom row
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            // Top row: CPU (left) | MEM (right)
            let top_cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(rows[0]);

            // CPU box
            let cpu_block = Block::default()
                .title(Span::styled(
                    " ⚡ CPU ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black));

            let cpu_lines = vec![
                Line::from(vec![
                    Span::styled("  Model      ", Style::default().fg(Color::DarkGray)),
                    Span::styled(cpu_model_name(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("  Usage      ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2}%", cpu_usage()),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Frequency  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2} GHz", cpu_freq()),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Temp       ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.1} °C", cpu_temperature() / 1000.0),
                        Style::default().fg(Color::Red),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Cores      ", Style::default().fg(Color::DarkGray)),
                    Span::styled(cpu_cores(), Style::default().fg(Color::Magenta)),
                ]),
                Line::from(vec![
                    Span::styled("  Threads    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(cpu_threads(), Style::default().fg(Color::Magenta)),
                ]),
            ];

            frame.render_widget(
                Paragraph::new(cpu_lines)
                    .block(cpu_block)
                    .alignment(Alignment::Left),
                top_cols[0],
            );

            // MEM box
            let mem_block = Block::default()
                .title(Span::styled(
                    " 🧠 Memory ",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Magenta))
                .style(Style::default().bg(Color::Black));

            let (swap_total, swap_free) = mem_swap_info();
            let mem_lines = vec![
                Line::from(vec![
                    Span::styled("  Total      ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2} GB", mem_total()),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Free       ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2} GB", mem_free()),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Available  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2} GB", mem_available()),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Swap Total ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2} GB", swap_total),
                        Style::default().fg(Color::Blue),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Swap Free  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2} GB", swap_free),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
            ];

            frame.render_widget(
                Paragraph::new(mem_lines)
                    .block(mem_block)
                    .alignment(Alignment::Left),
                top_cols[1],
            );

            // GPU box
            let gpu_block = Block::default()
                .title(Span::styled(
                    " 🎮 GPU (card1) ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black));

            let (vram_total, vram_used) = gpu_vram('1');
            let (power_used, power_max) = gpu_power('1');
            let (core_speed, mem_speed) = gpu_clock_speeds('1');
            let gpu_usage = gpu_usage('1');
            let gpu_lines = vec![
                Line::from(vec![
                    Span::styled("  VRAM Total    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2} GB", vram_total),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled("    VRAM Used    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2} GB", vram_used),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Power Used    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{} W", power_used), Style::default().fg(Color::Red)),
                    Span::styled("    Power Max    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{} W", power_max),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Usage  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.2} %", gpu_usage),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("  Temperature   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{} °C", gpu_temp('1')),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  Core Clock    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{} MHz", core_speed),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled("    Memory Clock ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{} MHz", mem_speed),
                        Style::default().fg(Color::Magenta),
                    ),
                ]),
            ];

            frame.render_widget(
                Paragraph::new(gpu_lines)
                    .block(gpu_block)
                    .alignment(Alignment::Left),
                rows[1],
            );
        })?;

        // Poll with timeout so the UI can refresh; quit on 'q' / Esc
        if event::poll(std::time::Duration::from_millis(2000))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

// CLI print helpers

fn print_cpu() {
    println!("CPU");
    println!("  {}", cpu_model_name());
    println!("  Usage: {:.2}%", cpu_usage());
    println!("  Frequency: {:.1} GHz", cpu_freq());
    println!("  Temperature: {:.1} °C", cpu_temperature() / 1000.0);
    println!("  Cores: {}", cpu_cores());
    println!("  Threads: {}", cpu_threads());
}

fn print_mem() {
    println!("MEM");
    println!("  Total: {:.1} GB", mem_total());
    println!("  Free: {:.1} GB", mem_free());
    println!("  Available: {:.1} GB", mem_available());
    let (swap_total, swap_free) = mem_swap_info();
    print!(
        "  Swap Total: {:.1} GB\n  Swap Free: {:.1} GB\n",
        swap_total, swap_free
    );
}

fn print_gpu() {
    println!("GPU");
    let (vram_total, vram_used) = gpu_vram('1');
    print!(
        "  VRAM Total: {:.2} GB\n  VRAM Used: {:.2} GB\n",
        vram_total, vram_used
    );
    let (power_used, power_max) = gpu_power('1');
    print!(
        "  Power Used: {} Watts\n  Power Max: {} Watts\n",
        power_used, power_max
    );
    println!("  Temperature: {} °C", gpu_temp('1'));
    let (core_speed, mem_speed) = gpu_clock_speeds('1');
    print!(
        "  Core Speed: {} MHz\n  Memory Speed: {} MHz\n",
        core_speed, mem_speed
    );
}

fn get_config_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("r-sysfetch.conf")
}

fn create_config(config_file: &PathBuf) {
    if !config_file.exists() {
        let config_content = r#"#r-sysfetch configuration file!
cpu_model_name
cpu_usage
cpu_frequency
cpu_temperature
cpu_cores
cpu_threads

mem_total
mem_free
mem_available
mem_swap_info

gpu_vram
gpu_power
gpu_temp
gpu_clock_speeds

#gpu=0
gpu=1
"#;
        fs::write(&config_file, &config_content).ok();
    }
}

fn read_config(
    config_file: &PathBuf,
) -> Result<(HashMap<String, bool>, Vec<String>), std::io::Error> {
    let content = fs::read_to_string(config_file)?;
    let mut gpu_ids = Vec::new();
    let mut flags = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(stripped) = line.strip_prefix("gpu=") {
            gpu_ids.push(stripped.to_string());
            continue;
        }
        flags.insert(line.to_string(), true);
    }

    Ok((flags, gpu_ids))
}

fn print_cpu_from_config(flags: &HashMap<String, bool>) {
    let cpu_group = [
        "cpu_model_name",
        "cpu_usage",
        "cpu_frequency",
        "cpu_temperature",
        "cpu_cores",
        "cpu_threads",
    ];
    let mut cpu_printed = false;

    for key in cpu_group {
        if flags.contains_key(key) {
            if !cpu_printed {
                println!("CPU");
                cpu_printed = true;
            }
            match key {
                "cpu_model_name" => println!("  {}", cpu_model_name()),
                "cpu_usage" => println!("  Usage: {:.2}%", cpu_usage()),
                "cpu_frequency" => println!("  Frequency: {:.1} GHz", cpu_freq()),
                "cpu_temperature" => {
                    println!("  Temperature: {:.1} °C", cpu_temperature() / 1000.0)
                }
                "cpu_cores" => println!("  Cores: {}", cpu_cores()),
                "cpu_threads" => println!("  Threads: {}", cpu_threads()),
                _ => {}
            }
        }
    }
}
