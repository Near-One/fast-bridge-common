use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    json_types::U128, log, serde::Deserialize, serde::Serialize, serde_json, AccountId,
};
use serde_json::json;
#[allow(unused_imports)]
use serde::de::Unexpected::Option;

pub const STANDARD: &str = "nep297";
pub const VERSION: &str = "1.0.0";
pub const EVENT_JSON_STR: &str = "EVENT_JSON:";

pub type EthAddress = [u8; 20];

#[derive(
    Default, BorshDeserialize, BorshSerialize, Debug, Clone, Serialize, Deserialize, PartialEq,
)]
pub struct Proof {
    pub log_index: u64,
    pub log_entry_data: Vec<u8>,
    pub receipt_index: u64,
    pub receipt_data: Vec<u8>,
    pub header_data: Vec<u8>,
    pub proof: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferDataEthereum {
    pub token_near: AccountId,
    pub token_eth: EthAddress,
    pub amount: U128,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferDataNear {
    pub token: AccountId,
    pub amount: U128,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum Event {
    SpectreBridgeTransferEvent {
        nonce: U128,
        chain_id: u32,
        valid_till: u64,
        transfer: TransferDataEthereum,
        fee: TransferDataNear,
        recipient: EthAddress,
    },
    SpectreBridgeUnlockEvent {
        nonce: U128,
        account: AccountId,
    },
    SpectreBridgeDepositEvent {
        account: AccountId,
        token: AccountId,
        amount: U128,
    },
    SpectreBridgeEthProoverNotProofedEvent {
        nonce: U128,
        proof: Proof,
    },
}

#[allow(dead_code)]
pub fn get_eth_address(address: String) -> EthAddress {
    let data = hex::decode(address).expect("address should be a valid hex string.");
    assert_eq!(data.len(), 20, "address should be 20 bytes long");
    let mut result = [0u8; 20];
    result.copy_from_slice(&data);
    result
}

pub fn remove_prefix(event_str: &str) -> std::option::Option<serde_json::Value> {
    if let Some(value) = event_str.strip_prefix(EVENT_JSON_STR) {
        if let Ok(r) = serde_json::from_str::<serde_json::Value>(value) {
            return Some(r);
        }
    }
    None
}

impl Event {
    #[allow(dead_code)]
    pub fn emit(&self) {
        emit_event(&self);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EventMessage {
    pub standard: String,
    pub version: String,
    pub event: serde_json::Value,
    pub data: serde_json::Value,
}

#[allow(dead_code)]
pub(crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let result = json!(data);
    let event_json = json!(EventMessage {
        standard: STANDARD.to_string(),
        version: VERSION.to_string(),
        event: result["event"].clone(),
        data: result["data"].clone()
    })
    .to_string();
    log!(format!("{}{}", EVENT_JSON_STR, event_json));
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{test_utils, AccountId};
    use assert_json_diff::assert_json_eq;

    fn alice() -> AccountId {
        AccountId::new_unchecked("alice".to_string())
    }

    fn token() -> AccountId {
        AccountId::new_unchecked("token".to_string())
    }

    fn get_eth_address() -> EthAddress {
        let address: String = "71C7656EC7ab88b098defB751B7401B5f6d8976F".to_string();
        super::get_eth_address(address)
    }

    #[test]
    fn nonce_event_test() {
        let nonce = U128(238);
        let token = token();
        let amount = U128(100);
        let validator_id = alice();
        let token_address = get_eth_address();
        Event::SpectreBridgeNonceEvent {
            nonce,
            account: validator_id,
            transfer: TransferDataEthereum {
                token_near: token,
                token_eth: token_address,
                amount,
            },
            recipient: token_address,
        }.emit();

        println!("{:?}", token_address);
        let log_data_str = &test_utils::get_logs()[0];
        let expected_result_str = r#"EVENT_JSON:{"standard":"nep297","version":"1.0.0","event":"spectre_bridge_nonce_event","data":{"nonce":"238","account":"alice","transfer":{"token_near":"token","token_eth":[113, 199, 101, 110, 199, 171, 136, 176, 152, 222, 251, 117, 27, 116, 1, 181, 246, 216, 151, 111],"amount":"100"},"recipient":[113,199,101,110,199,171,136,176,152,222,251,117,27,116,1,181,246,216,151,111]}}"#;

        let json1 = remove_prefix(log_data_str).unwrap();
        let json2 = remove_prefix(expected_result_str).unwrap();

        assert_json_eq!(json1, json2)
    }

    #[test]
    fn failed_event_test() {
        let nonce = U128(238);
        let validator_id = alice();
        Event::SpectreBridgeTransferFailedEvent {
            nonce,
            account: validator_id,
        }.emit();

        let log_data_str = &test_utils::get_logs()[0];
        let expected_result_str = r#"EVENT_JSON:{"standard":"nep297","version":"1.0.0","event":"spectre_bridge_transfer_failed_event","data":{"nonce":"238","account":"alice"}}"#;

        let json1 = remove_prefix(log_data_str).unwrap();
        let json2 = remove_prefix(expected_result_str).unwrap();

        assert_json_eq!(json1, json2)
    }

    #[test]
    fn transfer_event_test() {
        let nonce = U128(238);
        let validator_id = alice();
        let token_address = get_eth_address();
        let amount: u128 = 100;
        Event::SpectreBridgeTransferEvent {
            nonce,
            chain_id: 5,
            valid_till: 0,
            transfer: TransferDataEthereum {
                token_near: validator_id.clone(),
                token_eth: token_address,
                amount: U128(amount),
            },
            fee: TransferDataNear {
                token: validator_id,
                amount: U128(amount),
            },
            recipient: token_address,
        }
        .emit();

        let log_data_str = &test_utils::get_logs()[0];
        let expected_result_str = r#"EVENT_JSON:{"standard":"nep297","version":"1.0.0","event":"spectre_bridge_transfer_event","data":{"nonce":"238","chain_id":5,"valid_till":0,"transfer":{"token_near":"alice.near","token_eth": [113,199,101,110,199,171,136,176,152,222,251,117,27,116,1,181,246,216,151,111],"amount":"100"},"fee":{"token":"alice.near","amount":"100"},"recipient":[113,199,101,110,199,171,136,176,152,222,251,117,27,116,1,181,246,216,151,111]}}"#;

        let json1 = remove_prefix(log_data_str).unwrap();
        let json2 = remove_prefix(expected_result_str).unwrap();

        assert_json_eq!(json1, json2)
    }

    #[test]
    fn unlock_event_test() {
        let nonce = U128(238);
        let validator_id = alice();
        Event::SpectreBridgeUnlockEvent {
            nonce,
            account: validator_id,
        }
        .emit();

        let log_data_str = &test_utils::get_logs()[0];
        let expected_result_str = r#"EVENT_JSON:{"standard":"nep297","version":"1.0.0","event":"spectre_bridge_unlock_event","data":{"nonce":"238","account":"alice.near"}}"#;

        let json1 = remove_prefix(log_data_str).unwrap();
        let json2 = remove_prefix(expected_result_str).unwrap();

        assert_json_eq!(json1, json2)
    }
}
