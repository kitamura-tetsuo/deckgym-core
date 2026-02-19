use deckgym::{
    actions::{apply_action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    hooks::to_playable_card,
    generate_possible_actions,
    models::EnergyType,
    State,
};
use rand::SeedableRng;

#[test]
fn test_gourgeist_soul_shot_discards_and_deals_damage() {
    let mut state = State::default();
    let gourgeist = get_card_by_enum(CardId::B2072Gourgeist);
    let opponent_active = get_card_by_enum(CardId::A1002Ivysaur); // 90 HP
    let discard_fodder = get_card_by_enum(CardId::PA001Potion);

    state.turn_count = 1;

    let mut gourgeist_playable = to_playable_card(&gourgeist, false);
    gourgeist_playable.attached_energy.push(EnergyType::Psychic);

    state.in_play_pokemon[0][0] = Some(gourgeist_playable);
    state.in_play_pokemon[1][0] = Some(to_playable_card(&opponent_active, false));
    state.hands[0].push(discard_fodder.clone());
    state.hands_visibility[0].push(true);

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    
    // Check possible actions
    let (_, actions) = generate_possible_actions(&state);
    let attack_action = actions.iter().find(|a| matches!(a.action, SimpleAction::Attack(_))).expect("Should have attack action");
    
    apply_action(&mut rng, &mut state, attack_action);

    // After attack, it should have queued a discard action because Soul Shot has effect
    let (_, actions) = generate_possible_actions(&state);
    let discard_action = actions.iter().find(|a| matches!(a.action, SimpleAction::DiscardOwnCard { .. })).expect("Should have discard action");
    
    apply_action(&mut rng, &mut state, discard_action);

    // Verify damage dealt (Soul Shot does 70 damage)
    // Ivysaur has 90 HP
    assert_eq!(state.in_play_pokemon[1][0].as_ref().unwrap().remaining_hp, 90 - 70);
    // Verify card discarded
    assert!(state.hands[0].is_empty());
    assert_eq!(state.discard_piles[0].len(), 1);
}

#[test]
fn test_gourgeist_soul_shot_fails_if_hand_empty() {
    let mut state = State::default();
    let gourgeist = get_card_by_enum(CardId::B2072Gourgeist);
    let opponent_active = get_card_by_enum(CardId::A1002Ivysaur);

    state.turn_count = 1;

    let mut gourgeist_playable = to_playable_card(&gourgeist, false);
    gourgeist_playable.attached_energy.push(EnergyType::Psychic);

    state.in_play_pokemon[0][0] = Some(gourgeist_playable);
    state.in_play_pokemon[1][0] = Some(to_playable_card(&opponent_active, false));
    // Hand is empty

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    
    let (_, actions) = generate_possible_actions(&state);
    let attack_action = actions.iter().find(|a| matches!(a.action, SimpleAction::Attack(_))).expect("Should have attack action");
    
    apply_action(&mut rng, &mut state, attack_action);

    // Verify NO damage dealt
    assert_eq!(state.in_play_pokemon[1][0].as_ref().unwrap().remaining_hp, 90);
}
