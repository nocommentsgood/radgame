use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
struct LevelManager {
    kill_count: u8,
    enemy_scene: OnReady<Gd<PackedScene>>,
    wave_2: OnReady<Gd<PackedScene>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for LevelManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            kill_count: 0,
            enemy_scene: OnReady::from_loaded("res://main_character.tscn"),
            wave_2: OnReady::from_loaded("res://wave_2.tscn"),
            base,
        }
    }
    fn ready(&mut self) {
        self.connect_to_signals();
    }
}

impl LevelManager {
    fn connect_to_signals(&mut self) {
        // let mut enemy = self
        //     .base()
        //     .get_node_as::<crate::classes::enemies::test_enemy::TestEnemy>(
        //         "/root/Node2D/WorldEnvironment/TestEnemy",
        //     );
        let mut enemy = self
            .base()
            .get_node_as::<crate::classes::enemies::test_enemy::TestEnemy>(
                "../TileMapLayer/TestEnemy",
            );

        let mut this = self.to_gd();

        enemy
            .signals()
            .test_enemy_died()
            .connect(move || this.bind_mut().on_enemy_died());
    }

    fn increment_kills(&mut self) {
        self.kill_count += 1;

        if self.kill_count < 3 {
            let wave_2 = self
                .wave_2
                .instantiate_as::<crate::classes::scenes::wave_2::Wave2>();
            self.base_mut().add_child(&wave_2);
        }
    }

    fn on_enemy_died(&mut self) {
        self.increment_kills();

        if self.kill_count < 4 {
            let mut enemy = self
                .enemy_scene
                .instantiate_as::<crate::classes::enemies::test_enemy::TestEnemy>();
            let varg = enemy.to_variant();
            self.base_mut().call_deferred("add_child", &[varg]);

            let spawn_position = Vector2::new(400.0, 230.0);
            enemy.set_position(spawn_position);

            let mut this = self.to_gd();
            enemy
                .signals()
                .test_enemy_died()
                .connect(move || this.bind_mut().on_enemy_died());
        }
    }
}
