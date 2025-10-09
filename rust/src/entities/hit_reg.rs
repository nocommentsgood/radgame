use godot::{
    classes::{Area2D, IArea2D, RayCast2D},
    meta::ToGodot,
    obj::{Base, Gd, WithBaseField},
    prelude::{GodotClass, godot_api},
};

use crate::entities::damage::{AttackData, Damageable};

// TODO: Add resistances here.
#[derive(GodotClass)]
#[class(init, base = Area2D)]
pub struct Hitbox {
    /// The Damageable entity which owns the Hitbox.
    pub damageable_parent: Option<Box<dyn Damageable>>,
    base: Base<Area2D>,
}

#[godot_api]
impl Hitbox {
    #[signal]
    fn dummy();
}

#[derive(GodotClass)]
#[class(init, base=Area2D)]
pub struct Hurtbox {
    pub data: Option<AttackData>,
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for Hurtbox {
    fn ready(&mut self) {
        self.base_mut().set_deferred("disabled", &true.to_variant());

        let mut this = self.to_gd();
        self.base_mut()
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_hit(area));
    }
}

#[godot_api]
impl Hurtbox {
    fn on_hit(&mut self, area: Gd<Area2D>) {
        let mut hitbox = area.cast::<Hitbox>();
        hitbox
            .bind_mut()
            .damageable_parent
            .as_mut()
            .unwrap()
            .handle_attack(self.data.clone().unwrap());
    }
}

#[derive(Clone, Debug)]
pub struct HitReg {
    pub hitbox: Gd<Hitbox>,
    pub hurtbox: Gd<Hurtbox>,
}

impl HitReg {
    pub fn new(hitbox: Gd<Hitbox>, hurtbox: Gd<Hurtbox>) -> Self {
        Self { hitbox, hurtbox }
    }

        hitbox: Gd<Hitbox>,
        hurtbox: Gd<Hurtbox>,
        left_wall_cast: Option<Gd<RayCast2D>>,
        right_wall_cast: Option<Gd<RayCast2D>>,
    ) -> Self {
        Self {
            hitbox,
            hurtbox,
            left_wall_cast,
            right_wall_cast,
        }
    }


    }
}
