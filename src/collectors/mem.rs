#![allow(unused)]
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


pub fn mem_total() -> f64 {
    match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("MemTotal:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|v| v.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }
}

pub fn mem_free() -> f64 {
    match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("MemFree:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|v| v.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }
}

pub fn mem_available() -> f64 {
    match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("MemAvailable:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|v| v.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }
}

pub fn mem_cached() -> f64 {
    match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("Cached:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|v| v.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }
}

pub fn mem_swap_info() -> (f64, f64) {
    let swap_total = match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("SwapTotal:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|v| v.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    };
    let swap_free = match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("SwapFree:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|v| v.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    };
    (swap_total, swap_free)
}



