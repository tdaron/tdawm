run:
    cargo build
    LOG_LEVEL=info startx ./xinitrc -- /usr/bin/Xephyr :100 -ac -screen 1028x578 -host-cursor
full:
    cargo build
    startx ./xinitrc -- /usr/bin/Xephyr :100 -ac -screen 1920x1080 -host-cursor
