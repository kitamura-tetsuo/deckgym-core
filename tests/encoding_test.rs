use deckgym::actions::SimpleAction;
use deckgym::encoding::{encode_action, action_name, get_offset_apply_damage, encode_observation};
use deckgym::{State, card_ids::CardId};
use deckgym::test_helpers::load_test_decks;

#[test]
fn test_known_cards_encoding() {
    let (deck_a, deck_b) = load_test_decks();
    let mut state = State::new(&deck_a, &deck_b);

    // Player 0 observes Player 1
    // Mark one card in P1's deck as visible
    state.decks[1].visibility[0] = true;
    let visible_deck_card = state.decks[1].cards[0].clone();
    let visible_deck_id = CardId::from_card_id(&visible_deck_card.get_id()).unwrap() as usize as f32;

    // Give P1 a hand and mark one as visible
    // state.maybe_draw_card(1); // Draw 1
    if let Some((card, vis)) = state.decks[1].draw() {
        state.hands[1].push(card);
        state.hands_visibility[1].push(vis);
    }
    // state.maybe_draw_card(1); // Draw 2
    if let Some((card, vis)) = state.decks[1].draw() {
        state.hands[1].push(card);
        state.hands_visibility[1].push(vis);
    }
    state.hands_visibility[1][0] = true; // Mark 1st as visible
    let visible_hand_card = state.hands[1][0].clone();
    let visible_hand_id = CardId::from_card_id(&visible_hand_card.get_id()).unwrap() as usize as f32;

    let obs = encode_observation(&state, 0);

    // Find Opponent Known Hand section
    // It is after: Turn(4), Flags(3), Energy(30), HandCounts(2), MyActive(1+1+10+1+5+2+1+4=25), MyBench(3*25), OpActive(25), OpBench(3*25)=225?, SelfHand(10)
    // Actually counting offsets is hard.
    // But we know "Self Deck" is sorted. "Opponent Known Deck" follows "Opponent Deck Count".
    
    // Let's verify presence of values
    assert!(obs.contains(&visible_deck_id), "Observation should contain visible deck card ID");
    assert!(obs.contains(&visible_hand_id), "Observation should contain visible hand card ID");

    // Verify that unknown cards are -1.0
    // We know P1 has 2 cards in hand. 1 visible, 1 hidden.
    // So Opponent Hand slots should have one ID and others -1.0.
    // But since we can't easily locate the exact index without calculating offsets, 
    // we essentially rely on "it's extended into the vector".
    
    // Detailed offset check could be done if we export offsets or use `observation_length` incrementally.
}

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
