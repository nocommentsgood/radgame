use godot::prelude::*;

mod components {
    mod characters;
    mod enemies;
}
mod traits;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
