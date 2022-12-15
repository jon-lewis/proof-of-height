use near_sdk::{
    near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Gas, env, ext_contract, Balance,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap},
    serde::{Deserialize, Serialize}, store::UnorderedSet, CryptoHash, require, NearSchema,
};
use near_sdk::serde_json::{Value};
//mod social;

//use crate::social::*;

const NEAR_SOCIAL_ACCOUNT_ID: &str = "social.near";
// const NEAR_SOCIAL_APP_NAME: &str = "Proof of Height";
// const NEAR_SOCIAL_WINNER_BADGE: &str = "height";

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    UsersHeight,
    VotesByUser,
    VotersByUser,
    SubVotersByUserSet { account_hash: CryptoHash },
}

#[derive(NearSchema, BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VoteChoice {
    DefinitelyYes,
    Yes,
    No,
    DefinitelyNo
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Votes {
    weighted_sum: i64,
    total_votes: u32,
}

#[derive(NearSchema, BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Confidence {
    Lie,
    ProbablyALie,
    Inconclusive,
    MightBeTrue,
    True
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    users_height: LookupMap<AccountId, u8>,
    // a map of current voting outcome for a specific user
    votes_by_user: LookupMap<AccountId, Votes>,
    // a map of a set of users that voted on a specific user
    voters_by_user: LookupMap<AccountId, UnorderedSet<AccountId>>, 
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            users_height: LookupMap::new(StorageKey::UsersHeight),
            votes_by_user: LookupMap::new(StorageKey::VotesByUser),
            voters_by_user: LookupMap::new(StorageKey::VotersByUser),
        }
    }

    pub fn get_height_inches(&self, account_id: AccountId) -> Option<u8> {
        self.users_height.get(&account_id)
    }

    pub fn get_who_voted_for(&self, account_id: AccountId) -> Vec<AccountId> {
        match self.voters_by_user.get(&account_id) {
            Some(voters) => voters.iter().cloned().collect::<Vec<AccountId>>(),
            None => vec![]
        }
    }

    pub fn get_confidence(&self, account_id: AccountId) -> Option<Confidence> {
        match self.votes_by_user.get(&account_id) {
            None => None,
            Some(votes) => {
                if votes.total_votes == 0 {
                    return Some(Confidence::Inconclusive);
                }
                let score = votes.weighted_sum / votes.total_votes as i64;
                println!("Score: {}", score);
                Some(match score {
                    -2 => Confidence::Lie,
                    -1 => Confidence::ProbablyALie,
                    1 => Confidence::MightBeTrue,
                    2 => Confidence::True,
                    _ => Confidence::Inconclusive
                })
            }
        }
    }

    #[payable]
    pub fn set_height_inches(&mut self, height: u8) {
        let account_id = env::predecessor_account_id();

        require!(!self.users_height.contains_key(&account_id), "Can't change your height");

        self.users_height.insert(&account_id, &height);
        self.votes_by_user.insert(&account_id, &Votes { weighted_sum: 0, total_votes: 0 });

        // Ref: https://docs.near.org/sdk/rust/contract-structure/nesting#generating-unique-prefixes-for-persistent-collections
        self.voters_by_user.insert(&account_id, & UnorderedSet::new(StorageKey::SubVotersByUserSet {
            account_hash: env::sha256_array(account_id.as_bytes()),
        }));
    }

    #[payable]
    pub fn vote(&mut self, account_id: AccountId, vote: VoteChoice) {
        let current_user = env::predecessor_account_id();

        require!(current_user != account_id, "Can't confirm your own height");
        require!(self.voters_by_user.contains_key(&account_id) && self.votes_by_user.contains_key(&account_id), "This user must enter their height first");

        // Ref: https://docs.near.org/sdk/rust/contract-structure/nesting
        let mut voters = self.voters_by_user.get(&account_id).unwrap();
        require!(!voters.contains(&current_user), "Can't vote more than once");
        voters.insert(current_user);
        self.voters_by_user.insert(&account_id, &voters);

        let mut outcome = self.votes_by_user.get(&account_id).unwrap();
        outcome.total_votes += 1;
        outcome.weighted_sum += weight(vote);
        self.votes_by_user.insert(&account_id, &outcome);
    }
}

fn weight(vote: VoteChoice) -> i64 {
    match vote {
        VoteChoice::DefinitelyYes => 2,
        VoteChoice::Yes => 1,
        VoteChoice::No => -1,
        VoteChoice::DefinitelyNo => -2,
    }
}

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
        let mut contract = Contract::new();
        
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.get_height_inches(accounts(1)), None);

        testing_env!(context.is_view(false).build());
        contract.set_height_inches(72);
        
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.get_height_inches(accounts(1)), Some(72));
    }

    #[test]
    #[should_panic(expected = "Can't change your height")]
    fn test_setting_height_twice() {
        let mut context = get_context(accounts(1));
        let mut contract = Contract::new();
        
        testing_env!(context.is_view(false).build());
        contract.set_height_inches(72);

        testing_env!(context.is_view(false).build());
        contract.set_height_inches(74);
    }

    #[test]
    #[should_panic(expected = "Can't confirm your own height")]
    fn test_voting_on_own_account() {
        let context = get_context(accounts(0));
        let mut contract = Contract::new();
        
        testing_env!(context.build());

        contract.vote(accounts(0), VoteChoice::DefinitelyYes);
    }

    #[test]
    #[should_panic(expected = "This user must enter their height first")]
    fn test_voting_without_height() {
        let context = get_context(accounts(1));
        let mut contract = Contract::new();
        
        testing_env!(context.build());

        contract.vote(accounts(0), VoteChoice::DefinitelyYes);
    }

    #[test]
    fn test_voting() {
        let mut context = get_context(accounts(1));
        let mut contract = Contract::new();
        
        testing_env!(context.build());

        assert_eq!(contract.get_who_voted_for(accounts(1)), vec![]);
        contract.set_height_inches(72);

        testing_env!(context.predecessor_account_id(accounts(0)).build());

        contract.vote(accounts(1), VoteChoice::DefinitelyYes);

        assert_eq!(contract.get_who_voted_for(accounts(1)), vec![accounts(0)]);

        assert_eq!(contract.get_confidence(accounts(1)), Some(Confidence::True));

        // Have another person vote a non-confident "no"
        testing_env!(context.predecessor_account_id(accounts(2)).build());

        contract.vote(accounts(1), VoteChoice::No);

        assert_eq!(contract.get_who_voted_for(accounts(1)), vec![accounts(0), accounts(2)]);

        assert_eq!(contract.get_confidence(accounts(1)), Some(Confidence::Inconclusive));

        // Have another person vote a non-confident "yes"
        testing_env!(context.predecessor_account_id(accounts(3)).build());

        contract.vote(accounts(1), VoteChoice::Yes);

        assert_eq!(contract.get_who_voted_for(accounts(1)), vec![accounts(0), accounts(2), accounts(3)]);

        assert_eq!(contract.get_confidence(accounts(1)), Some(Confidence::Inconclusive));
    }

    #[test]
    fn test_confidence_without_votes() {
        let mut context = get_context(accounts(1));
        let mut contract = Contract::new();
        
        testing_env!(context.build());

        contract.set_height_inches(72);

        testing_env!(context.predecessor_account_id(accounts(0)).build());

        assert_eq!(contract.get_confidence(accounts(1)), Some(Confidence::Inconclusive));
    }
}