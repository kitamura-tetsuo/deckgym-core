use deckgym::actions::SimpleAction;
use deckgym::encoding::{encode_action, action_name, get_offset_apply_damage};

#[test]
fn test_apply_damage_encoding() {
    for i in 0..4 {
        let action = SimpleAction::ApplyDamage {
            attacking_ref: (0, 0),
            targets: vec![(30, 1, i)],
            is_from_active_attack: true,
        };
        
        let encoded = encode_action(&action).expect("Should encode ApplyDamage");
        let offset = get_offset_apply_damage();
        assert_eq!(encoded, offset + i);
        
        let name = action_name(encoded);
        assert_eq!(name, format!("ApplyDamage({})", i));
    }
    
    // Test validation
    let invalid_action = SimpleAction::ApplyDamage {
        attacking_ref: (0, 0),
        targets: vec![(30, 1, 4)],
        is_from_active_attack: true,
    };
    assert!(encode_action(&invalid_action).is_none());
    
    let empty_action = SimpleAction::ApplyDamage {
        attacking_ref: (0, 0),
        targets: vec![],
        is_from_active_attack: true,
    };
    assert!(encode_action(&empty_action).is_none());
}
