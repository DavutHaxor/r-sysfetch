#![allow(unused)]

mod collectors {
    pub mod cpu;
    pub mod gpu;
    pub mod mem;
}

pub mod tui;

use collectors::cpu::*;
use collectors::gpu::*;
use collectors::mem::*;

use tui::*;

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
            print_mem_from_config(&flags);
            print_gpu_from_config(&flags, &gpu_ids);
        }
    } else {
        match gpu_ids.len() {
            0 => eprintln!("No GPU detected in config file"),
            1 => run_tui_single_gpu(&gpu_ids[0]).expect("TUI failed"),
            _ => run_tui_dual_gpu(&gpu_ids[0], &gpu_ids[1]).expect("TUI failed"),
        }
    }
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
    let (vram_total, vram_used) = gpu_vram(&"1".to_string());
    print!(
        "  VRAM Total: {:.2} GB\n  VRAM Used: {:.2} GB\n",
        vram_total, vram_used
    );
    let (power_used, power_max) = gpu_power(&"1".to_string());
    print!(
        "  Power Used: {} Watts\n  Power Max: {} Watts\n",
        power_used, power_max
    );
    println!("  Temperature: {} °C", gpu_temp(&"1".to_string()));
    let (core_speed, mem_speed) = gpu_clock_speeds(&"1".to_string());
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

fn print_mem_from_config(flags: &HashMap<String, bool>) {
    let mem_group = ["mem_total", "mem_free", "mem_available", "mem_swap_info"];
    let mut mem_printed = false;

    for key in mem_group {
        if flags.contains_key(key) {
            if !mem_printed {
                println!("MEM");
                mem_printed = true;
            }
            match key {
                "mem_total" => println!("  Total: {:.1} GB", mem_total()),
                "mem_free" => println!("  Free: {:.1} GB", mem_free()),
                "mem_available" => println!("  Available: {:.1} GB", mem_available()),
                "mem_swap_info" => {
                    let (swap_total, swap_free) = mem_swap_info();
                    print!(
                        "  Swap Total: {:.1} GB\n  Swap Free: {:.1} GB\n",
                        swap_total, swap_free
                    );
                }
                _ => {}
            }
        }
    }
}

fn print_gpu_from_config(flags: &HashMap<String, bool>, gpu_ids: &Vec<String>) {
    let gpu_group = ["gpu_vram", "gpu_power", "gpu_temp", "gpu_clock_speeds"];
    for id in gpu_ids {
        let mut gpu_printed = false;
        for key in gpu_group {
            if flags.contains_key(key) {
                if !gpu_printed {
                    print!("GPU {} \n", id);
                    gpu_printed = true;
                }
                match key {
                    "gpu_vram" => {
                        let (vram_total, vram_used) = gpu_vram(id);
                        print!(
                            "  VRAM Total: {:.2} GB\n  VRAM Used: {:.2} GB\n",
                            vram_total, vram_used
                        );
                    }
                    "gpu_power" => {
                        let (power_used, power_max) = gpu_power(id);
                        print!(
                            "  Power Used: {} Watts\n  Power Max: {} Watts\n",
                            power_used, power_max
                        );
                    }
                    "gpu_temp" => println!("  Temperature: {} °C", gpu_temp(id)),
                    "gpu_clock_speeds" => {
                        let (core_speed, mem_speed) = gpu_clock_speeds(id);
                        print!(
                            "  Core Speed: {} MHz\n  Memory Speed: {} MHz\n",
                            core_speed, mem_speed
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}
