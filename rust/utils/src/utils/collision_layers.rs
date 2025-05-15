pub enum CollisionLayers {
    LayerOne,
    FloorWall,
    PlayerHitbox,
    EnemyHitbox,
    PlayerHurtbox,
    EnemyHurtbox,
    PlatformLedges,
    PlayerPhysics,
    Items,
}

impl From<CollisionLayers> for i32 {
    fn from(value: CollisionLayers) -> Self {
        match value {
            CollisionLayers::LayerOne => 1,
            CollisionLayers::FloorWall => 2,
            CollisionLayers::PlayerHitbox => 3,
            CollisionLayers::EnemyHitbox => 4,
            CollisionLayers::PlayerHurtbox => 5,
            CollisionLayers::EnemyHurtbox => 6,
            CollisionLayers::PlatformLedges => 7,
            CollisionLayers::PlayerPhysics => 8,
            CollisionLayers::Items => 9,
        }
    }
}
