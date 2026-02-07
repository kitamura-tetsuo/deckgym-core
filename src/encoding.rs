use crate::{
    actions::SimpleAction,
    card_ids::CardId,
    models::{EnergyType, PlayedCard},
    state::State,
    tool_ids::ToolId,
};
use strum::IntoEnumIterator;

pub const MAX_BENCH_SIZE: usize = 3;
pub const MAX_IN_PLAY: usize = 1 + MAX_BENCH_SIZE; // 1 active + 3 bench
pub const ENERGY_TYPES_COUNT: usize = 10; // Grass, Fire, ..., Colorless

pub fn get_card_count() -> usize {
    CardId::iter().len()
}

pub fn get_action_space_size() -> usize {
    let card_count = get_card_count();
    let tool_count = ToolId::iter().len();
    // EndTurn: 1
    // Attack: 3
    // Retreat: 4
    // UseAbility: 4
    // Place: card_count * 4
    // Evolve: card_count * 4
    // Play: card_count
    // Attach: 10 * 4
    // AttachTool: tool_count * 4

    12 + card_count * 9 + 40 + tool_count * 4
}

// Action Offsets
const OFFSET_END_TURN: usize = 0;
const OFFSET_ATTACK: usize = 1;
const OFFSET_RETREAT: usize = 4;
const OFFSET_USE_ABILITY: usize = 8;
const OFFSET_PLACE: usize = 12;

fn get_offset_evolve() -> usize {
    OFFSET_PLACE + get_card_count() * 4
}

fn get_offset_play() -> usize {
    get_offset_evolve() + get_card_count() * 4
}

fn get_offset_attach() -> usize {
    get_offset_play() + get_card_count()
}

fn get_offset_attach_tool() -> usize {
    get_offset_attach() + ENERGY_TYPES_COUNT * 4
}

pub fn encode_action(action: &SimpleAction) -> Option<usize> {
    match action {
        SimpleAction::EndTurn => Some(OFFSET_END_TURN),
        SimpleAction::Attack(idx) => {
            if *idx < 3 {
                Some(OFFSET_ATTACK + idx)
            } else {
                None
            }
        },
        SimpleAction::Retreat(idx) => {
            if *idx < 4 {
                Some(OFFSET_RETREAT + idx)
            } else {
                None
            }
        },
        SimpleAction::UseAbility { in_play_idx } => {
            if *in_play_idx < 4 {
                Some(OFFSET_USE_ABILITY + in_play_idx)
            } else {
                None
            }
        },
        SimpleAction::Place(card, slot) => {
            if *slot < 4 {
                let card_id = CardId::from_card_id(&card.get_id())?;
                let card_idx = card_id as usize;
                Some(OFFSET_PLACE + card_idx * 4 + slot)
            } else {
                None
            }
        },
        SimpleAction::Evolve { evolution, in_play_idx, .. } => {
            if *in_play_idx < 4 {
                let card_id = CardId::from_card_id(&evolution.get_id())?;
                let card_idx = card_id as usize;
                Some(get_offset_evolve() + card_idx * 4 + in_play_idx)
            } else {
                None
            }
        },
        SimpleAction::Play { trainer_card } => {
            let card_id = CardId::from_card_id(&trainer_card.id)?;
            let card_idx = card_id as usize;
            Some(get_offset_play() + card_idx)
        },
        SimpleAction::Attach { attachments, .. } => {
            // Assume single attachment for now as atomic action
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
        },
        SimpleAction::AttachTool { in_play_idx, tool_id } => {
             // Map ToolId to index. ToolId is not EnumIter but derived from database or manually managed?
             // src/tool_ids.rs is manual.
             // I'll just hash it or use a manual mapping if small.
             // Or use CardId?
             // The action uses ToolId.
             // I'll implement a simple mapping for known tools.
             let tool_idx = tool_id_to_index(*tool_id);
             if *in_play_idx < 4 {
                 Some(get_offset_attach_tool() + tool_idx * 4 + in_play_idx)
             } else {
                 None
             }
        },
        _ => None // Other actions not mapped yet
    }
}

pub fn action_name(id: usize) -> String {
    let offset_evolve = get_offset_evolve();
    let offset_play = get_offset_play();
    let offset_attach = get_offset_attach();
    let offset_attach_tool = get_offset_attach_tool();
    let total_size = get_action_space_size();

    if id == OFFSET_END_TURN {
        return "EndTurn".to_string();
    }
    if id >= OFFSET_ATTACK && id < OFFSET_RETREAT {
        return format!("Attack({})", id - OFFSET_ATTACK);
    }
    if id >= OFFSET_RETREAT && id < OFFSET_USE_ABILITY {
        return format!("Retreat({})", id - OFFSET_RETREAT);
    }
    if id >= OFFSET_USE_ABILITY && id < OFFSET_PLACE {
        return format!("UseAbility({})", id - OFFSET_USE_ABILITY);
    }
    if id >= OFFSET_PLACE && id < offset_evolve {
        let val = id - OFFSET_PLACE;
        let slot = val % 4;
        let card_idx = val / 4;
        let card_id = card_index_to_id(card_idx);
        return format!("Place({:?}, {})", card_id, slot);
    }
    if id >= offset_evolve && id < offset_play {
        let val = id - offset_evolve;
        let slot = val % 4;
        let card_idx = val / 4;
        let card_id = card_index_to_id(card_idx);
        return format!("Evolve({:?}, {})", card_id, slot);
    }
    if id >= offset_play && id < offset_attach {
        let card_idx = id - offset_play;
        let card_id = card_index_to_id(card_idx);
        return format!("Play({:?})", card_id);
    }
    if id >= offset_attach && id < offset_attach_tool {
        let val = id - offset_attach;
        let slot = val % 4;
        let energy_idx = val / 4;
        let energy_type = index_to_energy_type(energy_idx);
        return format!("Attach({:?}, {})", energy_type, slot);
    }
    if id >= offset_attach_tool && id < total_size {
        let val = id - offset_attach_tool;
        let slot = val % 4;
        let tool_idx = val / 4;
        let tool_id = ToolId::iter()
            .nth(tool_idx)
            .expect("Tool index should be valid");
        return format!("AttachTool({:?}, {})", tool_id, slot);
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

fn tool_id_to_index(t: ToolId) -> usize {
    ToolId::iter()
        .position(|x| x == t)
        .expect("ToolId should be in iter")
}

pub fn encode_observation(state: &State, player: usize) -> Vec<f32> {
    let mut obs = Vec::new();
    let card_count = get_card_count();

    // 1. Turn Info
    obs.push(state.turn_count as f32);
    obs.push(state.points[player] as f32);
    obs.push(state.points[1 - player] as f32);
    obs.push(if state.current_player == player { 1.0 } else { 0.0 });

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
            // Card ID One-Hot
            let mut card_vec = vec![0.0; card_count];
            if let Some(cid) = CardId::from_card_id(&p.card.get_id()) {
                card_vec[cid as usize] = 1.0;
            }
            obs.extend(card_vec);
            // Status
            obs.push(if p.poisoned { 1.0 } else { 0.0 });
            obs.push(if p.asleep { 1.0 } else { 0.0 });
            obs.push(if p.paralyzed { 1.0 } else { 0.0 });
            obs.push(if p.confused { 1.0 } else { 0.0 }); // If confused exists
            // TODO: other status?
        } else {
            // Empty slot
            obs.push(0.0); // HP
            obs.extend(vec![0.0; ENERGY_TYPES_COUNT]); // Energy
            obs.extend(vec![0.0; card_count]); // Card ID
            obs.extend(vec![0.0; 4]); // Status
        }
    };

    // 2. My Active
    encode_pokemon(state.in_play_pokemon[player][0].as_ref(), &mut obs);

    // 3. My Bench
    for i in 1..4 {
        encode_pokemon(state.in_play_pokemon[player][i].as_ref(), &mut obs);
    }

    // 4. Opponent Active
    encode_pokemon(state.in_play_pokemon[1 - player][0].as_ref(), &mut obs);

    // 5. Opponent Bench
    for i in 1..4 {
        encode_pokemon(state.in_play_pokemon[1 - player][i].as_ref(), &mut obs);
    }

    // 6. Hand (Bag of Cards)
    let mut hand_vec = vec![0.0; card_count];
    for card in &state.hands[player] {
        if let Some(cid) = CardId::from_card_id(&card.get_id()) {
            hand_vec[cid as usize] += 1.0;
        }
    }
    obs.extend(hand_vec);

    // 7. Deck Count
    obs.push(state.decks[player].cards.len() as f32);
    obs.push(state.decks[1 - player].cards.len() as f32); // Is opponent deck size visible? Yes.

    // 8. Discard (Bag of Cards)
    let mut discard_vec = vec![0.0; card_count];
    for card in &state.discard_piles[player] {
        if let Some(cid) = CardId::from_card_id(&card.get_id()) {
            discard_vec[cid as usize] += 1.0;
        }
    }
    obs.extend(discard_vec);

    // Opponent discard?
    let mut op_discard_vec = vec![0.0; card_count];
    for card in &state.discard_piles[1 - player] {
        if let Some(cid) = CardId::from_card_id(&card.get_id()) {
            op_discard_vec[cid as usize] += 1.0;
        }
    }
    obs.extend(op_discard_vec);

    obs
}
