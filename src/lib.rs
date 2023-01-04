use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    json_types::U128, log, require, serde::Deserialize, serde::Serialize, serde_json, AccountId,
};
use serde_json::json;

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
    #[serde(with = "hex::serde")]
    pub token_eth: EthAddress,
    pub amount: U128,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferDataNear {
    pub token: AccountId,
    pub amount: U128,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferMessage {
    valid_till: u64,
    transfer: TransferDataEthereum,
    fee: TransferDataNear,
    #[serde(with = "hex::serde")]
    recipient: EthAddress,
    valid_till_block_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum Event {
    SpectreBridgeInitTransferEvent {
        nonce: U128,
        sender_id: AccountId,
        transfer_message: TransferMessage,
    },
    SpectreBridgeUnlockEvent {
        nonce: U128,
        unlock_recipient: AccountId,
        transfer_message: TransferMessage,
    },
    SpectreBridgeLpUnlockEvent {
        nonce: U128,
        unlock_recipient: AccountId,
        transfer_message: TransferMessage,
    },
    SpectreBridgeDepositEvent {
        account: AccountId,
        token: AccountId,
        amount: U128,
    },
}

#[allow(dead_code)]
pub fn get_eth_address(address: String) -> EthAddress {
    let data = hex::decode(address)
        .unwrap_or_else(|_| near_sdk::env::panic_str("address should be a valid hex string."));
    require!(data.len() == 20, "address should be 20 bytes long");
    data.try_into().unwrap()
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
    use assert_json_diff::assert_json_eq;
    use near_sdk::test_utils::test_env::alice;
    use near_sdk::{test_utils, AccountId};

    fn token() -> AccountId {
        AccountId::new_unchecked("token.near".to_string())
    }

    fn get_eth_address() -> EthAddress {
        let address: String = "71C7656EC7ab88b098defB751B7401B5f6d8976F".to_string();
        super::get_eth_address(address)
    }

    #[test]
    fn transfer_event_test() {
        let nonce = U128(238);
        let validator_id = alice();
        let token_address = get_eth_address();
        let amount: u128 = 100;
        let sender_id = "sender.near".parse().unwrap();

        Event::SpectreBridgeInitTransferEvent {
            nonce,
            sender_id,
            transfer_message: TransferMessage {
                valid_till: 0,
                valid_till_block_height: Some(0),
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
            },
        }
        .emit();

        let log_data_str = &test_utils::get_logs()[0];
        let expected_result_str = r#"EVENT_JSON:{"standard":"nep297","version":"1.0.0","event":"spectre_bridge_init_transfer_event","data":{"nonce":"238","sender_id":"sender.near","transfer_message":{"valid_till":0,"valid_till_block_height":0,"transfer":{"token_near":"alice.near","token_eth": "71c7656ec7ab88b098defb751b7401b5f6d8976f","amount":"100"},"fee":{"token":"alice.near","amount":"100"},"recipient": "71c7656ec7ab88b098defb751b7401b5f6d8976f"}}}"#;

        let json1 = remove_prefix(log_data_str).unwrap();
        let json2 = remove_prefix(expected_result_str).unwrap();

        assert_json_eq!(json1, json2)
    }

    #[test]
    fn unlock_event_test() {
        let nonce = U128(238);
        let validator_id = alice();
        let token_address = get_eth_address();
        let amount: u128 = 100;
        let sender_id = "sender.near".parse().unwrap();

        Event::SpectreBridgeUnlockEvent {
            nonce,
            unlock_recipient: sender_id,
            transfer_message: TransferMessage {
                valid_till: 0,
                valid_till_block_height: Some(0),
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
            },
        }
        .emit();

        let log_data_str = &test_utils::get_logs()[0];
        let expected_result_str = r#"EVENT_JSON:{"standard":"nep297","version":"1.0.0","event":"spectre_bridge_unlock_event","data":{"nonce":"238","unlock_recipient":"sender.near","transfer_message":{"valid_till":0,"valid_till_block_height":0,"transfer":{"token_near":"alice.near","token_eth": "71c7656ec7ab88b098defb751b7401b5f6d8976f","amount":"100"},"fee":{"token":"alice.near","amount":"100"},"recipient": "71c7656ec7ab88b098defb751b7401b5f6d8976f"}}}"#;

        let json1 = remove_prefix(log_data_str).unwrap();
        let json2 = remove_prefix(expected_result_str).unwrap();

        assert_json_eq!(json1, json2)
    }

    #[test]
    fn deposit_event_test() {
        let account = alice();
        let token = token();
        let amount = 300;
        Event::SpectreBridgeDepositEvent {
            account,
            token,
            amount: U128(amount),
        }
        .emit();
        let log_data_str = &test_utils::get_logs()[0];
        let expected_result_str = r#"EVENT_JSON:{"standard":"nep297","version":"1.0.0","event":"spectre_bridge_deposit_event","data":{"account":"alice.near","token":"token.near","amount":"300"}}"#;

        let json1 = remove_prefix(log_data_str).unwrap();
        let json2 = remove_prefix(expected_result_str).unwrap();

        assert_json_eq!(json1, json2)
    }
}
