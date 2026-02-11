use rand::SeedableRng;
use crate::{
    actions::{apply_action, Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    hooks::to_playable_card,
    tool_ids::ToolId,
    State,
};
use crate::move_generation::generate_possible_trainer_actions;

#[test]
fn test_guzma_discards_multiple_tools() {
    let mut state = State::default();
    state.turn_count = 3;
    state.current_player = 0;

    // Opponent (Player 1) has 2 Pokemon, each with a tool
    let bulbasaur_card = get_card_by_enum(CardId::A1001Bulbasaur);
    let mut opponent_active = to_playable_card(&bulbasaur_card, false);
    opponent_active.attached_tool = Some(ToolId::A2148RockyHelmet);
    state.in_play_pokemon[1][0] = Some(opponent_active);

    let charmander_card = get_card_by_enum(CardId::A1033Charmander);
    let mut opponent_bench = to_playable_card(&charmander_card, false);
    opponent_bench.attached_tool = Some(ToolId::A2147GiantCape);
    state.in_play_pokemon[1][1] = Some(opponent_bench);

    // Player 0 has Guzma in hand
    let guzma_card_enum = get_card_by_enum(CardId::A3151Guzma);
    let guzma_card = if let crate::models::Card::Trainer(tc) = &guzma_card_enum { tc.clone() } else { panic!("Guzma should be a trainer card") };
    state.hands[0].push(guzma_card_enum.clone());

    // Act: Play Guzma
    let action = Action {
        actor: 0,
        action: SimpleAction::Play { trainer_card: guzma_card.clone() },
        is_stack: false,
    };

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    apply_action(&mut rng, &mut state, &action);

    // Assert: Both tools should be gone
    assert!(state.in_play_pokemon[1][0].as_ref().unwrap().attached_tool.is_none());
    assert!(state.in_play_pokemon[1][1].as_ref().unwrap().attached_tool.is_none());
}

#[test]
fn test_guzma_cannot_be_played_without_tools() {
    let mut state = State::default();
    state.turn_count = 3;
    state.current_player = 0;

    // Opponent (Player 1) has no tools
    let bulbasaur_card = get_card_by_enum(CardId::A1001Bulbasaur);
    state.in_play_pokemon[1][0] = Some(to_playable_card(&bulbasaur_card, false));

    // Player 0 has Guzma in hand
    let guzma_card_enum = get_card_by_enum(CardId::A3151Guzma);
    let guzma_card = if let crate::models::Card::Trainer(tc) = &guzma_card_enum { tc.clone() } else { panic!("Guzma should be a trainer card") };

    // Check move generation
    let actions = generate_possible_trainer_actions(&state, &guzma_card).unwrap();
    
    // Should not be able to play Guzma
    assert!(actions.is_empty(), "Guzma should not be playable if opponent has no tools");
}
