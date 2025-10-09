#![allow(unused)]
use std::{char, collections::HashMap, env, fmt::{format, Write}, fs, thread, time};

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
    // CPU section
    println!("CPU");    
    println!("  {}", cpu_model_name());
    println!("  Usage: {:.2}%", cpu_usage());
    println!("  Frequency: {:.1} GHz", cpu_freq());
    println!("  Temperature: {:.1} °C", cpu_temperature() / 1000.0);
    println!("  Cores: {}", cpu_cores());
    println!("  Threads: {}", cpu_threads());

    // MEM section
    println!("MEM");
    println!("  Total: {:.1} GB", mem_total());
    println!("  Free: {:.1} GB", mem_free());
    println!("  Available: {:.1} GB", mem_available());
    let (swap_total, swap_free) = mem_swap_info();
    print!("  Swap Total: {:.1} GB\n  Swap Free: {:.1} GB\n", swap_total, swap_free);
    
    // GPU section
    println!("GPU");
    let (vram_total, vram_used) = gpu_vram('1');
    print!("  VRAM Total: {:.2} GB\n  VRAM Used: {:.2} GB\n", vram_total, vram_used);
    let (power_used, power_max) = gpu_power('1');
    print!("  Power Used: {} Watts\n  Power Max: {} Watts\n", power_used, power_max);
    println!("  Temperature: {} °C", gpu_temp('1'));
    let (core_speed, mem_speed) = gpu_clocks('1');
    print!("  Core Speed: {} MHz\n  Memory Speed: {} MHz\n", core_speed, mem_speed);


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
fn cpu_temperature() -> f64 {
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
            }
            else if name == "acpitz" && acpitz_temp.is_none() {
                if let Ok(temp) = fs::read_to_string(&temp_path) {
                    acpitz_temp = temp.trim().parse::<f64>().ok();
                }
            }
        }
    }
    acpitz_temp.unwrap_or(0.0)
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
fn cpu_freq() -> f64 {
    let threads = cpu_threads()
        .parse::<usize>()
        .unwrap_or(0);
    let speeds: Vec<f64> = (0..threads)
        .filter_map(|i| {
            let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq", i);
            fs::read_to_string(&path).ok()
                .and_then(|freq| freq.trim().parse::<u32>().ok())
                .map(|freq| freq as f64 / 1_000_000.0)

        })
        .collect();
    if !speeds.is_empty() {(speeds.iter().sum::<f64>() / speeds.len() as f64)} 
    else {0.0}
}
// Memory Section
// t
fn mem_total() -> f64 {
    match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("MemTotal:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|memtotal| memtotal.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }
}
// f
fn mem_free() -> f64 {
    match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("MemFree:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|memtotal| memtotal.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }
}
// a
fn mem_available() -> f64 {
    match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("MemAvailable:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|memtotal| memtotal.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }   
}
// c
fn mem_cached() -> f64 {
    match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("Cached:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|memtotal| memtotal.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }   
}
// s
fn mem_swap_info() -> (f64, f64) {
    let swap_total = match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("SwapTotal:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|memtotal| memtotal.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    };

    let swap_free = match fs::read_to_string("/proc/meminfo") {
        Ok(content) => content
            .lines()
            .find(|line| line.starts_with("SwapFree:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|memtotal| memtotal.parse::<f64>().ok())
            .map(|kb| kb / 1_000_000.0)
            .unwrap_or(0.0),
        Err(_) => 0.0,
    };

    (swap_total, swap_free)
}



// GPU Section 
// v
fn gpu_vram(gpu_id: char) -> (f64, f64) {
    let vram_total_path = format!("/sys/class/drm/card{}/device/mem_info_vram_total", gpu_id);
    let vram_total = fs::read_to_string(&vram_total_path).ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|bytes| bytes as f64 / 1_000_000_000.0)
        .unwrap_or(0.0);
    let vram_used_path = format!("/sys/class/drm/card{}/device/mem_info_vram_usage", gpu_id);
    let vram_used = fs::read_to_string(&vram_used_path).ok() 
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|bytes| bytes as f64 / 1_000_000_000.0)
        .unwrap_or(0.0);
    (vram_total, vram_used)
}
// u
fn gpu_usage(gpu_id: char) -> u8 {
    let path = format!("/sys/class/drm/card{}/device/gpu_busy_percent", gpu_id);
    match fs::read_to_string(&path) {
        Ok(content) => content
            .trim()
            .parse::<u8>()
            .unwrap_or(0),
        Err(_) => 0,
    }
}
// p
fn gpu_power(gpu_id: char) -> (u64, u64) {
    let base_path = format!("/sys/class/drm/card{}/device/hwmon", gpu_id);
    let power_used = fs::read_dir(&base_path)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                let power_file = path.join("power1_average");
                fs::read_to_string(power_file).ok()
                    .and_then(|p| p.trim().parse::<u64>().ok())
                    .map(|microwatts| microwatts as u64 / 1_000_000 )
            })
        })
        .unwrap_or(0);

    let power_max = fs::read_dir(&base_path)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                let power_file = path.join("power1_cap_max");
                fs::read_to_string(power_file).ok()
                    .and_then(|p| p.trim().parse::<u64>().ok())
                    .map(|microwatts| microwatts as u64 / 1_000_000 )
            })
        })
        .unwrap_or(0);

    (power_used, power_max)
}
// t
fn gpu_temp(gpu_id: char) -> u64 {
    let base_path = format!("/sys/class/drm/card{}/device/hwmon", gpu_id);
    fs::read_dir(&base_path)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                let temp_file = path.join("temp1_input");
                fs::read_to_string(temp_file).ok()
                    .and_then(|temp| temp.trim().parse::<u64>().ok())
                    .map(|celsius| celsius as u64 / 1000)
            })
        })
        .unwrap_or(0)
}
// c
fn gpu_clocks(gpu_id: char) -> (u64, u64) {
    let base_path = format!("/sys/class/drm/card{}/device/hwmon", gpu_id);
    let core_speed = fs::read_dir(&base_path)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                let power_file = path.join("freq1_input");
                fs::read_to_string(power_file).ok()
                    .and_then(|p| p.trim().parse::<u64>().ok())
                    .map(|hertz| hertz as u64 / 1_000_000 )
            })
        })
        .unwrap_or(0);

    let mem_speed = fs::read_dir(&base_path)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                let power_file = path.join("freq2_input");
                fs::read_to_string(power_file).ok()
                    .and_then(|p| p.trim().parse::<u64>().ok())
                    .map(|hertz| hertz as u64 / 1_000_000 )
            })
        })
        .unwrap_or(0);

    (core_speed, mem_speed) 
}
















