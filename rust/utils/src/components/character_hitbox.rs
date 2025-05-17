use godot::prelude::GodotClass;

/// A wrapper type for the game's main character class Hitbox.
#[derive(GodotClass)]
#[class(init, base = Area2D)]
pub struct CharacterHitbox;
