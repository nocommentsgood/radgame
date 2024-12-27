use godot::prelude::*;

mod classes {
    pub mod characters;
    mod enemies;
}

mod components {
    mod managers;
    pub mod state_machines;
}
mod traits;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
