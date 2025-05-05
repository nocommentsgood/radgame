use godot::prelude::*;

#[derive(Default, Clone)]
pub struct PatrolComponent {
    east_target: Vector2,
    west_target: Vector2,
}

impl PatrolComponent {
    pub fn new(east_x: f32, east_y: f32, west_x: f32, west_y: f32) -> Self {
        Self {
            east_target: Vector2::new(east_x, east_y),
            west_target: Vector2::new(west_x, west_y),
        }
    }

    pub fn get_furthest_distance(&self, current_pos: Vector2) -> Vector2 {
        let left_distance = current_pos.distance_to(self.west_target);
        let right_distance = current_pos.distance_to(self.east_target);

        if left_distance >= right_distance {
            Vector2::LEFT
        } else {
            Vector2::RIGHT
        }
    }
}
