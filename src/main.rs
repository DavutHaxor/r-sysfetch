#![allow(unused)]
use std::{char, collections::HashMap, env, fmt::Write, fs, thread, time};

fn main() {
    
    let arguments: Vec<String> = env::args().collect();

    if arguments.len() >= 2 {
        let dev1 = &arguments[0];
        let args1: Vec<char> = arguments[1].chars().collect();
    } 
    if arguments.len() >= 4 {
        let dev2 = &arguments[2];
        let args2: Vec<char> = arguments[3].chars().collect();
    }
    if arguments.len() >= 6 {
        let dev3 = &arguments[4];
        let args3: Vec<char> = arguments[5].chars().collect();
    }

    println!("{:.2}", cpu_usage());
}

// CPU Section
// m
fn cpu_model_name() -> String {
    match fs::read_to_string("/proc/cpuinfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("model name"))
            .and_then(|line| line.split(": ").nth(1))
            .map(|model| model.trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
        Err(_) => "Unknown".to_string(),
    }
}
// c
fn cpu_cores() -> String {
    match fs::read_to_string("/proc/cpuinfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("cpu cores"))
            .and_then(|line| line.split(": ").nth(1))
            .map(|cores| cores.trim().to_string())
            .unwrap_or_else(|| "More than 0".to_string()),
        Err(_) => "More than 0".to_string(),
    }
}
// t
fn cpu_threads() -> String {
    match fs::read_to_string("/proc/cpuinfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("siblings"))
            .and_then(|line| line.split(": ").nth(1))
            .map(|siblings| siblings.trim().to_string())
            .unwrap_or_else(|| "More than 0".to_string()),
        Err(_) => "More than 0".to_string(),
    }
}
// h for heat 
fn cpu_temperature() -> String {
    let mut acpitz_temp = None;
    for i in 0..10 {
        let path = format!("/sys/class/hwmon/hwmon{}/name", i);
        let temp_path = format!("/sys/class/hwmon/hwmon{}/temp1_input", i);
        if let Ok(name) = fs::read_to_string(&path) {
            let name = name.trim();
            if name == "x86_pkg_temp" {
                if let Ok(temp) = fs::read_to_string(&temp_path) {
                    return temp.trim().to_string();
                }
            }
            else if name == "acpitz" && acpitz_temp.is_none() {
                if let Ok(temp) = fs::read_to_string(&temp_path) {
                    acpitz_temp = Some(temp.trim().to_string());
                }
            }
        }
    }
    acpitz_temp.unwrap_or_else(|| "Unknown".to_string())
}
// u
fn cpu_usage() -> f64 {
    fn get_values() -> Option<(u64, u64)> {
        fs::read_to_string("/proc/stat").ok().and_then(|content| {
            content.lines().next().and_then(|line| {
                let values: Vec<u64> = line
                    .split_whitespace()
                    .skip(1)
                    .filter_map(|s| s.parse().ok())
                    .collect();

                if values.len() >= 5 {
                    let total: u64 = values.iter().sum();
                    let idle = values[3] + values[4];
                    Some((total, idle))
                } else { None }
            })
        })
    }
    
    let (total1, idle1) = get_values().unwrap_or((0, 0));
    thread::sleep(time::Duration::from_millis(100));
    let (total2, idle2) = get_values().unwrap_or((0, 0));
    
    let total_diff = total2 - total1;
    let idle_diff = idle2 - idle1;
    if total_diff > 0 {
        (1.0 - (idle_diff as f64 / total_diff as f64)) * 100.0
    } else { 0.0 }
}
// s for speed
fn cpu_freq() {
    
}
// Memory Section
// t
fn mem_total() {
    
}
// f
fn mem_free() {
    
}
// a
fn mem_available() {
    
}
// c
fn mem_cached() {
    
}
// s
fn mem_swap_info() {
    
}
// GPU Section
// v
fn gpu_vram() {
    
}
// u
fn gpu_usage() {
    
}
// p
fn gpu_power() {
    
}
// t
fn gpu_temp() {
    
}
// c
fn gpu_clocks() {
    
}
// n
fn gpu_name() {
    
}


















