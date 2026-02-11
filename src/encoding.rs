use crate::{
    actions::SimpleAction,
    card_ids::CardId,
    models::{EnergyType, PlayedCard},
    state::State,

};
use strum::IntoEnumIterator;

pub const MAX_BENCH_SIZE: usize = 3;
pub const MAX_IN_PLAY: usize = 1 + MAX_BENCH_SIZE; // 1 active + 3 bench
pub const ENERGY_TYPES_COUNT: usize = 10; // Grass, Fire, ..., Colorless

pub fn get_card_count() -> usize {
    CardId::iter().len()
}

/// Defines each action slot with its name and size.
/// The order here determines the offset order in the action space.
/// When adding a new action type, add it to this list with the correct size.
#[derive(Debug, Clone, Copy)]
struct ActionSlot {
    name: &'static str,
    size: usize,
}

/// Generates the complete action slot registry.
/// This is the single source of truth for action space layout.
fn get_action_slots() -> Vec<ActionSlot> {
    let card_count = CardId::iter().len();

    
    vec![
        // Basic turn actions
        ActionSlot { name: "EndTurn", size: 1 },
        ActionSlot { name: "Attack", size: 3 },
        ActionSlot { name: "Retreat", size: 4 },
        ActionSlot { name: "UseAbility", size: 4 },
        ActionSlot { name: "Place", size: card_count * 4 },
        ActionSlot { name: "Evolve", size: card_count * 4 },
        ActionSlot { name: "Play", size: card_count },
        ActionSlot { name: "Attach", size: ENERGY_TYPES_COUNT * 4 },
        ActionSlot { name: "AttachTool", size: card_count * 4 },
        ActionSlot { name: "Activate", size: 4 },
        ActionSlot { name: "DrawCard", size: 1 },
        // Extended actions
        ActionSlot { name: "MoveEnergy", size: 3 * ENERGY_TYPES_COUNT }, // Bench(1..3) -> Active(0), 10 energy types
        ActionSlot { name: "CommunicatePokemon", size: card_count },
        ActionSlot { name: "ShufflePokemonIntoDeck", size: card_count },
        ActionSlot { name: "ShuffleOpponentSupporter", size: card_count },
        ActionSlot { name: "DiscardOpponentSupporter", size: card_count },
        ActionSlot { name: "DiscardOwnCard", size: card_count },
        ActionSlot { name: "AttachFromDiscard", size: 4 },
        ActionSlot { name: "ApplyEeveeBagDamageBoost", size: 1 },
        ActionSlot { name: "HealAllEeveeEvolutions", size: 1 },
        ActionSlot { name: "DiscardFossil", size: 4 },
        ActionSlot { name: "Heal", size: 4 },
        ActionSlot { name: "MoveAllDamage", size: 16 }, // 4 * 4 (from x to y)
        ActionSlot { name: "ApplyDamage", size: 1 }, // Deterministic action, single slot
        ActionSlot { name: "HealAndDiscardEnergy", size: 4 },
        ActionSlot { name: "ReturnPokemonToHand", size: 4 },
        ActionSlot { name: "UseOpponentAttack", size: 3 },
        ActionSlot { name: "Noop", size: 1 },
    ]
}

/// Returns the offset for a given action slot name.
/// Panics if the slot name is not found.
fn get_slot_offset(slot_name: &str) -> usize {
    let slots = get_action_slots();
    let mut offset = 0;
    for slot in &slots {
        if slot.name == slot_name {
            return offset;
        }
        offset += slot.size;
    }
    panic!("Unknown action slot: {}", slot_name);
}

pub fn get_action_space_size() -> usize {
    get_action_slots().iter().map(|s| s.size).sum()
}

// Action Offsets - now derived from the ActionSlot registry
fn get_offset_end_turn() -> usize { get_slot_offset("EndTurn") }
fn get_offset_attack() -> usize { get_slot_offset("Attack") }
fn get_offset_retreat() -> usize { get_slot_offset("Retreat") }
fn get_offset_use_ability() -> usize { get_slot_offset("UseAbility") }
fn get_offset_place() -> usize { get_slot_offset("Place") }
fn get_offset_evolve() -> usize { get_slot_offset("Evolve") }
fn get_offset_play() -> usize { get_slot_offset("Play") }
fn get_offset_attach() -> usize { get_slot_offset("Attach") }
fn get_offset_attach_tool() -> usize { get_slot_offset("AttachTool") }
fn get_offset_activate() -> usize { get_slot_offset("Activate") }
fn get_offset_draw_card() -> usize { get_slot_offset("DrawCard") }
fn get_offset_move_energy() -> usize { get_slot_offset("MoveEnergy") }
fn get_offset_communicate() -> usize { get_slot_offset("CommunicatePokemon") }
fn get_offset_shuffle_into_deck() -> usize { get_slot_offset("ShufflePokemonIntoDeck") }
fn get_offset_shuffle_opponent_supporter() -> usize { get_slot_offset("ShuffleOpponentSupporter") }
fn get_offset_discard_opponent_supporter() -> usize { get_slot_offset("DiscardOpponentSupporter") }
fn get_offset_discard_own_card() -> usize { get_slot_offset("DiscardOwnCard") }
fn get_offset_attach_from_discard() -> usize { get_slot_offset("AttachFromDiscard") }
fn get_offset_eevee_boost() -> usize { get_slot_offset("ApplyEeveeBagDamageBoost") }
fn get_offset_heal_eevee() -> usize { get_slot_offset("HealAllEeveeEvolutions") }
fn get_offset_discard_fossil() -> usize { get_slot_offset("DiscardFossil") }
fn get_offset_heal() -> usize { get_slot_offset("Heal") }
fn get_offset_move_damage() -> usize { get_slot_offset("MoveAllDamage") }
fn get_offset_apply_damage() -> usize { get_slot_offset("ApplyDamage") }
fn get_offset_heal_discard() -> usize { get_slot_offset("HealAndDiscardEnergy") }
fn get_offset_return_hand() -> usize { get_slot_offset("ReturnPokemonToHand") }
fn get_offset_use_opp_attack() -> usize { get_slot_offset("UseOpponentAttack") }
fn get_offset_noop() -> usize { get_slot_offset("Noop") }

pub fn encode_action(action: &SimpleAction) -> Option<usize> {
    match action {
        SimpleAction::EndTurn => Some(get_offset_end_turn()),
        SimpleAction::Attack(idx) => {
            if *idx < 3 {
                Some(get_offset_attack() + idx)
            } else {
                None
            }
        }
        SimpleAction::Retreat(idx) => {
            if *idx < 4 {
                Some(get_offset_retreat() + idx)
            } else {
                None
            }
        }
        SimpleAction::UseAbility { in_play_idx } => {
            if *in_play_idx < 4 {
                Some(get_offset_use_ability() + in_play_idx)
            } else {
                None
            }
        }
        SimpleAction::Place(card, slot) => {
            if *slot < 4 {
                let card_id = CardId::from_card_id(&card.get_id())?;
                let card_idx = card_id as usize;
                Some(get_offset_place() + card_idx * 4 + slot)
            } else {
                None
            }
        }
        SimpleAction::Evolve {
            evolution,
            in_play_idx,
            ..
        } => {
            if *in_play_idx < 4 {
                let card_id = CardId::from_card_id(&evolution.get_id())?;
                let card_idx = card_id as usize;
                Some(get_offset_evolve() + card_idx * 4 + in_play_idx)
            } else {
                None
            }
        }
        SimpleAction::Play { trainer_card } => {
            let card_id = CardId::from_card_id(&trainer_card.id)?;
            let card_idx = card_id as usize;
            Some(get_offset_play() + card_idx)
        }
        SimpleAction::Attach { attachments, .. } => {
            if let Some((_, energy_type, slot)) = attachments.first() {
                if *slot < 4 {
                    let energy_idx = energy_type_to_index(*energy_type);
                    Some(get_offset_attach() + energy_idx * 4 + slot)
                } else {
                    None
                }
            } else {
                None
            }
        }
        SimpleAction::AttachTool {
            in_play_idx,
            tool_card,
        } => {
            let card_id = CardId::from_card_id(&tool_card.get_id())?;
            let card_idx = card_id as usize;
            if *in_play_idx < 4 {
                Some(get_offset_attach_tool() + card_idx * 4 + in_play_idx)
            } else {
                None
            }
        }
        SimpleAction::Activate {
            player: _,
            in_play_idx,
        } => {
            if *in_play_idx < 4 {
                Some(get_offset_activate() + in_play_idx)
            } else {
                None
            }
        }
        SimpleAction::DrawCard { .. } => Some(get_offset_draw_card()),
        
        // New Mappings
        SimpleAction::MoveEnergy { from_in_play_idx, to_in_play_idx, energy_type, .. } => {
             // Supports Bench(1..3) -> any, or just Active?
             // Elemental switch is specifically Bench -> Active.
             // We map generically: from (1..3) -> 0.
             if *to_in_play_idx == 0 && *from_in_play_idx > 0 && *from_in_play_idx < 4 {
                 let energy_idx = energy_type_to_index(*energy_type);
                 let from_bench_idx = from_in_play_idx - 1; // 0..2
                 Some(get_offset_move_energy() + from_bench_idx * 10 + energy_idx)
             } else {
                 None
             }
        }
        SimpleAction::CommunicatePokemon { hand_pokemon } => {
             let card_id = CardId::from_card_id(&hand_pokemon.get_id())?;
             Some(get_offset_communicate() + card_id as usize)
        },
        SimpleAction::ShufflePokemonIntoDeck { hand_pokemon, .. } => {
             let card_id = CardId::from_card_id(&hand_pokemon.get_id())?;
             Some(get_offset_shuffle_into_deck() + card_id as usize)
        },
        SimpleAction::ShuffleOpponentSupporter { supporter_card } => {
            let card_id = CardId::from_card_id(&supporter_card.get_id())?;
             Some(get_offset_shuffle_opponent_supporter() + card_id as usize)
        },
        SimpleAction::DiscardOpponentSupporter { supporter_card } => {
             let card_id = CardId::from_card_id(&supporter_card.get_id())?;
             Some(get_offset_discard_opponent_supporter() + card_id as usize)
        },
        SimpleAction::DiscardOwnCard { card } => {
             let card_id = CardId::from_card_id(&card.get_id())?;
             Some(get_offset_discard_own_card() + card_id as usize)
        },
        SimpleAction::AttachFromDiscard { in_play_idx, .. } => {
             if *in_play_idx < 4 {
                 Some(get_offset_attach_from_discard() + in_play_idx)
             } else {
                 None
             }
        },
        SimpleAction::ApplyEeveeBagDamageBoost => Some(get_offset_eevee_boost()),
        SimpleAction::HealAllEeveeEvolutions => Some(get_offset_heal_eevee()),
        SimpleAction::DiscardFossil { in_play_idx } => {
             if *in_play_idx < 4 {
                 Some(get_offset_discard_fossil() + in_play_idx)
             } else {
                 None
             }
        },
        SimpleAction::Heal { in_play_idx, .. } => {
             if *in_play_idx < 4 {
                 Some(get_offset_heal() + in_play_idx)
             } else {
                 None
             }
        },
        SimpleAction::MoveAllDamage { from, to } => {
             if *from < 4 && *to < 4 {
                 Some(get_offset_move_damage() + from * 4 + to)
             } else {
                 None
             }
        },
        SimpleAction::ApplyDamage { .. } => {
            // ApplyDamage is deterministic but may appear in legal_actions.
            // Now has its own dedicated slot.
            Some(get_offset_apply_damage())
        }
        SimpleAction::HealAndDiscardEnergy { in_play_idx, .. } => {
            if *in_play_idx < 4 {
                Some(get_offset_heal_discard() + in_play_idx)
            } else {
                None
            }
        }
        SimpleAction::ReturnPokemonToHand { in_play_idx } => {
            if *in_play_idx < 4 {
                Some(get_offset_return_hand() + in_play_idx)
            } else {
                None
            }
        }
        SimpleAction::UseOpponentAttack(idx) => {
            if *idx < 3 {
                Some(get_offset_use_opp_attack() + idx)
            } else {
                None
            }
        }
        SimpleAction::Noop => Some(get_offset_noop()),
    }
}

pub fn action_name(id: usize) -> String {
    // Get all offsets from the registry
    let offset_end_turn = get_offset_end_turn();
    let offset_attack = get_offset_attack();
    let offset_retreat = get_offset_retreat();
    let offset_use_ability = get_offset_use_ability();
    let offset_place = get_offset_place();
    let offset_evolve = get_offset_evolve();
    let offset_play = get_offset_play();
    let offset_attach = get_offset_attach();
    let offset_attach_tool = get_offset_attach_tool();
    let offset_activate = get_offset_activate();
    let offset_draw_card = get_offset_draw_card();
    let offset_move_energy = get_offset_move_energy();
    let offset_communicate = get_offset_communicate();
    let offset_shuffle_into_deck = get_offset_shuffle_into_deck();
    let offset_shuffle_op_sup = get_offset_shuffle_opponent_supporter();
    let offset_discard_op_sup = get_offset_discard_opponent_supporter();
    let offset_discard_own = get_offset_discard_own_card();
    let offset_attach_from_discard = get_offset_attach_from_discard();
    let offset_eevee_boost = get_offset_eevee_boost();
    let offset_heal_eevee = get_offset_heal_eevee();
    let offset_discard_fossil = get_offset_discard_fossil();
    let offset_heal = get_offset_heal();
    let offset_move_damage = get_offset_move_damage();
    let offset_apply_damage = get_offset_apply_damage();
    let offset_noop = get_offset_noop();
    
    if id == offset_end_turn {
        return "EndTurn".to_string();
    }
    if (offset_attack..offset_retreat).contains(&id) {
        return format!("Attack({})", id - offset_attack);
    }
    if (offset_retreat..offset_use_ability).contains(&id) {
        return format!("Retreat({})", id - offset_retreat);
    }
    if (offset_use_ability..offset_place).contains(&id) {
        return format!("UseAbility({})", id - offset_use_ability);
    }
    if (offset_place..offset_evolve).contains(&id) {
        let val = id - offset_place;
        let slot = val % 4;
        let card_idx = val / 4;
        let card_id = card_index_to_id(card_idx);
        return format!("Place({:?}, {})", card_id, slot);
    }
    if (offset_evolve..offset_play).contains(&id) {
        let val = id - offset_evolve;
        let slot = val % 4;
        let card_idx = val / 4;
        let card_id = card_index_to_id(card_idx);
        return format!("Evolve({:?}, {})", card_id, slot);
    }
    if (offset_play..offset_attach).contains(&id) {
        let card_idx = id - offset_play;
        let card_id = card_index_to_id(card_idx);
        return format!("Play({:?})", card_id);
    }
    if (offset_attach..offset_attach_tool).contains(&id) {
        let val = id - offset_attach;
        let slot = val % 4;
        let energy_idx = val / 4;
        let energy_type = index_to_energy_type(energy_idx);
        return format!("Attach({:?}, {})", energy_type, slot);
    }
    if (offset_attach_tool..offset_activate).contains(&id) {
        let val = id - offset_attach_tool;
        let slot = val % 4;
        let card_idx = val / 4;
        let card_id = card_index_to_id(card_idx);
        return format!("AttachTool({:?}, {})", card_id, slot);
    }
    if (offset_activate..offset_draw_card).contains(&id) {
        let val = id - offset_activate;
        return format!("Activate({})", val);
    }
    if (offset_draw_card..offset_move_energy).contains(&id) {
        return "DrawCard".to_string();
    }
    if (offset_move_energy..offset_communicate).contains(&id) {
        let val = id - offset_move_energy;
        let energy_idx = val % 10;
        let bench_idx = (val / 10) + 1;
        let energy_type = index_to_energy_type(energy_idx);
        return format!("MoveEnergy(from:{}, to:0, {:?})", bench_idx, energy_type);
    }
    if (offset_communicate..offset_shuffle_into_deck).contains(&id) {
        let card_idx = id - offset_communicate;
        let card_id = card_index_to_id(card_idx);
        return format!("CommunicatePokemon({:?})", card_id);
    }
    if (offset_shuffle_into_deck..offset_shuffle_op_sup).contains(&id) {
        let card_idx = id - offset_shuffle_into_deck;
        let card_id = card_index_to_id(card_idx);
        return format!("ShufflePokemonIntoDeck({:?})", card_id);
    }
    if (offset_shuffle_op_sup..offset_discard_op_sup).contains(&id) {
        let card_idx = id - offset_shuffle_op_sup;
        let card_id = card_index_to_id(card_idx);
        return format!("ShuffleOpponentSupporter({:?})", card_id);
    }
    if (offset_discard_op_sup..offset_discard_own).contains(&id) {
        let card_idx = id - offset_discard_op_sup;
        let card_id = card_index_to_id(card_idx);
        return format!("DiscardOpponentSupporter({:?})", card_id);
    }
    if (offset_discard_own..offset_attach_from_discard).contains(&id) {
        let card_idx = id - offset_discard_own;
        let card_id = card_index_to_id(card_idx);
        return format!("DiscardOwnCard({:?})", card_id);
    }
    if (offset_attach_from_discard..offset_eevee_boost).contains(&id) {
        let idx = id - offset_attach_from_discard;
        return format!("AttachFromDiscard({})", idx);
    }
    if id == offset_eevee_boost {
        return "ApplyEeveeBagDamageBoost".to_string();
    }
    if id == offset_heal_eevee {
        return "HealAllEeveeEvolutions".to_string();
    }
    if (offset_discard_fossil..offset_heal).contains(&id) {
        let idx = id - offset_discard_fossil;
        return format!("DiscardFossil({})", idx);
    }
    if (offset_heal..offset_move_damage).contains(&id) {
        let idx = id - offset_heal;
        return format!("Heal({})", idx);
    }
    if (offset_move_damage..offset_apply_damage).contains(&id) {
        let val = id - offset_move_damage;
        let to = val % 4;
        let from = val / 4;
        return format!("MoveAllDamage(from:{}, to:{})", from, to);
    }
    if id == offset_apply_damage {
        return "ApplyDamage".to_string();
    }
    let offset_heal_discard = get_offset_heal_discard();
    let offset_return_hand = get_offset_return_hand();
    let offset_use_opp_attack = get_offset_use_opp_attack();
    
    if (offset_heal_discard..offset_return_hand).contains(&id) {
        let idx = id - offset_heal_discard;
        return format!("HealAndDiscardEnergy({})", idx);
    }
    if (offset_return_hand..offset_use_opp_attack).contains(&id) {
        let idx = id - offset_return_hand;
        return format!("ReturnPokemonToHand({})", idx);
    }
    if (offset_use_opp_attack..offset_noop).contains(&id) {
        let idx = id - offset_use_opp_attack;
        return format!("UseOpponentAttack({})", idx);
    }
    if id == offset_noop {
        return "Noop".to_string();
    }

    format!("UnknownAction({})", id)
}

fn energy_type_to_index(e: EnergyType) -> usize {
    match e {
        EnergyType::Grass => 0,
        EnergyType::Fire => 1,
        EnergyType::Water => 2,
        EnergyType::Lightning => 3,
        EnergyType::Psychic => 4,
        EnergyType::Fighting => 5,
        EnergyType::Darkness => 6,
        EnergyType::Metal => 7,
        EnergyType::Dragon => 8,
        EnergyType::Colorless => 9,
    }
}

fn index_to_energy_type(i: usize) -> EnergyType {
    match i {
        0 => EnergyType::Grass,
        1 => EnergyType::Fire,
        2 => EnergyType::Water,
        3 => EnergyType::Lightning,
        4 => EnergyType::Psychic,
        5 => EnergyType::Fighting,
        6 => EnergyType::Darkness,
        7 => EnergyType::Metal,
        8 => EnergyType::Dragon,
        _ => EnergyType::Colorless,
    }
}

fn card_index_to_id(idx: usize) -> Option<CardId> {
    // Iterate to find? Efficient reverse lookup?
    // CardId is an enum with assigned discriminants?
    // EnumIter iterates in order.
    CardId::iter().nth(idx)
}



pub fn encode_observation(state: &State, player: usize) -> Vec<f32> {
    encode_state(state, player, false)
}

pub fn encode_state(state: &State, player: usize, public_only: bool) -> Vec<f32> {
    let mut obs = Vec::new();

    // 1. Turn Info
    obs.push(state.turn_count as f32);
    obs.push(state.points[player] as f32);
    obs.push(state.points[1 - player] as f32);
    obs.push(if state.current_player == player {
        1.0
    } else {
        0.0
    });

    // 1.1 Turn Flags (New)
    obs.push(if state.has_played_support { 1.0 } else { 0.0 });
    obs.push(if state.has_retreated { 1.0 } else { 0.0 });
    obs.push(if state.knocked_out_by_opponent_attack_last_turn { 1.0 } else { 0.0 });

    // 1.2 Energy Info (New)
    let mut current_energy_vec = vec![0.0; ENERGY_TYPES_COUNT];
    if let Some(e) = state.current_energy {
        current_energy_vec[energy_type_to_index(e)] = 1.0;
    }
    obs.extend(current_energy_vec);

    let mut my_next_energy_vec = vec![0.0; ENERGY_TYPES_COUNT];
    if let Some(e) = state.next_energies[player] {
        my_next_energy_vec[energy_type_to_index(e)] = 1.0;
    }
    obs.extend(my_next_energy_vec);

    let mut opp_next_energy_vec = vec![0.0; ENERGY_TYPES_COUNT];
    if let Some(e) = state.next_energies[1 - player] {
        opp_next_energy_vec[energy_type_to_index(e)] = 1.0;
    }
    obs.extend(opp_next_energy_vec);

    // 2. Hand Counts

    // 2. Hand Counts (New)
    obs.push(state.hands[player].len() as f32);
    obs.push(state.hands[1 - player].len() as f32);

    // Helper to encode a pokemon slot

    let encode_pokemon = |pokemon: Option<&PlayedCard>, obs: &mut Vec<f32>| {
        if let Some(p) = pokemon {
            // HP
            obs.push(p.remaining_hp as f32 / 300.0); // Normalize HP
                                                     // Energy
            let mut energies = vec![0.0; ENERGY_TYPES_COUNT];
            for e in &p.attached_energy {
                energies[energy_type_to_index(*e)] += 1.0;
            }
            obs.extend(energies);
            // Card ID (as f32)
            if let Some(cid) = CardId::from_card_id(&p.card.get_id()) {
                obs.push(cid as usize as f32);
            } else {
                obs.push(-1.0);
            }
            // Status
            obs.push(if p.poisoned { 1.0 } else { 0.0 });
            obs.push(if p.asleep { 1.0 } else { 0.0 });
            obs.push(if p.paralyzed { 1.0 } else { 0.0 });
            obs.push(if p.confused { 1.0 } else { 0.0 });
            obs.push(if p.burned { 1.0 } else { 0.0 });

            // New: Pokemon Specific State
            obs.push(if p.played_this_turn { 1.0 } else { 0.0 });
            obs.push(if p.ability_used { 1.0 } else { 0.0 });

            // Attached Tool (ID)
            if let Some(tool_card) = &p.attached_tool {
                 if let Some(cid) = CardId::from_card_id(&tool_card.get_id()) {
                     obs.push(cid as usize as f32);
                 } else {
                     obs.push(-1.0);
                 }
            } else {
                obs.push(-1.0);
            }

            // Key Effects
            let active_effects = p.get_active_effects();
            obs.push(if active_effects.contains(&crate::effects::CardEffect::NoRetreat) { 1.0 } else { 0.0 });
            obs.push(if active_effects.contains(&crate::effects::CardEffect::CannotAttack) { 1.0 } else { 0.0 });
            obs.push(if active_effects.iter().any(|e| matches!(e, crate::effects::CardEffect::ReducedDamage { .. })) { 1.0 } else { 0.0 });
            obs.push(if active_effects.contains(&crate::effects::CardEffect::PreventAllDamageAndEffects) { 1.0 } else { 0.0 });

        } else {
            // Empty slot
            obs.push(0.0); // HP
            obs.extend(vec![0.0; ENERGY_TYPES_COUNT]); // Energy
            obs.push(-1.0); // Card ID
            obs.extend(vec![0.0; 5]); // Status (poisoned, asleep, paralyzed, confused, burned)
            obs.push(0.0); // played_this_turn
            obs.push(0.0); // ability_used
            obs.push(-1.0); // tool
            obs.extend(vec![0.0; 4]); // key effects
        }
    };

    // 3. My Active
    encode_pokemon(state.in_play_pokemon[player][0].as_ref(), &mut obs);

    // 4. My Bench
    for i in 1..4 {
        encode_pokemon(state.in_play_pokemon[player][i].as_ref(), &mut obs);
    }

    // 5. Opponent Active
    encode_pokemon(state.in_play_pokemon[1 - player][0].as_ref(), &mut obs);

    // 6. Opponent Bench
    for i in 1..4 {
        encode_pokemon(state.in_play_pokemon[1 - player][i].as_ref(), &mut obs);
    }

    // 7. Hand Slots (Fixed size: 10)
    let hand_slots_limit = 10;
    let mut hand_slots = vec![-1.0; hand_slots_limit];
    if !public_only {
        for (i, card) in state.hands[player].iter().take(hand_slots_limit).enumerate() {
            if let Some(cid) = CardId::from_card_id(&card.get_id()) {
                hand_slots[i] = cid as usize as f32;
            }
        }
    }
    obs.extend(hand_slots);

    // 8. Opponent Hand (Masked/Empty)
    // Removed opp_hand_vec to save space. We could add slots here if revealed.

    // 9. Deck Count
    obs.push(state.decks[player].cards.len() as f32);
    obs.push(state.decks[1 - player].cards.len() as f32);

    // 9.1 Deck (Bag of Cards) - Removed to save space
    // We keep the deck counts above (lines 595-596).

    // 10. Discard Slots (Fixed size: 10)
    let discard_slots_limit = 10;
    let mut discard_slots = vec![-1.0; discard_slots_limit];
    for (i, card) in state.discard_piles[player].iter().rev().take(discard_slots_limit).enumerate() {
        if let Some(cid) = CardId::from_card_id(&card.get_id()) {
            discard_slots[i] = cid as usize as f32;
        }
    }
    obs.extend(discard_slots);

    // 11. Opponent Discard Slots (Fixed size: 10)
    let mut op_discard_slots = vec![-1.0; discard_slots_limit];
    for (i, card) in state.discard_piles[1 - player].iter().rev().take(discard_slots_limit).enumerate() {
        if let Some(cid) = CardId::from_card_id(&card.get_id()) {
            op_discard_slots[i] = cid as usize as f32;
        }
    }
    obs.extend(op_discard_slots);

    obs
}

pub fn observation_length(state: &State) -> usize {
    encode_state(state, 0, false).len()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::load_test_decks;

    #[test]
    fn test_observation_encoding_expansion() {
        let (deck_a, deck_b) = load_test_decks();
        let mut state = State::new(&deck_a, &deck_b);

        // Initial state
        let obs_initial = encode_observation(&state, 0);
        let len_initial = obs_initial.len();
        assert!(len_initial > 0);

        // Set some flags
        state.set_knocked_out_by_opponent_attack_last_turn(true);
        let obs_with_ko = encode_observation(&state, 0);
        
        // Value should change
        assert_ne!(obs_initial, obs_with_ko);

        // Verify next energy is encoded
        state.next_energies[0] = Some(EnergyType::Fire);
        let obs_with_energy = encode_observation(&state, 0);
        assert_ne!(obs_with_ko, obs_with_energy);
        
        println!("Observation length: {}", len_initial);
    }
}
