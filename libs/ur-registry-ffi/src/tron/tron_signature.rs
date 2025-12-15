use crate::response::{PtrResponse, Response};
use crate::types::PtrVoid;

use serde_cbor::Value;
use std::collections::BTreeMap;

// CBOR map keys for TronSignature
const REQUEST_ID: i128 = 1;
const SIGNATURE: i128 = 2;

// UR Type for TRON signature
pub const TRON_SIGNATURE_TYPE: &str = "tron-signature";

#[derive(Clone, Debug, Default)]
pub struct TronSignature {
    request_id: Option<Vec<u8>>,
    signature: Vec<u8>,
}

impl TronSignature {
    pub fn new(request_id: Option<Vec<u8>>, signature: Vec<u8>) -> Self {
        TronSignature {
            request_id,
            signature,
        }
    }

    pub fn get_request_id(&self) -> Option<&Vec<u8>> {
        self.request_id.as_ref()
    }

    pub fn get_signature(&self) -> &Vec<u8> {
        &self.signature
    }

    /// Serialize to CBOR bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut map: BTreeMap<Value, Value> = BTreeMap::new();

        if let Some(ref id) = self.request_id {
            map.insert(Value::Integer(REQUEST_ID), Value::Bytes(id.clone()));
        }

        map.insert(Value::Integer(SIGNATURE), Value::Bytes(self.signature.clone()));

        serde_cbor::to_vec(&Value::Map(map))
            .map_err(|e| e.to_string())
    }
}

impl TryFrom<Vec<u8>> for TronSignature {
    type Error = String;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let cbor_value: Value = serde_cbor::from_slice(&data)
            .map_err(|e| format!("Failed to decode CBOR: {}", e))?;

        if let Value::Map(map) = cbor_value {
            let request_id = map.get(&Value::Integer(REQUEST_ID))
                .and_then(|v| if let Value::Bytes(b) = v { Some(b.clone()) } else { None });

            let signature = map.get(&Value::Integer(SIGNATURE))
                .and_then(|v| if let Value::Bytes(b) = v { Some(b.clone()) } else { None })
                .ok_or("Missing signature")?;

            Ok(TronSignature {
                request_id,
                signature,
            })
        } else {
            Err("Expected CBOR map".to_string())
        }
    }
}

// ========== FFI Functions ==========

pub fn resolve(data: Vec<u8>) -> PtrResponse {
    match TronSignature::try_from(data) {
        Ok(result) => Response::success_object(Box::into_raw(Box::new(result)) as PtrVoid).c_ptr(),
        Err(error) => Response::error(error.to_string()).c_ptr(),
    }
}

#[no_mangle]
pub extern "C" fn tron_signature_get_signature(tron_signature: &mut TronSignature) -> PtrResponse {
    Response::success_string(hex::encode(tron_signature.get_signature())).c_ptr()
}

#[no_mangle]
pub extern "C" fn tron_signature_get_request_id(tron_signature: &mut TronSignature) -> PtrResponse {
    match tron_signature.get_request_id() {
        Some(v) => Response::success_string(hex::encode(v)).c_ptr(),
        None => Response::error(format!("No request id supplied")).c_ptr()
    }
}
