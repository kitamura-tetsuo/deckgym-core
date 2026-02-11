use deckgym::{
    actions::{apply_action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    hooks::to_playable_card,
    generate_possible_actions,
    models::{EnergyType, Card},
    State,
};
use rand::SeedableRng;

#[test]
fn test_piers_requires_obstagoon_and_discards_energy() {
    let mut state = State::default();
    let piers = get_card_by_enum(CardId::B2152Piers);
    let opponent_active = get_card_by_enum(CardId::A1001Bulbasaur);
    let obstagoon = get_card_by_enum(CardId::B2100GalarianObstagoon);

    state.turn_count = 1;
    state.in_play_pokemon[1][0] = Some(to_playable_card(&opponent_active, false));
    state.in_play_pokemon[1][0].as_mut().unwrap().attached_energy.push(EnergyType::Grass);
    state.in_play_pokemon[1][0].as_mut().unwrap().attached_energy.push(EnergyType::Grass);
    state.hands[0].push(piers.clone());

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // 1. Check Piers cannot be played without Obstagoon
    let (_, actions) = generate_possible_actions(&state);
    assert!(!actions.iter().any(|a| matches!(a.action, SimpleAction::Play { .. })));

    // 2. Add Obstagoon and check Piers can be played
    state.in_play_pokemon[0][0] = Some(to_playable_card(&obstagoon, false));
    let (_, actions) = generate_possible_actions(&state);
    let play_action = actions.iter().find(|a| matches!(a.action, SimpleAction::Play { .. })).expect("Piers should be playable");

    apply_action(&mut rng, &mut state, play_action);

    // 3. Verify energy discarded
    assert!(state.in_play_pokemon[1][0].as_ref().unwrap().attached_energy.is_empty());
}

#[test]
fn test_sightseer_draws_stage1_pokemon() {
    let mut state = State::default();
    let sightseer = get_card_by_enum(CardId::B2150Sightseer);
    let ivysaur = get_card_by_enum(CardId::A1002Ivysaur); // Stage 1
    let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur); // Stage 0

    state.turn_count = 1;
    state.hands[0].push(sightseer.clone());
    state.decks[0].cards = vec![
        ivysaur.clone(),
        bulbasaur.clone(),
        bulbasaur.clone(),
        bulbasaur.clone(),
    ];

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let (_, actions) = generate_possible_actions(&state);
    let play_action = actions.iter().find(|a| matches!(a.action, SimpleAction::Play { .. })).expect("Sightseer should be playable");

    apply_action(&mut rng, &mut state, play_action);

    // Verify Ivysaur in hand, rest in deck
    assert!(state.hands[0].iter().any(|c| c.get_id() == ivysaur.get_id()));
    assert_eq!(state.decks[0].cards.len(), 3);
}

#[test]
fn test_juggler_requires_3_types_and_moves_energy() {
    let mut state = State::default();
    let juggler = get_card_by_enum(CardId::B2151Juggler);
    let frosmoth = get_card_by_enum(CardId::A1093Frosmoth);

    state.turn_count = 1;
    state.in_play_pokemon[0][0] = Some(to_playable_card(&frosmoth, false)); // Active
    state.in_play_pokemon[0][1] = Some(to_playable_card(&frosmoth, false)); // Bench

    // Case 1: Only 1 type (Water)
    state.in_play_pokemon[0][1].as_mut().unwrap().attached_energy.push(EnergyType::Water);
    state.hands[0].push(juggler.clone());

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let (_, actions) = generate_possible_actions(&state);
    assert!(!actions.iter().any(|a| matches!(a.action, SimpleAction::Play { .. })));

    // Case 2: 3 types
    state.in_play_pokemon[0][0].as_mut().unwrap().attached_energy.push(EnergyType::Fire);
    state.in_play_pokemon[0][0].as_mut().unwrap().attached_energy.push(EnergyType::Psychic);
    
    let (_, actions) = generate_possible_actions(&state);
    let play_action = actions.iter().find(|a| matches!(a.action, SimpleAction::Play { .. })).expect("Juggler should be playable now");

    apply_action(&mut rng, &mut state, play_action);

    // Verify energy moved to active
    assert_eq!(state.in_play_pokemon[0][0].as_ref().unwrap().attached_energy.len(), 3);
    assert!(state.in_play_pokemon[0][1].as_ref().unwrap().attached_energy.is_empty());
}
