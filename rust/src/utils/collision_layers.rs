#[derive(Debug)]
pub enum CollisionLayers {
    LayerOne = 1,
    FloorWall = 2,
    PlayerHitbox = 3,
    EnemyHitbox = 4,
    PlayerHurtbox = 5,
    EnemyHurtbox = 6,
    PlatformLedges = 7,
    PlayerPhysics = 8,
    Items = 9,
}

#[cfg(test)]
mod test {
    use crate::utils::collision_layers::CollisionLayers;

    #[test]
    fn test_collision_layer_as_i32() {
        assert_eq!(CollisionLayers::LayerOne as i32, 1);
        assert_eq!(CollisionLayers::FloorWall as i32, 2);
        assert_eq!(CollisionLayers::PlayerHitbox as i32, 3);
        assert_eq!(CollisionLayers::EnemyHitbox as i32, 4);
        assert_eq!(CollisionLayers::PlayerHurtbox as i32, 5);
        assert_eq!(CollisionLayers::EnemyHurtbox as i32, 6);
        assert_eq!(CollisionLayers::PlatformLedges as i32, 7);
        assert_eq!(CollisionLayers::PlayerPhysics as i32, 8);
        assert_eq!(CollisionLayers::Items as i32, 9);
    }
}
