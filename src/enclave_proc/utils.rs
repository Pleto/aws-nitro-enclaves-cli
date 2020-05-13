// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![deny(warnings)]

use std::fs::metadata;
use std::fs::File;
use std::io::Read;

use crate::common::NitroCliResult;

pub fn generate_enclave_id(slot_id: u64) -> NitroCliResult<String> {
    let file_path = "/sys/devices/virtual/dmi/id/board_asset_tag";
    if metadata(file_path).is_ok() {
        let mut file = File::open(file_path).map_err(|err| format!("{:?}", err))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|err| format!("{:?}", err))?;
        contents.retain(|c| !c.is_whitespace());
        return Ok(format!("{}-enc{:x}", contents, slot_id));
    }
    Ok(format!("i-0000000000000000-enc{:x}", slot_id))
}

pub fn get_slot_id(enclave_id: String) -> Result<u64, String> {
    let tokens: Vec<&str> = enclave_id.split("-enc").collect();

    match tokens.get(1) {
        Some(slot_id) => u64::from_str_radix(*slot_id, 16)
            .map_err(|_err| "Invalid enclave id format".to_string()),
        None => Err("Invalid enclave_id.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_enclave_id() {
        let slot_id: u64 = 7;
        let enc_id = generate_enclave_id(slot_id);
        let file_path = "/sys/devices/virtual/dmi/id/board_asset_tag";

        if !metadata(file_path).is_ok() {
            assert!(enc_id
                .unwrap()
                .eq(&format!("i-0000000000000000-enc{:?}", slot_id)));
        } else {
            assert!(!enc_id
                .unwrap()
                .split("-")
                .collect::<Vec<&str>>()
                .get(1)
                .unwrap()
                .eq(&"0000000000000000"));
        }
    }

    #[test]
    fn test_get_slot_id_valid() {
        let slot_id: u64 = 8;
        let enc_id = generate_enclave_id(slot_id);

        if let Ok(enc_id) = enc_id {
            let result = get_slot_id(enc_id);
            assert!(result.is_ok());
            assert_eq!(slot_id, result.unwrap());
        }
    }

    #[test]
    fn test_get_slot_id_invalid() {
        let enclave_id = String::from("i-0000_enc1234");
        let result = get_slot_id(enclave_id);

        assert!(result.is_err());
        if let Err(err_str) = result {
            assert!(err_str.eq("Invalid enclave_id."));
        }
    }
}
