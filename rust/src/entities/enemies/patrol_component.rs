use godot::builtin::Vector2;

/// Used for setting the maximum distance an enemy can move in its patrol state.
#[derive(Default)]
pub struct PatrolComp {
    /// The furthest distance the entity should move to the left in its patrol state.
    /// Note that only the x-axis is considered.
    pub left_target: Vector2,

    /// The furthest distance the entity should move to the right in its patrol state.
    /// Note that only the x-axis is considered.
    pub right_target: Vector2,
}

impl PatrolComp {
    pub fn get_furthest_distance(&self, current_pos: Vector2) -> Vector2 {
        let left_dist = (self.left_target.x - current_pos.x).abs();
        let right_dist = (self.right_target.x - current_pos.x).abs();

        if left_dist > right_dist {
            Vector2::LEFT
        } else {
            Vector2::RIGHT
        }
    }
}
