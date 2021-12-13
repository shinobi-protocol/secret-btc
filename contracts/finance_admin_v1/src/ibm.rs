///
/// initial bonus mining
///
///
use shared_types::E8;

const INITIAL_REWARD_RATE_E8: u128 = 33_000_000_000_000; // 330000 SNB per 1 BTC
const PHASE_MULTIPLIER_E8: u128 = 80_000_000; // 0.8
const PHASE_ZERO_MAX: u128 = 7000_000_000; // 70 SBTC
const DEVELOPER_REWARD_RATE_E8: u128 = 20_000_000; // 0.2
const MAX_PHASE: u32 = 12;

/// returns total_reward (minter reward + developer reward)
pub fn reward(mint_amount: u128, total_minted: u128) -> u128 {
    let mut phase = 0;
    let mut reward_rate_e8 = INITIAL_REWARD_RATE_E8;
    let mut min_of_phase = 0;
    let mut max_of_phase = PHASE_ZERO_MAX;
    while max_of_phase < total_minted && phase < MAX_PHASE {
        let prev_phase_range_satoshi = max_of_phase - min_of_phase;
        min_of_phase = max_of_phase;
        max_of_phase = max_of_phase + prev_phase_range_satoshi * E8 / PHASE_MULTIPLIER_E8;
        reward_rate_e8 = reward_rate_e8 * PHASE_MULTIPLIER_E8 / E8;
        phase += 1
    }
    if total_minted + mint_amount < max_of_phase || phase == MAX_PHASE {
        mint_amount * reward_rate_e8 / E8
    } else {
        (max_of_phase - total_minted) * reward_rate_e8 / E8
            + (total_minted + mint_amount - max_of_phase) * reward_rate_e8 * PHASE_MULTIPLIER_E8
                / E8
                / E8
    }
}

/// returns (minter reward, developer reward)
pub fn split_reward(total_reward: u128) -> (u128, u128) {
    let developer_reward = total_reward * DEVELOPER_REWARD_RATE_E8 / E8;
    let minter_reward = total_reward - developer_reward;
    (minter_reward, developer_reward)
}

pub fn fee(release_amount: u128) -> u128 {
    release_amount * INITIAL_REWARD_RATE_E8 / E8
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rewards_at_one_phase() {
        // Phase 0
        assert_eq!(reward(1000_000_000, 0), 330_000_000_000_000);
        assert_eq!(reward(100_000_000, 0), 33_000_000_000_000);
        assert_eq!(reward(10_000_000, 0), 3_300_000_000_000);
        assert_eq!(reward(1000_000, 0), 330_000_000_000);

        assert_eq!(reward(1_000_000_000, 6_000_000_000), 330_000_000_000_000);
        assert_eq!(reward(100_000_000, 6_900_000_000), 33_000_000_000_000);
        assert_eq!(reward(10_000_000, 6_900_000_000), 3_300_000_000_000);
        assert_eq!(reward(1_000_000, 6_999_000_000), 330_000_000_000);

        // Phase 1
        assert_eq!(reward(1_000_000_000, 7_000_000_000), 264_000_000_000_000);
        assert_eq!(reward(100_000_000, 7_000_000_000), 26_400_000_000_000);
        assert_eq!(reward(10_000_000, 7_000_000_000), 2_640_000_000_000);
        assert_eq!(reward(1_000_000, 7_000_000_000), 264_000_000_000);

        assert_eq!(reward(1_000_000_000, 14_750_000_000), 264_000_000_000_000);
        assert_eq!(reward(100_000_000, 15_650_000_000), 26_400_000_000_000);
        assert_eq!(reward(10_000_000, 15_740_000_000), 2_640_000_000_000);
        assert_eq!(reward(1_000_000, 15_749_000_000), 264_000_000_000);

        // Phase 2
        assert_eq!(reward(1_000_000_000, 15_750_000_000), 211_200_000_000_000);
        assert_eq!(reward(100_000_000, 15_750_000_000), 21_120_000_000_000);
        assert_eq!(reward(10_000_000, 15_750_000_000), 2_112_000_000_000);
        assert_eq!(reward(1_000_000, 15_750_000_000), 211_200_000_000);

        // Phase 5
        assert_eq!(reward(1_000_000_000, 57_449_218_750), 108_134_400_000_000);

        // Phase 11
        assert_eq!(reward(1_000_000_000, 297_962_901_115), 28_346_784_153_600);

        // Phase 12
        assert_eq!(reward(1_000_000_000, 379_453_626_394), 22_677_427_322_880);

        // After phase 12
        assert_eq!(reward(1_000_000000, 481_317_032_993), 22_677_427_322_880);
        assert_eq!(reward(1_000_000000, 500_000_000_000), 22_677_427_322_880);
    }

    #[test]
    fn test_rewards_over_two_phases() {
        // Phase 0-1
        assert_eq!(reward(11_00000000, 6_000_000_000), 3564000_00000000);
        assert_eq!(reward(11_00000000, 6_900_000_000), 2970000_00000000);
        assert_eq!(reward(1_00000000, 6_990_000_000), 270600_00000000);

        // Phase 1-2
        assert_eq!(reward(1_00000000, 15_700_000_000), 237600_00000000);

        // Phase 12 - after phase 12
        assert_eq!(reward(1_000_000000, 481_000_000_000), 22_677_427_322_880);
    }

    #[test]
    fn test_total_issued_reward() {
        let mint_amount = 100_000_000;
        let mut total_issued_reward = 0;
        let mut total_minted = 0;
        for _ in 0..4813 {
            total_issued_reward += reward(mint_amount, total_minted);
            total_minted += mint_amount;
        }
        assert_eq!(total_issued_reward, 30029613735039752);
    }
}
