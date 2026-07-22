/// Cosmetic tiers, indexed by rank. Rank 0 is the unadorned starting state;
/// every later tier only changes cosmetics, never behavior.
pub const TIERS: [&str; 14] = [
    "naked", "plastic", "wood", "iron", "bronze", "silver", "gold", "platinum", "emerald",
    "diamond", "master", "legend", "champion", "godlike",
];

/// Level required to *reach* the same-index rank. Gates widen so late tiers
/// stay aspirational even though the XP curve is already cubic.
pub const GATES: [u32; 14] = [0, 5, 10, 15, 21, 28, 36, 45, 55, 66, 78, 91, 105, 120];

pub const TOKENS_PER_XP: u64 = 1000;

/// Total XP required to reach `level`: floor(0.8 · L³ · 1.5^prestige), as the
/// exact integer form `4·L³·3^p / (5·2^p)`. u128 intermediates because
/// `3^p · L³` overflows u64 long before the inputs look unreasonable.
pub fn xp_for_level(level: u32, prestige: u32) -> u64 {
    if level <= 1 {
        return 0;
    }
    let l = level as u128;
    let p = prestige.min(40); // 1.5^40 already dwarfs any real token count
    let num = 4u128
        .saturating_mul(l * l * l)
        .saturating_mul(3u128.saturating_pow(p));
    let den = 5u128 * 2u128.saturating_pow(p);
    u64::try_from(num / den).unwrap_or(u64::MAX)
}

/// Largest level whose threshold is within `xp`. Linear walk: the 999 cap
/// bounds it, and the curve makes real values land in the low hundreds.
pub fn level_for_xp(xp: u64, prestige: u32) -> u32 {
    let mut level = 1;
    while level < 999 && xp_for_level(level + 1, prestige) <= xp {
        level += 1;
    }
    level
}

/// Whether the next rank's gate is met. Rank never auto-advances; this only
/// gates the manual Rank Up action.
pub fn rank_up_eligible(level: u32, rank: usize) -> bool {
    rank < TIERS.len() - 1 && level >= GATES[rank + 1]
}

/// Prestige is offered only at the final tier.
pub fn prestige_eligible(rank: usize) -> bool {
    rank == TIERS.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_matches_cubic_fast_formula() {
        assert_eq!(xp_for_level(1, 0), 0);
        assert_eq!(xp_for_level(2, 0), 6); // floor(0.8*8)
        assert_eq!(xp_for_level(10, 0), 800);
        assert_eq!(xp_for_level(120, 0), 1_382_400);
    }

    #[test]
    fn prestige_steepens_curve_by_1_5x_each_cycle() {
        assert_eq!(xp_for_level(10, 1), 1200); // 800 * 1.5
        assert_eq!(xp_for_level(10, 2), 1800);
        assert_eq!(xp_for_level(10, 4), 4050);
    }

    #[test]
    fn level_for_xp_inverts_curve() {
        assert_eq!(level_for_xp(0, 0), 1);
        assert_eq!(level_for_xp(5, 0), 1);
        assert_eq!(level_for_xp(6, 0), 2);
        assert_eq!(level_for_xp(799, 0), 9);
        assert_eq!(level_for_xp(800, 0), 10);
        assert_eq!(level_for_xp(1199, 1), 9);
        assert_eq!(level_for_xp(1200, 1), 10);
    }

    #[test]
    fn gates_align_with_tiers() {
        assert_eq!(TIERS.len(), 14);
        assert_eq!(GATES.len(), 14);
        assert_eq!(TIERS[0], "naked");
        assert_eq!(TIERS[13], "godlike");
        assert_eq!(GATES[13], 120);
        assert!(GATES.windows(2).all(|w| w[0] < w[1] || w[0] == 0));
    }

    #[test]
    fn eligibility_rules() {
        assert!(!rank_up_eligible(4, 0));
        assert!(rank_up_eligible(5, 0));
        assert!(rank_up_eligible(200, 0));
        assert!(!rank_up_eligible(200, 13)); // godlike: no more ranks
        assert!(!prestige_eligible(12));
        assert!(prestige_eligible(13));
    }
}
