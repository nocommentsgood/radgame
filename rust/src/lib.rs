use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(init,base = Object)]
struct Dummy;

#[godot_api]
impl Dummy {
    #[func]
    fn p() {}
}
