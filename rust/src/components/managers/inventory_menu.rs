use godot::{classes::Control, prelude::*};

#[derive(GodotClass)]
#[class(base=Control, init)]
struct InventoryMenu {
    #[init[node = "Control"]]
    rosary_bead_menu: OnReady<Gd<Control>>,
    base: Base<Control>,
}
