run:
    cargo build
    LOG_LEVEL=info XEPHYR=true startx ./xinitrc -- /usr/bin/Xephyr :100 -ac -screen 1233x694 -host-cursor
