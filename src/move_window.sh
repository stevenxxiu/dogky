#!/bin/sh

monitor_width=$1
monitor_height=$2
window_width=$3
window_height=$4

# Wait until the window appears
window_id=$(xdotool search --all --pid=$PPID --sync --name --onlyvisible "^dogky$")

wmctrl -i -r $window_id -b add,sticky
wmctrl -i -r $window_id -b add,skip_taskbar
wmctrl -i -r $window_id -b add,skip_pager
wmctrl -i -r $window_id -b add,below

x=$(( monitor_width - window_width ))
wmctrl -i -r $window_id -e "0,${x},0,-1,-1"
