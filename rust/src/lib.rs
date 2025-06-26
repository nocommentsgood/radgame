use godot::prelude::*;

mod traits;
pub mod utils;
mod classes {
    pub mod characters;
    pub mod components;
    pub mod enemies;
    pub mod scenes;
}
mod components {
    pub mod managers;
    pub mod state_machines;
}

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
