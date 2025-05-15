use godot::prelude::FromGodot;
use godot::{
    classes::{Area2D, Node2D},
    obj::{DynGd, Gd, Inherits, WithBaseField},
};

pub trait HasEnemyHitbox
where
    Self: Inherits<godot::classes::Node2D> + WithBaseField<Base: Inherits<godot::classes::Node>>,
{
    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        let damaging = DynGd::<Area2D, dyn super::damaging::Damaging>::from_godot(area);
        let target = self.to_gd().upcast::<Node2D>();
        let _guard = self.base_mut();
        let damageable = DynGd::<Node2D, dyn super::damageable::Damageable>::from_godot(target);
        damaging.dyn_bind().do_damage(damageable);
    }

    fn connect_hitbox_signal(&mut self) {
        let mut hitbox = self
            .base()
            .upcast_ref()
            .get_node_as::<Area2D>("EnemySensors/Hitbox");
        let mut this = self.to_gd();

        hitbox
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_area_entered_hitbox(area));
    }
}
