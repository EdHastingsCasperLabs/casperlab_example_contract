#![cfg_attr(
    not(target_arch = "wasm32"),
    crate_type = "target arch should be wasm32"
)]
#![no_main]
mod api;
mod error;
use crate::api::Api;
use crate::error::Error;
use casperlabs_contract::{
    contract_api::{runtime, storage}
};
use casperlabs_types::{Key, URef, ApiError};
use hex;
use obi::{OBIDecode, OBIEncode};
use tiny_keccak::{Hasher, Keccak};

#[macro_use]
extern crate alloc;

pub const INIT_FLAG_KEY: [u8; 32] = [1u8; 32];

#[derive(Clone, Debug, PartialEq, OBIDecode, OBIEncode)]
pub struct MyPacket {
    pub req: Req,
    pub res: Res,
}

#[derive(Clone, Debug, PartialEq, OBIDecode, OBIEncode)]
pub struct Res {
    pub client_id: String,
    pub request_id: u64,
    pub ans_count: u64,
    pub request_time: u64,
    pub resolve_time: u64,
    pub resolve_status: u8,
    pub result: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, OBIDecode, OBIEncode)]
pub struct Req {
    pub client_id: String,
    pub oracle_script_id: u64,
    pub calldata: Vec<u8>,
    pub ans_count: u64,
    pub min_count: u64,
}

impl Req {
    pub fn get_hash(&self) -> [u8; 32] {
        let mut keccak = Keccak::v256();
        let mut output = [0u8; 32];
        keccak.update(&(self.try_to_vec().unwrap()));
        keccak.finalize(&mut output);
        output
    }
}

#[no_mangle]
#[allow(unreachable_patterns)]
pub extern "C" fn call() {
    runtime::print(&"____________ session main call _____________");
    match Api::from_args() {
        Api::RelayAndVerify(proof) => {
            let bp = match MyPacket::try_from_slice(&proof){
                Ok(bp) => bp,
                Err(e) => {
                    runtime::print(&"_____ session main call relay & verify _____");
                    runtime::print(&format!("arg 1; proof: {:?}", proof));
                    runtime::print(&format!("proof length = {}", proof.len()));
                    let text = format!("MyPacket::try_from_slice(&proof) error: {:?}", e);
                    runtime::print(&text);
                    runtime::revert(ApiError::User(Error::PacketSliceError as u16))
                }
            };

            let value_ref: URef = storage::new_uref(proof);
            let value_key: Key = value_ref.into();
            runtime::put_key(&hex::encode(bp.req.get_hash()), value_key);
        }
        _ => runtime::revert(Error::UnknownBridgeCallCommand),
    }
}

// #[no_mangle]
// #[allow(unreachable_patterns)]
// pub extern "C" fn my_contract() {
//     match Api::from_args() {
//         Api::RelayAndVerify(proof) => {
//             let bp = MyPacket::try_from_slice(&proof).unwrap();
//
//             let value_ref: URef = storage::new_uref(proof);
//             let value_key: Key = value_ref.into();
//             runtime::put_key(&hex::encode(bp.req.get_hash()), value_key);
//         }
//         _ => runtime::revert(Error::UnknownBridgeCallCommand),
//     }
// }
