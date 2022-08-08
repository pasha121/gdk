use super::BEScript;

use crate::error::Error;
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
