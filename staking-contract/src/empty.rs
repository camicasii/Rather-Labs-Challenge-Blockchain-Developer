#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const BLOCKS_IN_YEAR: u64 = 60 * 60 * 24 * 365 / 6;
pub const MAX_PERCENTAGE: u64 = 10_000;

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Debug)]
pub struct StakingPosition<M: ManagedTypeApi> {
    pub stake_amount: BigUint<M>,
    pub total_invested: BigUint<M>,
    pub total_withdrawn: BigUint<M>,
    pub last_action_block: u64,
}

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Debug)]
pub struct StakingGlobal<M: ManagedTypeApi> {
    pub stake_amount: BigUint<M>,
    pub total_invested: BigUint<M>,
    pub total_withdrawn: BigUint<M>,
    pub reward_per_second: BigUint<M>,
    pub reward_per_block: BigUint<M>,
}

#[multiversx_sc::contract]
pub trait StakingContract {
    #[init]
    fn init(&self) {
        let base: u64 = 3 * u64::pow(10, 14);
        let base_per_block = base * 6;
        self.staking_global().set(StakingGlobal {
            stake_amount: BigUint::zero(),
            total_invested: BigUint::zero(),
            total_withdrawn: BigUint::zero(),
            reward_per_second: BigUint::from(base),
            reward_per_block: BigUint::from(base_per_block),
        });
    }

    #[payable("EGLD")]
    #[endpoint]
    fn stake(&self) {
        let payment_amount = self.call_value().egld_value().clone_value();
        require!(payment_amount > 0, "Must pay more than 0");

        let caller = self.blockchain().get_caller();

        let stake_mapper = self.staking_position(&caller);
        let new_user = self.staked_addresses().insert(caller.clone());

        let mut staking_pos = if !new_user {
            stake_mapper.get()
        } else {
            let current_block = self.blockchain().get_block_epoch();
            StakingPosition {
                stake_amount: BigUint::zero(),
                last_action_block: current_block,
                total_invested: BigUint::zero(),
                total_withdrawn: BigUint::zero(),
            }
        };

        self.claim_rewards_for_user(&caller, &mut staking_pos);
        staking_pos.stake_amount += payment_amount.clone();

        stake_mapper.set(&staking_pos);

        let mut staking_global = self.staking_global().get();
        staking_global.total_invested += payment_amount.clone();
        staking_global.stake_amount += payment_amount.clone();
        self.staking_global().set(&staking_global);
    }

    #[endpoint]
    fn unstake(&self, opt_unstake_amount: OptionalValue<BigUint>) {
        let caller = self.blockchain().get_caller();
        self.require_user_staked(&caller);

        let stake_mapper = self.staking_position(&caller);
        let mut staking_pos = stake_mapper.get();

        let unstake_amount = match opt_unstake_amount {
            OptionalValue::Some(amt) => amt,
            OptionalValue::None => staking_pos.stake_amount.clone(),
        };
        require!(
            unstake_amount > 0 && unstake_amount <= staking_pos.stake_amount,
            "Invalid unstake amount"
        );

        self.claim_rewards_for_user(&caller, &mut staking_pos);
        staking_pos.stake_amount -= &unstake_amount;
        staking_pos.total_withdrawn += unstake_amount.clone();

        if staking_pos.stake_amount > 0 {
            stake_mapper.set(&staking_pos);
        } else {
            stake_mapper.clear();
            self.staked_addresses().swap_remove(&caller);
        }
        let mut staking_global = self.staking_global().get();
        staking_global.total_withdrawn += unstake_amount.clone();
        staking_global.stake_amount -= unstake_amount.clone();
        self.staking_global().set(&staking_global);
        self.send().direct_egld(&caller, &unstake_amount);
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let caller = self.blockchain().get_caller();
        self.require_user_staked(&caller);

        let stake_mapper = self.staking_position(&caller);
        let mut staking_pos = stake_mapper.get();
        self.claim_rewards_for_user(&caller, &mut staking_pos);

        stake_mapper.set(&staking_pos);
    }

    fn claim_rewards_for_user(
        &self,
        user: &ManagedAddress,
        staking_pos: &mut StakingPosition<Self::Api>,
    ) {
        let reward_amount_divider = self.calculate_rewards(staking_pos);
        let current_block = self.blockchain().get_block_nonce();
        staking_pos.last_action_block = current_block;
        if reward_amount_divider == BigUint::zero() {
            return;
        }
        let divider: u64 = 10_000;
        let reward_amount = &staking_pos.stake_amount * divider / reward_amount_divider;
        let d_r = reward_amount / divider;
        if d_r > 0 {
            self.send().direct_egld(user, &d_r);
        }
    }

    fn calculate_rewards(&self, staking_position: &StakingPosition<Self::Api>) -> BigUint {
        let current_block = self.blockchain().get_block_nonce();

        if current_block <= staking_position.last_action_block {
            return BigUint::zero();
        }

        let reward_per_block = self.staking_global().get().reward_per_block.clone();
        let total_invested = self.staking_global().get().stake_amount.clone();
        let block_diff = current_block - staking_position.last_action_block;

        let d_r_base = reward_per_block * block_diff;
        let d_s_ = total_invested * d_r_base;
        d_s_
    }

    #[view(calculateRewardsForUser)]
    fn calculate_rewards_for_user(&self, addr: ManagedAddress) -> BigUint {
        let staking_pos = self.staking_position(&addr).get();
        self.calculate_rewards(&staking_pos)
    }

    fn require_user_staked(&self, user: &ManagedAddress) {
        require!(self.staked_addresses().contains(user), "Must stake first");
    }

    #[view(getStakedAddresses)]
    #[storage_mapper("stakedAddresses")]
    fn staked_addresses(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getStakingPosition)]
    #[storage_mapper("stakingPosition")]
    fn staking_position(
        &self,
        addr: &ManagedAddress,
    ) -> SingleValueMapper<StakingPosition<Self::Api>>;

    #[view(getStakingGlobal)]
    #[storage_mapper("stakingGlobal")]
    fn staking_global(&self) -> SingleValueMapper<StakingGlobal<Self::Api>>;
}
