use deckgym::{card_ids::CardId, database::get_card_by_enum, hooks::to_playable_card, State, Deck};

#[test]
fn test_stadium_can_be_set_and_retrieved() {
    let mut state = State::new(&Deck::default(), &Deck::default());
    let training_area = get_card_by_enum(CardId::B2153TrainingArea);
    
    // Initially no Stadium
    assert!(state.get_stadium().is_none());
    
    // Set Stadium
    state.set_stadium(training_area.clone(), 0);
    
    // Stadium should be set
    assert!(state.get_stadium().is_some());
    assert_eq!(state.get_stadium().unwrap().get_id(), "B2 153");
}

#[test]
fn test_stadium_replacement() {
    let mut state = State::new(&Deck::default(), &Deck::default());
    let training_area = get_card_by_enum(CardId::B2153TrainingArea);
    let starting_plains = get_card_by_enum(CardId::B2154StartingPlains);
    
    // Set first Stadium
    state.set_stadium(training_area.clone(), 0);
    assert_eq!(state.get_stadium().unwrap().get_id(), "B2 153");
    
    // Replace with second Stadium
    state.set_stadium(starting_plains.clone(), 1);
    assert_eq!(state.get_stadium().unwrap().get_id(), "B2 154");
    
    // Old Stadium should be in player 1's discard pile
    assert_eq!(state.discard_piles[1].len(), 1);
    assert_eq!(state.discard_piles[1][0].get_id(), "B2 153");
}

#[test]
fn test_training_area_damage_bonus() {
    use deckgym::hooks::modify_damage;
    
    let mut state = State::new(&Deck::default(), &Deck::default());
    
    // Set up a Stage 1 Pokemon (Ivysaur) as attacker
    let ivysaur = get_card_by_enum(CardId::A1002Ivysaur);
    state.in_play_pokemon[0][0] = Some(to_playable_card(&ivysaur, false));
    
    // Set up defender
    let charmander = get_card_by_enum(CardId::A1033Charmander);
    state.in_play_pokemon[1][0] = Some(to_playable_card(&charmander, false));
    
    // Damage without Training Area
    let base_damage = modify_damage(&state, (0, 0), (30, 1, 0), true, None);
    
    // Add Training Area Stadium
    let training_area = get_card_by_enum(CardId::B2153TrainingArea);
    state.set_stadium(training_area, 0);
    
    // Damage with Training Area should be +10
    let damage_with_stadium = modify_damage(&state, (0, 0), (30, 1, 0), true, None);
    assert_eq!(damage_with_stadium, base_damage + 10);
}

#[test]
fn test_training_area_only_affects_stage_1() {
    use deckgym::hooks::modify_damage;
    
    let mut state = State::new(&Deck::default(), &Deck::default());
    
    // Set up a Basic Pokemon (Bulbasaur) as attacker
    let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur);
    state.in_play_pokemon[0][0] = Some(to_playable_card(&bulbasaur, false));
    
    // Set up defender
    let charmander = get_card_by_enum(CardId::A1033Charmander);
    state.in_play_pokemon[1][0] = Some(to_playable_card(&charmander, false));
    
    // Add Training Area Stadium
    let training_area = get_card_by_enum(CardId::B2153TrainingArea);
    state.set_stadium(training_area, 0);
    
    // Damage should NOT increase for Basic Pokemon
    let damage = modify_damage(&state, (0, 0), (30, 1, 0), true, None);
    assert_eq!(damage, 30); // No bonus for Basic Pokemon
}
