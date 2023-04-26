/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

/// BlockRewards : Fields relating to rewards,

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct BlockRewards {
    /// \\[fees\\] accepts transaction fees, it can only spend to the incentive pool.
    #[serde(rename = "fee-sink")]
    pub fee_sink: String,
    /// \\[rwcalr\\] number of leftover MicroAlgos after the distribution of rewards-rate MicroAlgos for every reward unit in the next round.
    #[serde(rename = "rewards-calculation-round")]
    pub rewards_calculation_round: i32,
    /// \\[earn\\] How many rewards, in MicroAlgos, have been distributed to each RewardUnit of MicroAlgos since genesis.
    #[serde(rename = "rewards-level")]
    pub rewards_level: i32,
    /// \\[rwd\\] accepts periodic injections from the fee-sink and continually redistributes them as rewards.
    #[serde(rename = "rewards-pool")]
    pub rewards_pool: String,
    /// \\[rate\\] Number of new MicroAlgos added to the participation stake from rewards at the next round.
    #[serde(rename = "rewards-rate")]
    pub rewards_rate: i32,
    /// \\[frac\\] Number of leftover MicroAlgos after the distribution of RewardsRate/rewardUnits MicroAlgos for every reward unit in the next round.
    #[serde(rename = "rewards-residue")]
    pub rewards_residue: i32,
}

impl BlockRewards {
    /// Fields relating to rewards,
    pub fn new(
        fee_sink: String,
        rewards_calculation_round: i32,
        rewards_level: i32,
        rewards_pool: String,
        rewards_rate: i32,
        rewards_residue: i32,
    ) -> BlockRewards {
        BlockRewards {
            fee_sink,
            rewards_calculation_round,
            rewards_level,
            rewards_pool,
            rewards_rate,
            rewards_residue,
        }
    }
}
