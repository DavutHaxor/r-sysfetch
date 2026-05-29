# r-sysfetch

Fast momentarily info about system resources.



Arguments:
-- a => Prints all the information about CPU, MEM and GPU.

-- t => Prints information respect to config file.

-- s => Data logging mode

-- cpu gpu mem => Each argument prints information about their corresponding element.


Configuration is handled in $XDG_CONFIG_HOME/r-sysfetch.conf

You can comment lines with # and program won't print those when run with -- t.

For GPUs you can specify numbers that are in /sys/class/drm/cardX, and then write as "gpu=X" to configuration file.


Data logging mode saves all the data in a csv file. The interval is set in configuration file as seconds.

File format that data is saved to is %HH_%MM_%SS___%DD_%MM_%YY.csv


