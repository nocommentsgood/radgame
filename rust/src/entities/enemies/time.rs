use godot::{
    classes::{Node, Timer},
    obj::Gd,
};

#[derive(Debug)]
pub enum EnemyTimers {
    Attack,
    Patrol,
    Idle,
    AttackChain,
    AttackAnimation,
}

#[derive(Clone)]
pub struct Timers {
    pub attack: Gd<Timer>,
    pub patrol: Gd<Timer>,
    pub idle: Gd<Timer>,
    pub attack_chain: Gd<Timer>,
    pub attack_anim: Gd<Timer>,
}

impl Timers {
    pub fn new(node: &Gd<Node>) -> Self {
        Self {
            attack: node.get_node_as::<Timer>("Attack"),
            patrol: node.get_node_as::<Timer>("Patrol"),
            idle: node.get_node_as::<Timer>("Idle"),
            attack_chain: node.get_node_as::<Timer>("AttackChain"),
            attack_anim: node.get_node_as::<Timer>("AttackAnim"),
        }
    }
}
