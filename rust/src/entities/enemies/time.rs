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
    /// Provides automatic collection of `SceneTree` nodes at the expected path.
    pub fn default_new(node: &Gd<Node>) -> Self {
        Self {
            attack: node.get_node_as::<Timer>("Attack"),
            patrol: node.get_node_as::<Timer>("Patrol"),
            idle: node.get_node_as::<Timer>("Idle"),
            attack_chain: node.get_node_as::<Timer>("AttackChain"),
            attack_anim: node.get_node_as::<Timer>("AttackAnim"),
        }
    }

    /// Connects the given callbacks.
    /// Expected callbacks:
    /// - attack timeout
    /// - patrol timeout
    /// - idle timeout
    /// - attack chain timeout
    /// - attack animation timeout
    pub fn connect_signals<A, B, C, D, E>(
        &mut self,
        on_attack_timeout: A,
        on_patrol_timeout: B,
        on_idle_timeout: C,
        on_attack_chain_timeout: D,
        on_attack_anim_timeout: E,
    ) where
        A: FnMut() + 'static,
        B: FnMut() + 'static,
        C: FnMut() + 'static,
        D: FnMut() + 'static,
        E: FnMut() + 'static,
    {
        self.attack.signals().timeout().connect(on_attack_timeout);
        self.patrol.signals().timeout().connect(on_patrol_timeout);
        self.idle.signals().timeout().connect(on_idle_timeout);
        self.attack_chain
            .signals()
            .timeout()
            .connect(on_attack_chain_timeout);
        self.attack_anim
            .signals()
            .timeout()
            .connect(on_attack_anim_timeout);
    }
}
