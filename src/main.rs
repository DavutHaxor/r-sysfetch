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

use std::{
    char,
    collections::HashMap,
    env,
    fmt::format,
    fs::{self, exists},
    io::{self, prelude::*},
    path::PathBuf,
    process::exit,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn main() {
    let config_path = get_config_path();
    create_config(&config_path);
    let Ok((flags, gpu_ids, logging_interval)) = read_config(&config_path) else {
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
            print_gpu(&gpu_ids[0]);
        }
        if arg_count.contains_key(&"a".to_string()) {
            print_cpu();
            print_mem();
            print_gpu(&gpu_ids[0]);
        }
        if arg_count.contains_key(&"t".to_string()) {
            print_cpu_from_config(&flags);
            print_mem_from_config(&flags);
            print_gpu_from_config(&flags, &gpu_ids);
        }
        if arg_count.contains_key(&"s".to_string()) {
            logging(logging_interval, &gpu_ids[0]);
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

fn print_gpu(gpu_id1: &String) {
    println!("GPU");
    let (vram_total, vram_used) = gpu_vram(gpu_id1);
    print!(
        "  VRAM Total: {:.2} GB\n  VRAM Used: {:.2} GB\n",
        vram_total, vram_used
    );
    let (power_used, power_max) = gpu_power(gpu_id1);
    print!(
        "  Power Used: {} Watts\n  Power Max: {} Watts\n",
        power_used, power_max
    );
    println!("  Temperature: {} °C", gpu_temp(gpu_id1));
    let (core_speed, mem_speed) = gpu_clock_speeds(gpu_id1);
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

logging_interval=60
"#;
        fs::write(&config_file, &config_content).ok();
    }
}

fn read_config(
    config_file: &PathBuf,
) -> Result<(HashMap<String, bool>, Vec<String>, u64), std::io::Error> {
    let content = fs::read_to_string(config_file)?;
    let mut gpu_ids = Vec::new();
    let mut flags = HashMap::new();
    let mut logging_interval: u64 = 60;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(stripped) = line.strip_prefix("gpu=") {
            gpu_ids.push(stripped.to_string());
            continue;
        }
        if let Some(stripped) = line.strip_prefix("logging_interval=") {
            logging_interval = stripped.parse().unwrap_or(60);
            continue;
        }
        flags.insert(line.to_string(), true);
    }

    Ok((flags, gpu_ids, logging_interval))
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

fn logging(logging_interval: u64, gpu_id1: &String) {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let filename = chrono::Local::now()
        .format("r-sysfetch_%H_%M_%S___%d_%m_%y.csv")
        .to_string();
    let path = PathBuf::from(home).join(filename);
    if !path.exists() {
        let header = "cpu_usage,cpu_freq,cpu_temp,mem_free,mem_available,swap_free,vram_used,power_used,gpu_temp,core_clock,mem_clock\n";
        fs::write(&path, header).ok();
    }
    loop {
        let (mem_swap_total, mem_swap_free) = mem_swap_info();
        let (vram_total, vram_used) = gpu_vram(gpu_id1);
        let (power_used, power_max) = gpu_power(gpu_id1);
        let (core_speed, mem_speed) = gpu_clock_speeds(gpu_id1);
        let row = format!(
            "{:.2},{:.1},{:.1},{:.1},{:.1},{:.1},{:.2},{},{},{},{}\n",
            cpu_usage(),
            cpu_freq(),
            cpu_temperature() / 1000.0,
            mem_free(),
            mem_available(),
            mem_swap_free,
            vram_used,
            power_used,
            gpu_temp(gpu_id1),
            core_speed,
            mem_speed
        );
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut f| f.write_all(row.as_bytes()))
            .ok();
        thread::sleep(Duration::from_secs(logging_interval));
    }
}
