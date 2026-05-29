# r-sysfetch

Fast momentarily info about system resources.

## Arguments:  
-- a => Prints all the information about CPU, MEM and GPU.  
-- t => Prints information respect to config file.  
-- s => Data logging mode  
-- cpu gpu mem => Each argument prints information about their corresponding element.

## Configuration
Configuration is handled in $XDG_CONFIG_HOME/r-sysfetch.conf

Config Example:
```
#r-sysfetch configuration file!
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

gpu=1
gpu=2

logging_interval=60
```

You can comment lines with # and program won't print those when dry run or -- t.

For GPUs you can specify numbers that are in /sys/class/drm/cardX, and then write as "gpu=X" to configuration file.  

Data logging mode saves all the data in a csv file.   
File format that data is saved to is %HH_%MM_%SS___%DD_%MM_%YY.csv


Example output for GPU:

```
❯ r-sysfetch gpu
GPU
  VRAM Total: 4.28 GB
  VRAM Used: 0.00 GB
  Power Used: 16 Watts
  Power Max: 65 Watts
  Temperature: 58 °C
  Core Speed: 65 MHz
  Memory Speed: 875 MHz
````

