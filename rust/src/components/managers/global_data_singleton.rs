use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base = Object)]
pub struct GlobalData {
    pub paths: PathData,
    base: Base<Object>,
}

#[derive(Default)]
pub struct PathData {
    pub player: Option<String>,
    pub map: Option<String>,
}
