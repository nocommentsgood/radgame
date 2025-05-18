use godot::prelude::*;
pub use rust_classes::*;
struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
