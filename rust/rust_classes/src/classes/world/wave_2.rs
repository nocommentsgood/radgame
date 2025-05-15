use godot::prelude::*;

use crate::classes::enemies::test_enemy::TestEnemy;

#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct Wave2 {
    kill_count: u8,
    base: Base<Node>,
}

#[godot_api]
impl INode for Wave2 {
    fn ready(&mut self) {
        let enemies = self.base().get_children();

        for enemy in enemies.iter_shared() {
            if let Ok(mut e) = enemy.try_cast::<TestEnemy>() {
                let mut this = self.to_gd();

                e.signals()
                    .test_enemy_died()
                    .connect(move || this.bind_mut().on_enemy_killed());
            }
        }
    }
}

#[godot_api]
impl Wave2 {
    #[signal]
    fn last_enemy_killed();

    fn on_enemy_killed(&mut self) {
        self.kill_count += 1;
        if self.kill_count == 2 {
            self.signals().last_enemy_killed().emit();
        }
    }
}
