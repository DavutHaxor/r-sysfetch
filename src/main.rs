#![allow(unused)]
use std::{char, env, fmt::Write};

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

}

// CPU Section
// m
fn cpu_model_name() {

}
// c
fn cpu_cores() {

}
// t
fn cpu_threads() {
    
}
// h for heat 
fn cpu_temperature() {

}
// u
fn cpu_usage() {

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


















