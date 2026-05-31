//! Strongly-typed serde models for the RPG Maker MZ database files.
//!
//! Each MZ `data/*.json` file maps onto types in this module. Sub-objects that
//! are reused across several files live in [`common`].
//!
//! The MZ files store entity arrays with a leading `null` at index 0 (ids are
//! 1-based). That null padding is a concern of the asset layer, not these
//! per-entity structs.

pub mod actor;
pub mod armor;
pub mod class;
pub mod common;
pub mod enemy;
pub mod item;
pub mod skill;
pub mod state;
pub mod weapon;

pub use actor::Actor;
pub use armor::Armor;
pub use class::{Class, Learning};
pub use common::{Damage, Effect, EventCommand, Trait};
pub use enemy::{DropItem, Enemy, EnemyAction};
pub use item::Item;
pub use skill::Skill;
pub use state::State;
pub use weapon::Weapon;

#[cfg(test)]
mod tests {
    use super::{Actor, Armor, Class, Enemy, Item, Skill, State, Weapon};

    #[test]
    fn deserializes_actor() {
        let json = r#"{
            "id":1,"battlerName":"","characterIndex":0,"characterName":"Actor1",
            "classId":1,"equips":[1,1,2,3,0],"faceIndex":0,"faceName":"Actor1",
            "traits":[{"code":11,"dataId":2,"value":1}],"initialLevel":1,"maxLevel":99,
            "name":"Harold","nickname":"","note":"<cls:warrior>","profile":"A hero."
        }"#;
        let a: Actor = serde_json::from_str(json).unwrap();
        assert_eq!(a.id, 1);
        assert_eq!(a.class_id, 1);
        assert_eq!(a.equips, vec![1, 1, 2, 3, 0]);
        assert_eq!(a.max_level, 99);
        assert_eq!(a.traits.len(), 1);
        assert_eq!(a.note, "<cls:warrior>");
    }

    #[test]
    fn deserializes_class_with_learnings_and_nested_params() {
        let json = r#"{
            "id":1,"expParams":[30,20,30,30],
            "traits":[{"code":23,"dataId":0,"value":1}],
            "learnings":[{"level":1,"note":"","skillId":8},{"level":5,"note":"x","skillId":9}],
            "name":"Hero","note":"","params":[[1,2,3],[4,5,6]]
        }"#;
        let c: Class = serde_json::from_str(json).unwrap();
        assert_eq!(c.exp_params, vec![30, 20, 30, 30]);
        assert_eq!(c.learnings.len(), 2);
        assert_eq!(c.learnings[1].skill_id, 9);
        assert_eq!(c.params[1][2], 6);
    }

    #[test]
    fn deserializes_skill_with_damage_and_effects() {
        let json = r#"{
            "id":1,"animationId":1,
            "damage":{"type":1,"elementId":2,"formula":"a.atk*4-b.def*2","variance":20,"critical":true},
            "description":"Attack.","effects":[{"code":11,"dataId":0,"value1":0.5,"value2":0}],
            "hitType":1,"iconIndex":76,"message1":" attacks!","message2":"",
            "mpCost":0,"name":"Attack","note":"","occasion":1,"repeats":1,
            "requiredWtypeId1":0,"requiredWtypeId2":0,"scope":1,"speed":0,
            "stypeId":0,"successRate":100,"tpCost":0,"tpGain":10
        }"#;
        let s: Skill = serde_json::from_str(json).unwrap();
        assert_eq!(s.animation_id, 1);
        assert_eq!(s.damage.kind, 1);
        assert_eq!(s.damage.element_id, 2);
        assert_eq!(s.effects.len(), 1);
        assert_eq!(s.hit_type, 1);
        assert_eq!(s.stype_id, 0);
        assert_eq!(s.success_rate, 100);
        assert_eq!(s.tp_gain, 10);
    }

    #[test]
    fn deserializes_item() {
        let json = r#"{
            "id":1,"animationId":0,"consumable":true,
            "damage":{"type":0,"elementId":0,"formula":"0","variance":20,"critical":false},
            "description":"Heals 50 HP.","effects":[{"code":11,"dataId":0,"value1":0,"value2":50}],
            "hitType":0,"iconIndex":176,"itypeId":1,"name":"Potion","note":"",
            "occasion":0,"price":50,"repeats":1,"scope":7,"speed":0,"successRate":100,"tpGain":0
        }"#;
        let i: Item = serde_json::from_str(json).unwrap();
        assert!(i.consumable);
        assert_eq!(i.itype_id, 1);
        assert_eq!(i.price, 50);
        assert_eq!(i.scope, 7);
    }

    #[test]
    fn deserializes_weapon() {
        let json = r#"{
            "id":1,"animationId":6,"description":"","etypeId":1,
            "traits":[{"code":31,"dataId":1,"value":0}],"iconIndex":97,
            "name":"Sword","note":"","params":[0,0,10,0,0,0,0,0],"price":500,"wtypeId":2
        }"#;
        let w: Weapon = serde_json::from_str(json).unwrap();
        assert_eq!(w.etype_id, 1);
        assert_eq!(w.wtype_id, 2);
        assert_eq!(w.params.len(), 8);
        assert_eq!(w.params[2], 10);
    }

    #[test]
    fn deserializes_armor() {
        let json = r#"{
            "id":1,"atypeId":1,"description":"","etypeId":2,
            "traits":[{"code":22,"dataId":1,"value":0.1}],"iconIndex":128,
            "name":"Shield","note":"","params":[0,0,0,10,0,0,0,0],"price":300
        }"#;
        let a: Armor = serde_json::from_str(json).unwrap();
        assert_eq!(a.atype_id, 1);
        assert_eq!(a.etype_id, 2);
        assert_eq!(a.params[3], 10);
    }

    #[test]
    fn deserializes_enemy_with_sub_structs() {
        let json = r#"{
            "id":1,"actions":[{"conditionParam1":0,"conditionParam2":0,"conditionType":0,"rating":5,"skillId":1}],
            "battlerHue":0,"battlerName":"Slime",
            "dropItems":[{"dataId":1,"denominator":3,"kind":1}],
            "exp":10,"traits":[{"code":11,"dataId":3,"value":1}],"gold":5,
            "name":"Slime","note":"","params":[100,0,20,20,20,20,30,30]
        }"#;
        let e: Enemy = serde_json::from_str(json).unwrap();
        assert_eq!(e.battler_hue, 0);
        assert_eq!(e.actions[0].skill_id, 1);
        assert_eq!(e.actions[0].condition_type, 0);
        assert_eq!(e.drop_items[0].kind, 1);
        assert_eq!(e.drop_items[0].denominator, 3);
        assert_eq!(e.params.len(), 8);
    }

    #[test]
    fn deserializes_state_with_bool_flags() {
        let json = r#"{
            "id":1,"autoRemovalTiming":1,"chanceByDamage":100,
            "traits":[{"code":14,"dataId":1,"value":0}],"iconIndex":4,"maxTurns":3,
            "message1":" is poisoned!","message2":"","message3":"","message4":" recovers!",
            "minTurns":1,"motion":0,"name":"Poison","note":"","overlay":1,"priority":50,
            "releaseByDamage":false,"removeAtBattleEnd":true,"removeByDamage":false,
            "removeByRestriction":false,"removeByWalking":true,"restriction":0,"stepsToRemove":100
        }"#;
        let s: State = serde_json::from_str(json).unwrap();
        assert_eq!(s.auto_removal_timing, 1);
        assert_eq!(s.chance_by_damage, 100);
        assert!(s.remove_at_battle_end);
        assert!(s.remove_by_walking);
        assert!(!s.remove_by_damage);
        assert_eq!(s.steps_to_remove, 100);
    }
}
