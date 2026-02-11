use serde::{Serialize, Deserialize};
use crate::models::EnergyType;
use crate::card_ids::CardId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum TrainerMechanic {
    EvolutionAcceleration {
        stages_to_skip: u8,
        target_scope: TargetScope,
    },
    Heal {
        amount: u32,
        cure_status: bool,
        target_scope: TargetScope,
    },
    Draw {
        amount: u32,
        shuffle_hand_first: bool,
    },
    EnergyAttachment {
        amount: u32,
        energy_type: Option<EnergyType>,
        from_zone: String, // "Deck", "Discard", "EnergyZone", "Field"
        target_scope: TargetScope,
    },
    Switch {
        target: TargetScope,
        forced: bool,
    },
    Search {
        target_type: String,
        amount: u32,
        from_zone: String, // "Deck", "Discard"
    },
    DamageBoost {
        amount: u32,
        target_scope: TargetScope,
        against_ex: bool,
        specific_pokemon: Option<Vec<String>>,
    },
    RetreatCostReduction {
        amount: u32,
    },
    ShuffleHandInDraw {
        amount_type: String, // "fixed_3", "opponent_points", "opponent_hand_size", "hand_size"
    },
    AttachTool,
    PlaceFossil,
    MultiEffect {
        effects: Vec<TrainerMechanic>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetScope {
    SelfActive,
    SelfBench,
    SelfBoard,
    OpponentActive,
    OpponentBench,
    OpponentBoard,
}

impl CardId {
    pub fn get_trainer_mechanic(&self) -> Option<TrainerMechanic> {
        match self {
            // --- Healing ---
            CardId::PA001Potion => Some(TrainerMechanic::Heal { amount: 20, cure_status: false, target_scope: TargetScope::SelfBoard }),
            CardId::A1219Erika | CardId::A1266Erika | CardId::A4b328Erika | CardId::A4b329Erika => Some(TrainerMechanic::Heal { amount: 50, cure_status: false, target_scope: TargetScope::SelfBoard }),
            CardId::A2a072Irida | CardId::A2a087Irida | CardId::A4b330Irida | CardId::A4b331Irida => Some(TrainerMechanic::Heal { amount: 40, cure_status: false, target_scope: TargetScope::SelfBoard }),
            CardId::A2b070PokemonCenterLady | CardId::A2b089PokemonCenterLady => Some(TrainerMechanic::Heal { amount: 30, cure_status: true, target_scope: TargetScope::SelfBoard }),
            CardId::A3155Lillie | CardId::A3197Lillie | CardId::A3209Lillie | CardId::A4b348Lillie | CardId::A4b349Lillie | CardId::A4b374Lillie => Some(TrainerMechanic::Heal { amount: 60, cure_status: false, target_scope: TargetScope::SelfBoard }),

            // --- Retreat ---
            CardId::PA002XSpeed => Some(TrainerMechanic::RetreatCostReduction { amount: 1 }),
            CardId::A1a068Leaf | CardId::A1a082Leaf | CardId::A4b346Leaf | CardId::A4b347Leaf => Some(TrainerMechanic::RetreatCostReduction { amount: 2 }),

            // --- Search ---
            CardId::PA005PokeBall | CardId::A2b111PokeBall => Some(TrainerMechanic::Search { target_type: "Pokemon".to_string(), amount: 1, from_zone: "Deck".to_string() }),
            CardId::A2146PokemonCommunication | CardId::A4b316PokemonCommunication | CardId::A4b317PokemonCommunication => Some(TrainerMechanic::Search { target_type: "Pokemon".to_string(), amount: 1, from_zone: "Deck".to_string() }),
            CardId::A3a067Gladion | CardId::A3a081Gladion => Some(TrainerMechanic::Search { target_type: "SilvallyLine".to_string(), amount: 1, from_zone: "Deck".to_string() }),
            CardId::B1223May | CardId::B1268May => Some(TrainerMechanic::Search { target_type: "Pokemon".to_string(), amount: 2, from_zone: "Deck".to_string() }),
            CardId::B1226Lisia | CardId::B1271Lisia => Some(TrainerMechanic::Search { target_type: "BasicPokemonLE50HP".to_string(), amount: 2, from_zone: "Deck".to_string() }),
            CardId::A2a073CelesticTownElder | CardId::A2a088CelesticTownElder => Some(TrainerMechanic::Search { target_type: "BasicPokemon".to_string(), amount: 1, from_zone: "Discard".to_string() }),
            CardId::B1a068Clemont | CardId::B1a081Clemont => Some(TrainerMechanic::Search { target_type: "ElectricSupport".to_string(), amount: 2, from_zone: "Deck".to_string() }),
            CardId::B1a069Serena | CardId::B1a082Serena => Some(TrainerMechanic::Search { target_type: "MegaPokemon".to_string(), amount: 1, from_zone: "Deck".to_string() }),

            // --- Draw ---
            CardId::PA007ProfessorsResearch | CardId::A4b373ProfessorsResearch => Some(TrainerMechanic::Draw { amount: 2, shuffle_hand_first: false }),
            CardId::A1a065MythicalSlab => Some(TrainerMechanic::Draw { amount: 1, shuffle_hand_first: false }),

            // --- Hand / Deck Shuffle ---
            CardId::PA006RedCard => Some(TrainerMechanic::ShuffleHandInDraw { amount_type: "fixed_3".to_string() }),
            CardId::A2155Mars | CardId::A2195Mars | CardId::A4b344Mars | CardId::A4b345Mars => Some(TrainerMechanic::ShuffleHandInDraw { amount_type: "opponent_points".to_string() }),
            CardId::B1225Copycat | CardId::B1270Copycat => Some(TrainerMechanic::ShuffleHandInDraw { amount_type: "opponent_hand_size".to_string() }),
            CardId::A2b069Iono | CardId::A2b088Iono | CardId::A4b340Iono | CardId::A4b341Iono => Some(TrainerMechanic::ShuffleHandInDraw { amount_type: "hand_size".to_string() }),

            // --- Damage Boost ---
            CardId::A1223Giovanni | CardId::A1270Giovanni | CardId::A4b334Giovanni | CardId::A4b335Giovanni => Some(TrainerMechanic::DamageBoost { amount: 10, target_scope: TargetScope::SelfActive, against_ex: false, specific_pokemon: None }),
            CardId::A1221Blaine | CardId::A1268Blaine => Some(TrainerMechanic::DamageBoost { amount: 30, target_scope: TargetScope::SelfActive, against_ex: false, specific_pokemon: Some(vec!["Ninetales".to_string(), "Rapidash".to_string(), "Magmar".to_string()]) }),
            CardId::A2b071Red | CardId::A2b090Red | CardId::A4b352Red | CardId::A4b353Red => Some(TrainerMechanic::DamageBoost { amount: 20, target_scope: TargetScope::SelfActive, against_ex: true, specific_pokemon: None }),
            CardId::B1a066ClemontsBackpack => Some(TrainerMechanic::DamageBoost { amount: 20, target_scope: TargetScope::SelfActive, against_ex: false, specific_pokemon: Some(vec!["Magneton".to_string(), "Heliolisk".to_string()]) }),

            // --- Energy Attachment ---
            CardId::A1220Misty | CardId::A1267Misty => Some(TrainerMechanic::EnergyAttachment { amount: 1, energy_type: Some(EnergyType::Water), from_zone: "EnergyZone".to_string(), target_scope: TargetScope::SelfBoard }),
            CardId::A1224Brock | CardId::A1271Brock => Some(TrainerMechanic::EnergyAttachment { amount: 1, energy_type: Some(EnergyType::Fighting), from_zone: "EnergyZone".to_string(), target_scope: TargetScope::SelfBoard }),
            CardId::A3a069Lusamine | CardId::A3a083Lusamine | CardId::A4b350Lusamine | CardId::A4b351Lusamine | CardId::A4b375Lusamine => Some(TrainerMechanic::EnergyAttachment { amount: 2, energy_type: None, from_zone: "Discard".to_string(), target_scope: TargetScope::SelfBoard }),
            CardId::A4151ElementalSwitch | CardId::A4b310ElementalSwitch | CardId::A4b311ElementalSwitch => Some(TrainerMechanic::EnergyAttachment { amount: 1, energy_type: None, from_zone: "Field".to_string(), target_scope: TargetScope::SelfActive }),
            CardId::B1217FlamePatch | CardId::B1331FlamePatch => Some(TrainerMechanic::EnergyAttachment { amount: 1, energy_type: Some(EnergyType::Fire), from_zone: "Discard".to_string(), target_scope: TargetScope::SelfActive }),
            CardId::B1224Fantina | CardId::B1269Fantina => Some(TrainerMechanic::EnergyAttachment { amount: 1, energy_type: Some(EnergyType::Psychic), from_zone: "EnergyZone".to_string(), target_scope: TargetScope::SelfBoard }),

            // --- Switch / Forced Switch ---
            CardId::A1225Sabrina | CardId::A1272Sabrina | CardId::A4b338Sabrina | CardId::A4b339Sabrina => Some(TrainerMechanic::Switch { target: TargetScope::OpponentActive, forced: true }),
            CardId::A3a064Repel => Some(TrainerMechanic::Switch { target: TargetScope::OpponentActive, forced: true }),
            CardId::A2150Cyrus | CardId::A2190Cyrus | CardId::A4b326Cyrus | CardId::A4b327Cyrus => Some(TrainerMechanic::Switch { target: TargetScope::OpponentBench, forced: true }),
            CardId::A1222Koga | CardId::A1269Koga => Some(TrainerMechanic::Switch { target: TargetScope::SelfActive, forced: false }),
            CardId::A4157Lyra | CardId::A4197Lyra | CardId::A4b332Lyra | CardId::A4b333Lyra => Some(TrainerMechanic::Switch { target: TargetScope::SelfActive, forced: false }),

            // --- Evolution ---
            CardId::A3144RareCandy | CardId::A4b314RareCandy | CardId::A4b315RareCandy | CardId::A4b379RareCandy => Some(TrainerMechanic::EvolutionAcceleration { stages_to_skip: 1, target_scope: TargetScope::SelfBoard }),
            CardId::B1a067QuickGrowExtract | CardId::B1a103QuickGrowExtract => Some(TrainerMechanic::EvolutionAcceleration { stages_to_skip: 0, target_scope: TargetScope::SelfBoard }),

            // --- Tools ---
            CardId::A2147GiantCape | CardId::A2148RockyHelmet | CardId::A3146PoisonBarb | CardId::A3147LeafCape | CardId::A3a065ElectricalCord | CardId::A4a067InflatableBoat | CardId::A4b318ElectricalCord | CardId::A4b319ElectricalCord | CardId::A4b320GiantCape | CardId::A4b321GiantCape | CardId::A4b322RockyHelmet | CardId::A4b323RockyHelmet | CardId::B1219HeavyHelmet => Some(TrainerMechanic::AttachTool),

            // --- Fossils ---
            CardId::A1216HelixFossil | CardId::A1217DomeFossil | CardId::A1218OldAmber | CardId::A1a063OldAmber | CardId::A2144SkullFossil | CardId::A2145ArmorFossil | CardId::A4b312OldAmber | CardId::A4b313OldAmber | CardId::B1214PlumeFossil | CardId::B1216CoverFossil => Some(TrainerMechanic::PlaceFossil),

            // --- MultiEffect ---
            CardId::A3b066EeveeBag | CardId::A3b107EeveeBag | CardId::A4b308EeveeBag | CardId::A4b309EeveeBag => Some(TrainerMechanic::MultiEffect { effects: vec![
                TrainerMechanic::DamageBoost { amount: 20, target_scope: TargetScope::SelfActive, against_ex: false, specific_pokemon: None },
                TrainerMechanic::Heal { amount: 100, cure_status: false, target_scope: TargetScope::SelfBoard },
            ]}),

            _ => None,
        }
    }
}
