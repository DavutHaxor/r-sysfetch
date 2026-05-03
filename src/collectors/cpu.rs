#![allow(unused)]
use std::{
    fs::{self},
    thread, time,
};

pub fn cpu_model_name() -> String {
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

pub fn cpu_cores() -> String {
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

pub fn cpu_threads() -> String {
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

pub fn cpu_temperature() -> f64 {
    let mut acpitz_temp = None;
    for i in 0..10 {
        let path = format!("/sys/class/hwmon/hwmon{}/name", i);
        let temp_path = format!("/sys/class/hwmon/hwmon{}/temp1_input", i);
        if let Ok(name) = fs::read_to_string(&path) {
            let name = name.trim();
            if name == "x86_pkg_temp" {
                if let Ok(temp) = fs::read_to_string(&temp_path) {
                    return temp.trim().parse::<f64>().unwrap_or(0.0);
                }
            } else if name == "acpitz" && acpitz_temp.is_none() {
                if let Ok(temp) = fs::read_to_string(&temp_path) {
                    acpitz_temp = temp.trim().parse::<f64>().ok();
                }
            }
        }
    }
    acpitz_temp.unwrap_or(0.0)
}

pub fn cpu_usage() -> f64 {
    pub fn get_values() -> Option<(u64, u64)> {
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
                } else {
                    None
                }
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
    } else {
        0.0
    }
}

pub fn cpu_freq() -> f64 {
    let threads = cpu_threads().parse::<usize>().unwrap_or(0);
    let speeds: Vec<f64> = (0..threads)
        .filter_map(|i| {
            let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq", i);
            fs::read_to_string(&path)
                .ok()
                .and_then(|freq| freq.trim().parse::<u32>().ok())
                .map(|freq| freq as f64 / 1_000_000.0)
        })
        .collect();
    if !speeds.is_empty() {
        speeds.iter().sum::<f64>() / speeds.len() as f64
    } else {
        0.0
    }
}
