use godot::{
    classes::{Node, Timer},
    obj::Gd,
};

use crate::entities::graphics::Graphics;

#[derive(Clone)]
pub struct PlayerTimers {
    pub wall_jump: Gd<Timer>,
    pub dodge_anim: Gd<Timer>,
    pub attack_anim: Gd<Timer>,
    pub attack_2_anim: Gd<Timer>,
    pub healing_anim: Gd<Timer>,
    pub healing_cooldown: Gd<Timer>,
    pub hurt_anim: Gd<Timer>,
    pub parry_anim: Gd<Timer>,
    pub parry: Gd<Timer>,
    pub perfect_parry: Gd<Timer>,
    pub coyote: Gd<Timer>,
    pub dodge_cooldown: Gd<Timer>,
    pub jump_limit: Gd<Timer>,
    pub charged_attack_anim: Gd<Timer>,
    pub spell_cooldown: Gd<Timer>,
    pub cast_spell_anim: Gd<Timer>,
}

impl PlayerTimers {
    pub fn new(player: &Gd<Node>, graphics: &mut Graphics) -> Self {
        fn get(node: &Gd<Node>, s: &str) -> Gd<Timer> {
            node.get_node_as::<Timer>(s)
        }
        let mut this = Self {
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
            charged_attack_anim: get(player, "ChargedAttack"),
            spell_cooldown: get(player, "SpellCooldown"),
            cast_spell_anim: get(player, "CastSpellAnimation"),
        };
        this.dodge_anim
            .set_wait_time(graphics.get_animation_length("dodge_right"));
        this.attack_anim
            .set_wait_time(graphics.get_animation_length("attack_right"));
        this.attack_2_anim
            .set_wait_time(graphics.get_animation_length("chainattack_right"));
        this.healing_anim
            .set_wait_time(graphics.get_animation_length("heal_right"));
        this.parry_anim
            .set_wait_time(graphics.get_animation_length("parry_right"));
        this.hurt_anim
            .set_wait_time(graphics.get_animation_length("hurt_right"));
        this.charged_attack_anim
            .set_wait_time(graphics.get_animation_length("chargedattack"));
        this
    }

    #[allow(clippy::too_many_arguments)]
    pub fn connect_signals<A, B, C, D, E, F, G, H, I, J>(
        &mut self,
        on_walljump: A,
        on_dodge_anim: B,
        on_attack_anim: C,
        on_attack_2_anim: D,
        on_healing_anim: E,
        on_hurt_anim: F,
        on_parry_anim: G,
        on_jump_limit: H,
        on_charged_attack_anim: I,
        on_cast_spell_anim: J,
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
        self.jump_limit.signals().timeout().connect(on_jump_limit);
        self.charged_attack_anim
            .signals()
            .timeout()
            .connect(on_charged_attack_anim);
        self.cast_spell_anim
            .signals()
            .timeout()
            .connect(on_cast_spell_anim);
    }
}
