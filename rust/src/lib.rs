use godot::prelude::*;

pub mod godot_traits;
pub mod rust_classes;
pub mod utils;
pub use godot_traits::*;
pub use rust_classes::*;
pub use utils::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
