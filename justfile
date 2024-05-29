run:
    cargo build
    startx ./xinitrc -- /usr/bin/Xephyr :100 -ac -screen 960x540 -host-cursor
full:
    cargo build
    startx ./xinitrc -- /usr/bin/Xephyr :100 -ac -screen 1920x1080 -host-cursor
