use crate::models::{Card, EnergyType, StatusCondition};
use crate::ability_ids::AbilityId;
use super::mechanic::{AbilityMechanic, TargetScope};

pub fn get_ability_mechanic(card: &Card) -> Option<AbilityMechanic> {
    let card_id = card.get_id();
    let ability_id = AbilityId::from_pokemon_id(&card_id[..])?;

    match ability_id {
        AbilityId::A1020VictreebelFragranceTrap | AbilityId::A1188PidgeotDriveOff | AbilityId::A4112UmbreonExDarkChase => 
            Some(AbilityMechanic::ForcedSwitchActive),
        AbilityId::A1098MagnetonVoltCharge | AbilityId::A2a010LeafeonExForestBreath => 
            Some(AbilityMechanic::EnergyAttachment { 
                amount: 1, 
                energy_type: if matches!(ability_id, AbilityId::A1098MagnetonVoltCharge) { Some(EnergyType::Lightning) } else { Some(EnergyType::Grass) }, 
                from_zone: "EnergyZone".to_string(), 
                target: TargetScope::SelfBoard 
            }),
        AbilityId::A1177Weezing => 
            Some(AbilityMechanic::ApplyStatus { condition: StatusCondition::Poisoned, target: TargetScope::OpponentActive }),
        AbilityId::A1132Gardevoir => 
            Some(AbilityMechanic::EnergyAttachment { amount: 1, energy_type: Some(EnergyType::Psychic), from_zone: "EnergyZone".to_string(), target: TargetScope::SelfActive }),
        AbilityId::A1a019VaporeonWashOut => 
            Some(AbilityMechanic::MoveEnergy { amount: 1, energy_type: EnergyType::Water, from: TargetScope::SelfBench, to: TargetScope::SelfActive }),
        AbilityId::A2072DusknoirShadowVoid => 
            Some(AbilityMechanic::MoveDamage { amount: None, from: TargetScope::SelfBoard, to: TargetScope::SelfActive }), // Placeholder
        AbilityId::A2b035GiratinaExBrokenSpaceBellow => 
            Some(AbilityMechanic::ChargeSelfAndEndTurn { energy_type: EnergyType::Psychic, amount: 1 }),
        AbilityId::A3122SolgaleoExRisingRoad => 
            Some(AbilityMechanic::SwitchSelfWithBench),
        AbilityId::A3a027ShiinoticIlluminate => 
            Some(AbilityMechanic::Search { target_type: "Pokemon".to_string(), amount: 1, from_zone: "Deck".to_string() }),
        AbilityId::A3a062CelesteelaUltraThrusters => 
            Some(AbilityMechanic::SwitchSelfWithBench), // Restricted to Ultra Beast in logic, but simplified here
        AbilityId::A3b009FlareonExCombust => 
            Some(AbilityMechanic::EnergyAttachment { amount: 1, energy_type: Some(EnergyType::Fire), from_zone: "Discard".to_string(), target: TargetScope::SelfBoard }),
        AbilityId::A4083EspeonExPsychicHealing | AbilityId::B1121IndeedeeExWatchOver => 
            Some(AbilityMechanic::Heal { amount: if matches!(ability_id, AbilityId::A4083EspeonExPsychicHealing) { 30 } else { 20 }, target: TargetScope::SelfBoard }),
        AbilityId::A2a050CrobatCunningLink => 
            Some(AbilityMechanic::DamageOpponent { amount: 30, target: TargetScope::OpponentActive }),
        AbilityId::B1157HydreigonRoarInUnison => 
            Some(AbilityMechanic::EnergyAttachment { amount: 2, energy_type: Some(EnergyType::Darkness), from_zone: "EnergyZone".to_string(), target: TargetScope::SelfBoard }),
        AbilityId::A2110DarkraiExNightmareAura => 
            Some(AbilityMechanic::OnEnergyAttachDamage { energy_type: EnergyType::Darkness, amount: 20, from_zone: "EnergyZone".to_string(), target: TargetScope::OpponentActive }),
        AbilityId::A2a035RotomSpeedLink => 
            Some(AbilityMechanic::NoRetreatCost),
        AbilityId::A3066OricoricSafeguard => 
            Some(AbilityMechanic::Safeguard),
        AbilityId::A3a042NihilegoMorePoison => 
            Some(AbilityMechanic::IncreasedPoisonDamage { amount: 10 }),
        _ => None,
    }
}
