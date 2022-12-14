use near_sdk::{
    near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Gas, env, ext_contract, log, PromiseError, Promise, PromiseOrValue, require, Balance,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap},
    serde::{Deserialize, Serialize},
};
use near_sdk::serde_json::{Map, Value};
mod social;

//use crate::social::*;

const NEAR_SOCIAL_ACCOUNT_ID: &str = "social.near";
const NEAR_SOCIAL_APP_NAME: &str = "ProofofHeight";
const NEAR_SOCIAL_WINNER_BADGE: &str = "height";

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    UsersHeight
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    users_height: LookupMap<AccountId, u8>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            users_height: LookupMap::new(StorageKey::UsersHeight),
        }
    }

    pub fn get_height_inches(&self, account_id: AccountId) -> Option<u8> {
        self.users_height.get(&account_id)
    }

    #[payable]
    pub fn set_height_inches(&mut self, height: u8) {
        let account_id = env::predecessor_account_id();

        require!(!self.users_height.contains_key(&account_id), "Can't change your height");

        self.users_height.insert(&account_id, &height);
    }

    // // finalize the game. Read data from social, verify, calculate score and create NFT & Social badge for winners
    // #[payable]
    // pub fn nft_mint(&mut self, receiver_id: AccountId) -> PromiseOrValue<usize> {
    //     let account_id = env::predecessor_account_id();
    //     require!(receiver_id == account_id, "Illegal receiver");
    //     require!(self.players_score.get(&account_id).is_none(), "Already finalized");

    //     let get_request = format!("{}/{}/**", account_id, NEAR_SOCIAL_APP_NAME);

    //     ext_social::ext(AccountId::new_unchecked(NEAR_SOCIAL_ACCOUNT_ID.to_string()))
    //         .with_static_gas(GAS_FOR_SOCIAL_GET)
    //         .get(
    //             vec![get_request],
    //             None,
    //         )
    //         .then(
    //             ext_self::ext(env::current_account_id())
    //                 .with_static_gas(GAS_FOR_AFTER_SOCIAL_GET)
    //                 .with_attached_deposit(env::attached_deposit())
    //                 .after_social_get()
    //         ).into()
    // }

    // #[payable]
    // #[private]
    // pub fn after_social_get(
    //     &mut self,
    //     #[callback_result] value: Result<Value, PromiseError>,
    // ) -> usize {
    //     let mut score: usize = 0;
    //     if let Ok(mut value) = value {
    //         let data = value.as_object_mut().expect("Data is not a JSON object");
    //         for (account_id, value) in data {
    //             let account_id = AccountId::new_unchecked(account_id.to_owned());
    //             let turns = self.get_turns(account_id.clone());

    //             for (turn_index, turn_data) in value.get(NEAR_SOCIAL_APP_NAME.to_string()).expect("Missing data").as_object().expect("Missing turns") {
    //                 let turn_index = turn_index.to_owned().parse::<usize>().unwrap();
    //                 require!(turn_index < MAX_TURNS, "Illegal turn index");
    //                 for (key, value) in turn_data.as_object().unwrap() {
    //                     let value = value.as_str().unwrap();
    //                     if key == "bot" {
    //                         let turn_key = turns[turn_index] as usize;
    //                         if get_bot(turn_key) == value {
    //                             score += 1;
    //                         }
    //                     }
    //                 }
    //             }

    //             self.players_score.insert(&account_id, &score);

    //             if score == MAX_TURNS {
    //                 self.internal_mint(&account_id);
    //                 self.internal_social_set(NEAR_SOCIAL_WINNER_BADGE.to_string(), account_id);
    //                 log!("You win!");
    //             }
    //             else{
    //                 log!("You didn't win. Deposit for NFT storage reverted");
    //                 Promise::new(account_id).transfer(env::attached_deposit());
    //             }
    //         }
    //     }

    //     score
    // }

    // pub fn get_score(&self, account_id: AccountId) -> Option<usize> {
    //     self.players_score.get(&account_id)
    // }

    // // legacy method to reward first winners
    // #[private]
    // pub fn set_winner(&mut self, account_id: AccountId) {
    //     self.internal_social_set(NEAR_SOCIAL_WINNER_BADGE.to_string(), account_id);
    // }
}

// pub fn get_binary_random() -> usize {
//     let random_seed = env::random_seed();
//     (random_seed[0] % 2) as usize
// }

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env};

    use super::*;


    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new();
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.get_height_inches(accounts(1)), None);
        contract.set_height_inches(72);
        assert_eq!(contract.get_height_inches(accounts(1)), Some(72));
    }
}