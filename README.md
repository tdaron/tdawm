# TDAWM

Hey ! Welcome to this repo. The goal of this project is to end up as my own window manager, replacing tdawm.

This is still a POC atm, but here is the directions i want this project to go:

- Minimalist, I really like unix philosophy. This WM should just do what a WM do and nothing else.

This includes using sxhkd for non wm-included keybindings. (i am still hesitating on building my own included status bar like dwm do)

- Easy to read codebase (not atm ahah)

## How to build

As this project is built in Rust, you must have rust toolchain installed.

To build, `cargo build` is enough. Then, inside the justfile you can find commands to run it through Xephyr.

If just is installed, you can simply do `just run`.

To use it as your "native" window manager, simply put `exec /path/to/tdawmbin` into your .xinitrc :)
