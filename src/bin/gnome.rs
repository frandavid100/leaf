//! Leaf GTK executable; the frontend is organized under src/gnome/.
#[path = "../gnome/mod.rs"]
mod gnome;

fn main(){gnome::run();}
