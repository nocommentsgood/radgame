use godot::prelude::*;
extern crate godot;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
