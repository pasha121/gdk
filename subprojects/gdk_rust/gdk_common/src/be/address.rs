use super::BEScript;

use crate::error::Error;
use crate::model::{ValidateAddressParams, ValidateAddressResult};
use crate::NetworkId;

use std::str::FromStr;

#[derive(Debug)]
pub enum BEAddress {
    Bitcoin(bitcoin::Address),
    Elements(elements::Address),
}

impl BEAddress {
    pub fn script_pubkey(&self) -> BEScript {
        match self {
            BEAddress::Bitcoin(addr) => addr.script_pubkey().into(),
            BEAddress::Elements(addr) => addr.script_pubkey().into(),
        }
    }
    pub fn blinding_pubkey(&self) -> Option<bitcoin::secp256k1::PublicKey> {
        match self {
            BEAddress::Bitcoin(_) => None,
            BEAddress::Elements(addr) => addr.blinding_pubkey,
        }
    }

    pub fn from_str(address: &str, network_id: NetworkId) -> Result<Self, Error> {
        match network_id {
            NetworkId::Bitcoin(network) => {
                let address =
                    bitcoin::Address::from_str(&address).map_err(|_| Error::InvalidAddress)?;
                if address.network != network {
                    // regtest have some same prefixes wrt testnet
                    if network != bitcoin::Network::Regtest
                        || address.network != bitcoin::Network::Testnet
                    {
                        return Err(Error::InvalidAddress);
                    }
                }
                if !address.is_standard() {
                    return Err(Error::InvalidAddress);
                }
                Ok(BEAddress::Bitcoin(address))
            }
            NetworkId::Elements(network) => {
                let address =
                    elements::Address::parse_with_params(&address, network.address_params())
                        .map_err(|_| Error::InvalidAddress)?;
                if !address.is_blinded() {
                    return Err(Error::NonConfidentialAddress);
                }
                if let elements::address::Payload::WitnessProgram {
                    version: v,
                    program: p,
                } = &address.payload
                {
                    // Do not support segwit greater than v1 and non-P2TR v1
                    if v.to_u8() > 1 || (v.to_u8() == 1 && p.len() != 32) {
                        return Err(Error::InvalidAddress);
                    }
                }
                Ok(BEAddress::Elements(address))
            }
        }
    }
}

impl ToString for BEAddress {
    fn to_string(&self) -> String {
        match self {
            BEAddress::Bitcoin(addr) => addr.to_string(),
            BEAddress::Elements(addr) => addr.to_string(),
        }
    }
}

pub fn validate_address(param: &ValidateAddressParams) -> ValidateAddressResult {
    let (is_valid, error) = match BEAddress::from_str(&param.address, param.net_params.id()) {
        Ok(_) => (true, None),
        Err(e) => {
            let err_string = match e {
                Error::InvalidAddress => "id_invalid_address".to_string(),
                Error::NonConfidentialAddress => "id_nonconfidential_addresses_not".to_string(),
                _ => e.to_string(),
            };
            (false, Some(err_string))
        }
    };
    ValidateAddressResult {
        is_valid,
        error,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::NetworkParameters;

    fn test_validate_address(is_liquid: bool, test_cases: &[(&str, &str)]) {
        let mut net_params = NetworkParameters::default();
        net_params.development = true;
        net_params.mainnet = false;
        net_params.liquid = is_liquid;

        let mut params = ValidateAddressParams {
            address: "".to_string(),
            net_params,
        };

        for (address, expected_error) in test_cases {
            params.address = address.to_string();
            let res = validate_address(&params);
            if expected_error.is_empty() {
                assert!(res.is_valid);
                assert!(res.error.is_none());
            } else {
                assert!(!res.is_valid);
                assert_eq!(&res.error.unwrap(), expected_error);
            }
        }
    }

    #[test]
    fn test_validate_address_bitcoin() {
        let test_cases = [
            ("", "id_invalid_address"),
            ("mrLAf39iKU17Sw68tr1CCVp3o84dxKYmRv", ""), // pre-segwit
            ("bcrt1qtmp74ayg7p24uslctssvjm06q5phz4yrxucgnv", ""), // segwit v0 bech32
            (
                "bcrt1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqdmchcc",
                "id_invalid_address",
            ), // segwit v1 bech32
            ("bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7k35mrzd", "id_invalid_address"), // segwit v0 bech32m
            ("bcrt1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqc8gma6", ""), // segwit v1 bech32m
            (
                "bcrt1pw508d6qejxtdg4y5r3zarvary0c5xw7kw508d6qejxtdg4y5r3zarvary0c5xw7k0ylj56",
                "id_invalid_address",
            ), // segwit v1 bech32m non p2tr len
            ("bcrt1zw508d6qejxtdg4y5r3zarvaryv2wuatf", "id_invalid_address"), // segwit v2 bech32m
        ];
        test_validate_address(false, &test_cases);
    }

    #[test]
    fn test_validate_address_liquid() {
        let test_cases = [
            ("", "id_invalid_address"),
            ("CTEnoPJ5zCeaYB4uEkJFouWDsmapYbU9KUw5WHj9GNeHKNednZGphEE4F7uDQwJfnYxK3CPUfNs9Qnfz", ""), // pre-segwit
            ("el1qq0umk3pez693jrrlxz9ndlkuwne93gdu9g83mhhzuyf46e3mdzfpva0w48gqgzgrklncnm0k5zeyw8my2ypfsmxh4xcjh2rse", ""), // segwit v0 blech32
            ("el1pq0umk3pez693jrrlxz9ndlkuwne93gdu9g83mhhzuyf46e3mdzfpva0w48gqgzgrklncnm0k5zeyw8my2ypfsxguu9nrdg2pc", "id_invalid_address"), // segwit v1 blech32
            ("el1qq0umk3pez693jrrlxz9ndlkuwne93gdu9g83mhhzuyf46e3mdzfpva0w48gqgzgrklncnm0k5zeyw8my2ypfsmxh4xc8t604m", "id_invalid_address"), // segwit v0 blech32m
            ("el1pqdw8vgncs6ep0e4vcllwcvt8kr7z5e45z3qr4wsvnnq2qatsm3ejws3ylj93nn9qw0w7e5p20m06mp7hp33kt56nt0jtlw39md63p00wj7v4j5vahy5l", ""), // segwit v1 blech32m
            ("el1pq0umk3pez693jrrlxz9ndlkuwne93gdu9g83mhhzuyf46e3mdzfpvqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq87gd2ckgcugl", "id_invalid_address"), // segwit v1 blech32m non p2tr len
            ("el1zq0umk3pez693jrrlxz9ndlkuwne93gdu9g83mhhzuyf46e3mdzfpvqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqg3sqzyqnv9cq", "id_invalid_address"), // segwit v2 blech32m
            ("ert1qu6ssk77c466kg3x9wd82dqkd9udddykyfykm9k", "id_nonconfidential_addresses_not"), // non confidential
        ];
        test_validate_address(true, &test_cases);
    }
}
