use crate::response::{PtrResponse, Response};
use crate::types::{PtrString, PtrVoid};
use crate::utils::{convert_ptr_string_to_string, parse_ptr_string_to_bytes};

use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use std::collections::BTreeMap;

// CBOR map keys for TronSignRequest
const REQUEST_ID: i128 = 1;
const SIGN_DATA: i128 = 2;
const DATA_TYPE: i128 = 3;
const DERIVATION_PATH: i128 = 4;
const ADDRESS: i128 = 5;
const ORIGIN: i128 = 6;

// UR Type for TRON sign request
pub const TRON_SIGN_REQUEST_TYPE: &str = "tron-sign-request";

#[derive(Clone, Debug)]
pub enum DataType {
    Transaction = 1,
    Message = 2,
    TypedData = 3,
}

impl DataType {
    pub fn from_u32(value: u32) -> Result<Self, String> {
        match value {
            1 => Ok(DataType::Transaction),
            2 => Ok(DataType::Message),
            3 => Ok(DataType::TypedData),
            _ => Err(format!("Invalid data type: {}", value)),
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            DataType::Transaction => 1,
            DataType::Message => 2,
            DataType::TypedData => 3,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TronSignRequest {
    request_id: Option<Vec<u8>>,
    sign_data: Vec<u8>,
    data_type: u32,  // 1=Transaction, 2=Message, 3=TypedData
    derivation_path: String,
    xfp: Option<u32>,
    address: Option<String>,
    origin: Option<String>,
}

impl TronSignRequest {
    pub fn new(
        request_id: Option<Vec<u8>>,
        sign_data: Vec<u8>,
        data_type: u32,
        derivation_path: String,
        xfp: Option<u32>,
        address: Option<String>,
        origin: Option<String>,
    ) -> Self {
        TronSignRequest {
            request_id,
            sign_data,
            data_type,
            derivation_path,
            xfp,
            address,
            origin,
        }
    }

    pub fn get_request_id(&self) -> Option<&Vec<u8>> {
        self.request_id.as_ref()
    }

    pub fn get_sign_data(&self) -> &Vec<u8> {
        &self.sign_data
    }

    pub fn get_data_type(&self) -> u32 {
        self.data_type
    }

    pub fn get_derivation_path(&self) -> &str {
        &self.derivation_path
    }

    pub fn get_address(&self) -> Option<&String> {
        self.address.as_ref()
    }

    pub fn get_origin(&self) -> Option<&String> {
        self.origin.as_ref()
    }

    /// Serialize to CBOR bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut map: BTreeMap<Value, Value> = BTreeMap::new();

        if let Some(ref id) = self.request_id {
            map.insert(Value::Integer(REQUEST_ID), Value::Bytes(id.clone()));
        }

        map.insert(Value::Integer(SIGN_DATA), Value::Bytes(self.sign_data.clone()));
        map.insert(Value::Integer(DATA_TYPE), Value::Integer(self.data_type as i128));

        // Encode derivation path as CBOR
        let path_cbor = encode_derivation_path(&self.derivation_path, self.xfp)?;
        map.insert(Value::Integer(DERIVATION_PATH), path_cbor);

        if let Some(ref addr) = self.address {
            map.insert(Value::Integer(ADDRESS), Value::Text(addr.clone()));
        }

        if let Some(ref origin) = self.origin {
            map.insert(Value::Integer(ORIGIN), Value::Text(origin.clone()));
        }

        serde_cbor::to_vec(&Value::Map(map))
            .map_err(|e| e.to_string())
    }
}

/// Encode BIP44 derivation path to CBOR
fn encode_derivation_path(path: &str, xfp: Option<u32>) -> Result<Value, String> {
    // Parse path like "m/44'/195'/0'/0/0"
    let parts: Vec<&str> = path.trim_start_matches("m/").split('/').collect();
    let mut components: Vec<Value> = Vec::new();

    for part in parts {
        let hardened = part.ends_with('\'');
        let index: u32 = part.trim_end_matches('\'').parse()
            .map_err(|_| format!("Invalid path component: {}", part))?;
        
        // CBOR array: [index, hardened]
        components.push(Value::Array(vec![
            Value::Integer(index as i128),
            Value::Bool(hardened),
        ]));
    }

    let mut map: BTreeMap<Value, Value> = BTreeMap::new();
    map.insert(Value::Integer(1), Value::Array(components));
    
    if let Some(fingerprint) = xfp {
        map.insert(Value::Integer(2), Value::Integer(fingerprint as i128));
    }

    Ok(Value::Map(map))
}

impl TryFrom<Vec<u8>> for TronSignRequest {
    type Error = String;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let cbor_value: Value = serde_cbor::from_slice(&data)
            .map_err(|e| format!("Failed to decode CBOR: {}", e))?;

        if let Value::Map(map) = cbor_value {
            let request_id = map.get(&Value::Integer(REQUEST_ID))
                .and_then(|v| if let Value::Bytes(b) = v { Some(b.clone()) } else { None });

            let sign_data = map.get(&Value::Integer(SIGN_DATA))
                .and_then(|v| if let Value::Bytes(b) = v { Some(b.clone()) } else { None })
                .ok_or("Missing sign_data")?;

            let data_type = map.get(&Value::Integer(DATA_TYPE))
                .and_then(|v| if let Value::Integer(i) = v { Some(*i as u32) } else { None })
                .unwrap_or(1);

            let derivation_path = decode_derivation_path(
                map.get(&Value::Integer(DERIVATION_PATH))
            )?;

            let address = map.get(&Value::Integer(ADDRESS))
                .and_then(|v| if let Value::Text(s) = v { Some(s.clone()) } else { None });

            let origin = map.get(&Value::Integer(ORIGIN))
                .and_then(|v| if let Value::Text(s) = v { Some(s.clone()) } else { None });

            Ok(TronSignRequest {
                request_id,
                sign_data,
                data_type,
                derivation_path,
                xfp: None,  // Will be decoded from path if present
                address,
                origin,
            })
        } else {
            Err("Expected CBOR map".to_string())
        }
    }
}

fn decode_derivation_path(value: Option<&Value>) -> Result<String, String> {
    match value {
        Some(Value::Map(map)) => {
            let components = map.get(&Value::Integer(1))
                .and_then(|v| if let Value::Array(arr) = v { Some(arr) } else { None })
                .ok_or("Missing path components")?;

            let mut path = String::from("m");
            for component in components {
                if let Value::Array(arr) = component {
                    if arr.len() >= 2 {
                        let index = if let Value::Integer(i) = &arr[0] { *i as u32 } else { 0 };
                        let hardened = if let Value::Bool(b) = &arr[1] { *b } else { false };
                        
                        path.push('/');
                        path.push_str(&index.to_string());
                        if hardened {
                            path.push('\'');
                        }
                    }
                }
            }
            Ok(path)
        }
        _ => Ok("m/44'/195'/0'/0/0".to_string()), // Default TRON path
    }
}

// ========== FFI Functions ==========

pub fn resolve(data: Vec<u8>) -> PtrResponse {
    match TronSignRequest::try_from(data) {
        Ok(result) => Response::success_object(Box::into_raw(Box::new(result)) as PtrVoid).c_ptr(),
        Err(error) => Response::error(error.to_string()).c_ptr(),
    }
}

#[no_mangle]
pub extern "C" fn tron_sign_request_new() -> PtrResponse {
    Response::success_object(Box::into_raw(Box::new(TronSignRequest::default())) as PtrVoid).c_ptr()
}

#[no_mangle]
pub extern "C" fn tron_sign_request_construct(
    request_id: PtrString,
    sign_data: PtrString,
    path: PtrString,
    xfp: u32,
    address: PtrString,
    origin: PtrString,
    data_type: u32,
) -> PtrResponse {
    let request_id = match parse_ptr_string_to_bytes(request_id).map_err(|e| Response::error(e)) {
        Ok(v) => v,
        Err(e) => return e.c_ptr(),
    };
    let sign_data = match parse_ptr_string_to_bytes(sign_data).map_err(|e| Response::error(e)) {
        Ok(v) => v,
        Err(e) => return e.c_ptr(),
    };
    let path = match convert_ptr_string_to_string(path).map_err(|e| Response::error(e)) {
        Ok(v) => v,
        Err(e) => return e.c_ptr(),
    };
    let address = match convert_ptr_string_to_string(address).map_err(|e| Response::error(e)) {
        Ok(v) => Some(v).filter(|s| !s.is_empty()),
        Err(e) => return e.c_ptr(),
    };
    let origin = match convert_ptr_string_to_string(origin).map_err(|e| Response::error(e)) {
        Ok(v) => Some(v).filter(|s| !s.is_empty()),
        Err(e) => return e.c_ptr(),
    };

    let request = TronSignRequest::new(
        Some(request_id),
        sign_data,
        data_type,
        path,
        Some(xfp),
        address,
        origin,
    );
    Response::success_object(Box::into_raw(Box::new(request)) as PtrVoid).c_ptr()
}

#[no_mangle]
pub extern "C" fn tron_sign_request_get_ur_encoder(tron_sign_request: &mut TronSignRequest) -> PtrResponse {
    match tron_sign_request.to_bytes() {
        Ok(message) => {
            let ur_encoder = ur::Encoder::new(
                message.as_slice(),
                400,
                TRON_SIGN_REQUEST_TYPE,
            )
            .unwrap();
            Response::success_object(Box::into_raw(Box::new(ur_encoder)) as PtrVoid).c_ptr()
        }
        Err(e) => Response::error(e).c_ptr(),
    }
}

#[no_mangle]
pub extern "C" fn tron_sign_request_get_request_id(tron_sign_request: &mut TronSignRequest) -> PtrResponse {
    tron_sign_request.get_request_id().map_or(Response::success_null().c_ptr(), |id| {
        Response::success_string(hex::encode(id)).c_ptr()
    })
}

#[no_mangle]
pub extern "C" fn tron_sign_request_get_sign_data(tron_sign_request: &mut TronSignRequest) -> PtrResponse {
    Response::success_string(hex::encode(tron_sign_request.get_sign_data())).c_ptr()
}

#[no_mangle]
pub extern "C" fn tron_sign_request_get_derivation_path(tron_sign_request: &mut TronSignRequest) -> PtrResponse {
    Response::success_string(tron_sign_request.get_derivation_path().to_string()).c_ptr()
}
