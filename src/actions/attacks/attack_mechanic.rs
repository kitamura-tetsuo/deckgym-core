use crate::models::{Card, EnergyType};
use crate::attack_ids::AttackId;
use super::mechanic::{Mechanic, TargetScope};

pub fn get_attack_mechanic(card: &Card, attack_index: usize) -> Option<Mechanic> {
    let card_id = card.get_id();
    let attack_id = AttackId::from_pokemon_index(&card_id[..], attack_index)?;

    match attack_id {
        AttackId::A1115AbraTeleport | AttackId::A1a017MagikarpLeapOut | AttackId::A4a021FeebasLeapOut | AttackId::A3085CosmogTeleport => 
            Some(Mechanic::SwitchSelfWithBench),
        AttackId::A1136GolurkDoubleLariat => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![0, 100, 200] }),
        AttackId::A1149GolemDoubleEdge => 
            Some(Mechanic::SelfDamage { amount: 50 }),
        AttackId::A1153MarowakExBonemerang => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![0, 80, 160] }),
        AttackId::A1163GrapploctKnockBack => 
            Some(Mechanic::KnockBack),
        AttackId::A1178MawileCrunch => 
            Some(Mechanic::DiscardEnergyFromOpponentActive),
        AttackId::A1181MeltanAmass | AttackId::A1a001ExeggcuteGrowthSpurt | AttackId::A2023MagmarStoke | AttackId::A2056ElectabuzzCharge | AttackId::A3040AlolanVulpixCallForthCold | AttackId::A3071SpoinkPsycharge => {
            let energy_type = match attack_id {
                AttackId::A1181MeltanAmass => EnergyType::Metal,
                AttackId::A1a001ExeggcuteGrowthSpurt => EnergyType::Grass,
                AttackId::A2023MagmarStoke => EnergyType::Fire,
                AttackId::A2056ElectabuzzCharge => EnergyType::Lightning,
                AttackId::A3040AlolanVulpixCallForthCold => EnergyType::Water,
                AttackId::A3071SpoinkPsycharge => EnergyType::Psychic,
                _ => EnergyType::Colorless,
            };
            Some(Mechanic::ChargeFromEnergyZone { energy_type, count: 1, target: TargetScope::SelfActive })
        },
        AttackId::A1196MeowthPayDay | AttackId::A3b055EeveeCollect => 
            Some(Mechanic::DrawAndDamage { draw_count: 1 }),
        AttackId::A1201LickitungContinuousLick | AttackId::A1a061EeveeContinuousSteps => 
            Some(Mechanic::FlipUntilTailsDamage { damage_per_head: if matches!(attack_id, AttackId::A1201LickitungContinuousLick) { 60 } else { 20 } }),
        AttackId::A1203KangaskhanDizzyPunch => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![0, 30, 60] }),
        AttackId::A1a010PonytaStomp => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.5, 0.5], damages: vec![10, 40] }),
        AttackId::A1a011RapidashRisingLunge => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.5, 0.5], damages: vec![40, 100] }),
        AttackId::A1a026RaichuGigashock => 
            Some(Mechanic::AlsoBenchDamage { opponent: true, damage: 20, must_have_energy: false }),
        AttackId::A2029InfernapeExFlareBlitz => 
            Some(Mechanic::DiscardAllEnergyOfType { energy_type: EnergyType::Fire }),
        AttackId::A2060LuxrayVoltBolt => 
            Some(Mechanic::AlsoChoiceBenchDamage { opponent: true, damage: 100 }), // Fix damage if needed,Luxray does 100 to any
        AttackId::A2084GliscorAcrobatics => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![20, 40, 60] }),
        AttackId::A2098SneaselDoubleScratch | AttackId::A3b058AipomDoubleHit | AttackId::A3a044Poipole2Step => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![0, 20, 40] }),
        AttackId::A2118ProbopassTripleNose => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.125, 0.375, 0.375, 0.125], damages: vec![30, 80, 130, 180] }),
        AttackId::A2131AmbipomDoubleHit => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![0, 40, 80] }),
        AttackId::A2141ChatotFuryAttack => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.125, 0.375, 0.375, 0.125], damages: vec![0, 20, 40, 60] }),
        AttackId::A2a001HeracrossSingleHornThrow => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.75], damages: vec![120, 50] }),
        AttackId::A2a063SnorlaxCollapse | AttackId::A3b057SnorlaxExFlopDownPunch => 
            Some(Mechanic::InflictStatusConditions { conditions: vec![crate::models::StatusCondition::Asleep], target_opponent: false }),
        AttackId::A2b032MrMimeJuggling | AttackId::A3116ToxapexSpikeCannon => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.0625, 0.25, 0.375, 0.25, 0.0625], damages: vec![0, 20, 40, 60, 80] }),
        AttackId::A2b044FlamigoDoubleKick | AttackId::A3b020VanilluxeDoubleSpin => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![0, 50, 100] }),
        AttackId::A3002AlolanExeggutorTropicalHammer => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.5, 0.5], damages: vec![0, 150] }),
        AttackId::A3012DecidueyeExPierceThePain => 
            Some(Mechanic::DirectDamageIfDamaged { damage: 100 }),
        AttackId::A3019SteeneeDoubleSpin => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![0, 30, 60] }),
        AttackId::A3020TsareenaThreeKickCombo => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.125, 0.375, 0.375, 0.125], damages: vec![0, 50, 100, 150] }),
        AttackId::A3a003RowletFuryAttack => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.125, 0.375, 0.375, 0.125], damages: vec![0, 10, 20, 30] }),
        AttackId::A3a019TapuKokoExPlasmaHurricane => 
            Some(Mechanic::ChargeFromEnergyZone { energy_type: EnergyType::Lightning, count: 1, target: TargetScope::SelfActive }),
        AttackId::A3a047AlolanDugtrioExTripletHeadbutt => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.125, 0.375, 0.375, 0.125], damages: vec![0, 60, 120, 180] }),
        AttackId::A3a060TypeNullQuickBlow => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.5, 0.5], damages: vec![20, 40] }),
        AttackId::A3a061SilvallyBraveBuddies => 
            Some(Mechanic::ExtraDamageIfCondition { extra_damage: 60, condition: "SupporterUsed".to_string() }),
        AttackId::A3a062CelesteelaMoombahton => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.5, 0.5], damages: vec![0, 100] }),
        AttackId::A3b013IncineroarDarkestLariat => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![0, 100, 200] }),
        AttackId::A4021ShuckleExTripleSlap => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.125, 0.375, 0.375, 0.125], damages: vec![0, 20, 40, 60] }),
        AttackId::A4032MagbyToastyToss | AttackId::A4066PichuCracklyToss | AttackId::A4a023MantykeSplashyToss => {
            let energy_type = match attack_id {
                AttackId::A4032MagbyToastyToss => EnergyType::Fire,
                AttackId::A4066PichuCracklyToss => EnergyType::Lightning,
                AttackId::A4a023MantykeSplashyToss => EnergyType::Water,
                _ => EnergyType::Colorless,
            };
            Some(Mechanic::ChargeFromEnergyZone { energy_type, count: 1, target: TargetScope::SelfBench })
        },
        AttackId::A4077CleffaTwinklyCall | AttackId::A4134EeveeFindAFriend => 
            Some(Mechanic::SearchToHandByEnergy { energy_type: EnergyType::Colorless }), // Placeholder for Search
        AttackId::A4105BinacleDualChop => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![0, 30, 60] }),
        AttackId::A4146UrsaringSwingAround => 
            Some(Mechanic::ProbabilisticDamage { probs: vec![0.25, 0.5, 0.25], damages: vec![60, 80, 100] }),
        AttackId::A3112AbsolUnseenClaw => 
            Some(Mechanic::ExtraDamageIfCondition { extra_damage: 70, condition: "OpponentHandLow".to_string() }),
        AttackId::B1052MegaGyaradosExMegaBlaster => 
            Some(Mechanic::DiscardOpponentDeck { count: 3 }),
        AttackId::B1101SableyeDirtyThrow => 
            Some(Mechanic::ExtraDamageIfCondition { extra_damage: 60, condition: "OpponentHasSupporterInDiscard".to_string() }),
        AttackId::B1150AbsolOminousClaw => 
            Some(Mechanic::ExtraDamageIfCondition { extra_damage: 40, condition: "OpponentDamaged".to_string() }),
        AttackId::B1151MegaAbsolExDarknessClaw => 
            Some(Mechanic::ExtraDamageIfCondition { extra_damage: 80, condition: "OpponentHandHigh".to_string() }),
        _ => None,
    }
}
