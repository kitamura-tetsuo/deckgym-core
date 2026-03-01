use deckgym::card_ids::CardId;
use deckgym::database::get_card_by_enum;
use deckgym::hooks::to_playable_card;
use deckgym::actions::{Action, SimpleAction, apply_action};
use deckgym::move_generation::generate_possible_actions;
use deckgym::state::State;
use rand::SeedableRng;

#[test]
fn test_klefki_dismantling_keys_discards_tool_and_self() {
    let mut state = State::default();
    
    // Player 0 has Klefki on the bench
    let klefki = get_card_by_enum(CardId::B1120Klefki);
    state.in_play_pokemon[0][1] = Some(to_playable_card(&klefki, false));
    
    // Just to fill the active spot
    let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur);
    state.in_play_pokemon[0][0] = Some(to_playable_card(&bulbasaur, false));
    
    // Player 1 has active pokemon with a Tool attached
    let opponent_mon = get_card_by_enum(CardId::A1033Charmander);
    let mut opponent_playable = to_playable_card(&opponent_mon, false);
    opponent_playable.attached_tool = Some(get_card_by_enum(CardId::A2148RockyHelmet));
    state.in_play_pokemon[1][0] = Some(opponent_playable);
    
    state.current_player = 0;
    state.turn_count = 1;
    
    // Check possible moves for player 0
    let possible_moves = generate_possible_actions(&state);
    assert!(possible_moves.1.iter().any(|a| matches!(a.action, SimpleAction::UseAbility { in_play_idx: 1 })), "Klefki's ability should be available");

    // Act: Player 0 uses Klefki's ability
    let action = Action {
        actor: 0,
        action: SimpleAction::UseAbility { in_play_idx: 1 },
        is_stack: false,
    };
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    apply_action(&mut rng, &mut state, &action);
    
    // Assert: Tool was discarded from opponent
    let opponent_active = state.in_play_pokemon[1][0].as_ref().unwrap();
    assert!(opponent_active.attached_tool.is_none(), "Tool should be discarded from opponent's active Pokemon");
    assert_eq!(state.discard_piles[1].len(), 1, "Discard pile for Player 1 should receive the tool");
    if let deckgym::models::Card::Trainer(tc) = &state.discard_piles[1][0] {
        assert_eq!(tc.name, "Rocky Helmet", "Discarded tool should be Rocky Helmet");
    } else {
        panic!("Discarded item should be a trainer card");
    }

    // Assert: Klefki discarded from play
    assert!(state.in_play_pokemon[0][1].is_none(), "Bench slot 1 should be empty, Klefki discarded");
    assert_eq!(state.discard_piles[0].len(), 1, "Klefki should be in Player 0's discard pile");
}

#[test]
fn test_klefki_dismantling_keys_cannot_be_used_if_no_tool() {
    let mut state = State::default();
    
    // Player 0 has Klefki on the bench
    let klefki = get_card_by_enum(CardId::B1120Klefki);
    state.in_play_pokemon[0][1] = Some(to_playable_card(&klefki, false));
    
    // Player 0 has active bulbasaur
    let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur);
    state.in_play_pokemon[0][0] = Some(to_playable_card(&bulbasaur, false));
    
    // Player 1 has active pokemon WITH NO TOOL
    let opponent_mon = get_card_by_enum(CardId::A1033Charmander);
    state.in_play_pokemon[1][0] = Some(to_playable_card(&opponent_mon, false));
    
    state.current_player = 0;
    state.turn_count = 1;

    let possible_moves = generate_possible_actions(&state);
    
    assert!(!possible_moves.1.iter().any(|a| matches!(a.action, SimpleAction::UseAbility { in_play_idx: 1 })), "Klefki's ability should NOT be available if opponent has no tool");
}
