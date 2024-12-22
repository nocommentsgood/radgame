use godot::prelude::*;

mod traits;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
