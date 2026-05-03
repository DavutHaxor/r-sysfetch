#![allow(unused)]

use crate::collectors::cpu::*;
use crate::collectors::gpu::*;
use crate::collectors::mem::*;

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

pub fn gpu_box_lines(gpu_id: &String) -> Vec<Line<'static>> {
    let (vram_total, vram_used) = gpu_vram(gpu_id);
    let (power_used, power_max) = gpu_power(gpu_id);
    let (core_speed, mem_speed) = gpu_clock_speeds(gpu_id);
    let usage = gpu_usage(gpu_id);
    vec![
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
                format!("{:.2} %", usage),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Temperature   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} °C", gpu_temp(gpu_id)),
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
    ]
}

pub fn run_tui_dual_gpu(gpu_id1: &String, gpu_id2: &String) -> std::io::Result<()> {
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

            // Bottom row: GPU card1 (left) | GPU card2 (right)
            let bottom_cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(rows[1]);

            // ── CPU box ──────────────────────────────────────────
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

            // ── MEM box ──────────────────────────────────────────
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

            // ── GPU card1 box (bottom-left) ───────────────────────
            let gpu1_block = Block::default()
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

            frame.render_widget(
                Paragraph::new(gpu_box_lines(gpu_id1))
                    .block(gpu1_block)
                    .alignment(Alignment::Left),
                bottom_cols[0],
            );

            // ── GPU card2 box (bottom-right) ──────────────────────
            let gpu2_block = Block::default()
                .title(Span::styled(
                    " 🎮 GPU (card2) ",
                    Style::default()
                        .fg(Color::LightYellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::LightYellow))
                .style(Style::default().bg(Color::Black));

            frame.render_widget(
                Paragraph::new(gpu_box_lines(gpu_id2))
                    .block(gpu2_block)
                    .alignment(Alignment::Left),
                bottom_cols[1],
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

pub fn run_tui_single_gpu(gpu_id1: &String) -> std::io::Result<()> {
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

            let (vram_total, vram_used) = gpu_vram(gpu_id1);
            let (power_used, power_max) = gpu_power(gpu_id1);
            let (core_speed, mem_speed) = gpu_clock_speeds(gpu_id1);
            let gpu_usage = gpu_usage(gpu_id1);
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
                        format!("{} °C", gpu_temp(gpu_id1)),
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
