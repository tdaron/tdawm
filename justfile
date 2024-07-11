run:
    cargo build
    LOG_LEVEL=info XEPHYR=true startx ./xinitrc -- /usr/bin/Xephyr :100 -ac -screen 1233x694 -host-cursor
full:
    cargo build
    startx ./xinitrc -- /usr/bin/Xephyr :100 -ac -screen 1920x1080 -host-cursor
mult:
    LOG_LEVEL=trace startx ./xinitrc -- /usr/bin/Xephyr +xinerama :101 -ac -screen 1028x578 -screen 800x600 -host-cursor
