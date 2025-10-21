use godot::{
    classes::{Node, Timer},
    obj::Gd,
};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum PlayerTimer {
    WallJumpLimit,
    AttackChain,
    DodgeAnimation,
    JumpingCooldown,
    AttackAnimation,
    Attack2Animation,
    HealingAnimation,
    HealingCooldown,
    HurtAnimation,
    ParryAnimation,
    Parry,
    PerfectParry,
    Coyote,
    DodgeCooldown,
    JumpTimeLimit,
}

pub struct PlayerTimers {
    wall_jump: Gd<Timer>,
    dodge_anim: Gd<Timer>,
    attack_anim: Gd<Timer>,
    attack_2_anim: Gd<Timer>,
    healing_anim: Gd<Timer>,
    healing_cooldown: Gd<Timer>,
    hurt_anim: Gd<Timer>,
    parry_anim: Gd<Timer>,
    parry: Gd<Timer>,
    perfect_parry: Gd<Timer>,
    coyote: Gd<Timer>,
    dodge_cooldown: Gd<Timer>,
    jump_limit: Gd<Timer>,
}

impl PlayerTimers {
    pub fn new(player: &Gd<Node>) -> Self {
        fn get(node: &Gd<Node>, s: &str) -> Gd<Timer> {
            node.get_node_as::<Timer>(s)
        }
        Self {
            wall_jump: get(player, "WallJump"),
            dodge_anim: get(player, "DodgeAnimation"),
            attack_anim: get(player, "AttackAnimation"),
            attack_2_anim: get(player, "AttackAnimation2"),
            healing_anim: get(player, "HealingAnimation"),
            healing_cooldown: get(player, "HealingCooldown"),
            hurt_anim: get(player, "HurtAnimation"),
            parry_anim: get(player, "ParryAnimation"),
            parry: get(player, "Parry"),
            perfect_parry: get(player, "PerfectParry"),
            coyote: get(player, "Coyote"),
            dodge_cooldown: get(player, "DodgeCooldown"),
            jump_limit: get(player, "JumpLimit"),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn connect_signals<A, B, C, D, E, F, G, H, I, J, K, L>(
        &mut self,
        on_walljump: A,
        on_dodge_anim: B,
        on_attack_anim: C,
        on_attack_2_anim: D,
        on_healing_anim: E,
        on_hurt_anim: F,
        on_parry_anim: G,
        on_parry: H,
        on_perfect_parry: I,
        on_coyote: J,
        on_dodge_cool: K,
        on_jump_limit: L,
    ) where
        A: FnMut() + 'static,
        B: FnMut() + 'static,
        C: FnMut() + 'static,
        D: FnMut() + 'static,
        E: FnMut() + 'static,
        F: FnMut() + 'static,
        G: FnMut() + 'static,
        H: FnMut() + 'static,
        I: FnMut() + 'static,
        J: FnMut() + 'static,
        K: FnMut() + 'static,
        L: FnMut() + 'static,
    {
        self.wall_jump.signals().timeout().connect(on_walljump);
        self.dodge_anim.signals().timeout().connect(on_dodge_anim);
        self.attack_anim.signals().timeout().connect(on_attack_anim);
        self.attack_2_anim
            .signals()
            .timeout()
            .connect(on_attack_2_anim);
        self.healing_anim
            .signals()
            .timeout()
            .connect(on_healing_anim);
        self.hurt_anim.signals().timeout().connect(on_hurt_anim);
        self.parry_anim.signals().timeout().connect(on_parry_anim);
        self.parry.signals().timeout().connect(on_parry);
        self.perfect_parry
            .signals()
            .timeout()
            .connect(on_perfect_parry);
        self.coyote.signals().timeout().connect(on_coyote);
        self.dodge_cooldown
            .signals()
            .timeout()
            .connect(on_dodge_cool);
        self.jump_limit.signals().timeout().connect(on_jump_limit);
    }
}
