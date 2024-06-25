output=`(hyprctl monitors)`
laptopMonitor="eDP-1"
if [[ "$output" == *"$laptopMonitor"* ]]; then
    hyprctl keyword monitor eDP-1, disable
    exit 1
else
    hyprctl keyword monitor "eDP-1, 1920x1080@60,auto,1"
    exit 0
fi
