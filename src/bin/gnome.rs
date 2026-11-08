//! Leaf GTK executable; the frontend is organized under src/gnome/.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[path = "../gnome/mod.rs"]
mod gnome;

fn main() {
    gnome::run();
}
