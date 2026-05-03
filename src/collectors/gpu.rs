#![allow(unused)]
use std::fs::{self};

pub fn gpu_vram(gpu_id: &String) -> (f64, f64) {
    let vram_total = fs::read_to_string(format!(
        "/sys/class/drm/card{}/device/mem_info_vram_total",
        gpu_id
    ))
    .ok()
    .and_then(|s| s.trim().parse::<u64>().ok())
    .map(|b| b as f64 / 1_000_000_000.0)
    .unwrap_or(0.0);

    let vram_used = fs::read_to_string(format!(
        "/sys/class/drm/card{}/device/mem_info_vram_usage",
        gpu_id
    ))
    .ok()
    .and_then(|s| s.trim().parse::<u64>().ok())
    .map(|b| b as f64 / 1_000_000_000.0)
    .unwrap_or(0.0);

    (vram_total, vram_used)
}

pub fn gpu_usage(gpu_id: &String) -> u8 {
    let path = format!("/sys/class/drm/card{}/device/gpu_busy_percent", gpu_id);
    match fs::read_to_string(&path) {
        Ok(content) => content.trim().parse::<u8>().unwrap_or(0),
        Err(_) => 0,
    }
}

pub fn gpu_power(gpu_id: &String) -> (u64, u64) {
    let base_path = format!("/sys/class/drm/card{}/device/hwmon", gpu_id);
    let power_used = fs::read_dir(&base_path)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                fs::read_to_string(path.join("power1_average"))
                    .ok()
                    .and_then(|p| p.trim().parse::<u64>().ok())
                    .map(|uw| uw / 1_000_000)
            })
        })
        .unwrap_or(0);

    let power_max = fs::read_dir(&base_path)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                fs::read_to_string(path.join("power1_cap_max"))
                    .ok()
                    .and_then(|p| p.trim().parse::<u64>().ok())
                    .map(|uw| uw / 1_000_000)
            })
        })
        .unwrap_or(0);

    (power_used, power_max)
}

pub fn gpu_temp(gpu_id: &String) -> u64 {
    let base_path = format!("/sys/class/drm/card{}/device/hwmon", gpu_id);
    fs::read_dir(&base_path)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                fs::read_to_string(path.join("temp1_input"))
                    .ok()
                    .and_then(|t| t.trim().parse::<u64>().ok())
                    .map(|c| c / 1000)
            })
        })
        .unwrap_or(0)
}

pub fn gpu_clock_speeds(gpu_id: &String) -> (u64, u64) {
    let base_path = format!("/sys/class/drm/card{}/device/hwmon", gpu_id);
    let core_speed = fs::read_dir(&base_path)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                fs::read_to_string(path.join("freq1_input"))
                    .ok()
                    .and_then(|p| p.trim().parse::<u64>().ok())
                    .map(|hz| hz / 1_000_000)
            })
        })
        .unwrap_or(0);

    let mem_speed = fs::read_dir(&base_path)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                fs::read_to_string(path.join("freq2_input"))
                    .ok()
                    .and_then(|p| p.trim().parse::<u64>().ok())
                    .map(|hz| hz / 1_000_000)
            })
        })
        .unwrap_or(0);

    (core_speed, mem_speed)
}
