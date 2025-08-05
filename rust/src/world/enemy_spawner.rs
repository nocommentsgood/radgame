// use godot::prelude::*;
//
// #[derive(GodotClass)]
// #[class(init, base = Node2D)]
// pub struct EnemySpawner {
//     #[init(load = "uid://bcae4wnfye0do")]
//     enemy_scene: OnReady<Gd<PackedScene>>,
//
//     TODO: Not sure how to satisfy traits...
//     Attempted to work around not being able to have generic structs by using something like:
//     d_type: DynGd<SomeEnemyThat impls EnemyStatExt trait.
//
//     d_type: DynGd<
//         Node2D,
//         dyn EnemyEntityStateMachineExt<
//                 Memory = MemManual,
//                 DynMemory = MemManual,
//                 Exportable = MemManual,
//             >,
//     >,
//     base: Base<Node2D>,
// }
//
// #[godot_api]
// impl EnemySpawner {
//     pub fn spawn(&mut self) -> Gd<ProjectileEnemy> {
//         let mut enemy = self.enemy_scene.instantiate_as::<ProjectileEnemy>();
//         enemy.set_position(self.base().get_position());
//         enemy
//     }
// }
