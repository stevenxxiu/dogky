#!/bin/sh

args=" --pid "$(pgrep '\bdogky$' | tr '\n' ' \-\-pid ')

# Wait until the window appears
window_id=$(eval "xdotool search --all ${args} --sync --name --onlyvisible ^dogky$")

width=$(xwininfo -id $window_id | rg --only-matching --replace='$1' '  Width: (\d+)')
screen_width=$(xrandr --current | rg --only-matching --replace='$1' 'current (\d+)')
x=$(( screen_width - width ))

wmctrl -i -r $window_id -e "0,${x},0,-1,-1"
target/debug/dogky
