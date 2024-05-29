run:
    cargo build
    startx ./xinitrc -- /usr/bin/Xephyr :100 -ac -screen 960x540 -host-cursor
