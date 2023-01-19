#pragma once

#include "nlohmann/json.hpp"
#include <optional>
#include <string>
#include <vector>

namespace ga::sdk::networks {

enum class network_type {
    mainnet,
    liquid,
    lightning,
};

enum class server_type {
    green,
    electrum,
    greenlight,
};

struct network_config {
    std::string address_explorer_url;
    std::string asset_registry_url;
    std::string asset_registry_onion_url;
    std::string bech32_prefix;
    std::string bip21_prefix;
    std::string blech32_prefix;
    uint32_t blinded_prefix;
    std::vector<uint32_t> csv_buckets;
    bool development;
    bool electrum_tls;
    std::string electrum_url;
    std::string electrum_onion_url;
    std::string pin_server_url;
    std::string pin_server_onion_url;
    std::string pin_server_public_key;
    std::string price_url;
    std::string price_onion_url;
    bool liquid;
    bool mainnet;
    uint_fast32_t max_reorg_blocks;
    std::string name;
    std::string network;
    uint32_t p2pkh_version;
    uint32_t p2sh_version;
    std::string policy_asset;
    std::string server_type;
    std::string service_chain_code;
    std::string service_pubkey;
    bool spv_multi;
    std::vector<std::string> spv_servers;
    bool spv_enabled;
    std::string tx_explorer_url;
    std::vector<std::string> wamp_cert_pins;
    std::vector<std::string> wamp_cert_roots;
    std::string wamp_url;
    std::string wamp_onion_url;
    std::string greenlight_url;
    bool lightning;
};

NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(network_config, address_explorer_url, bech32_prefix, bip21_prefix, csv_buckets,
    development, electrum_tls, electrum_url, electrum_onion_url, pin_server_url, pin_server_onion_url,
    pin_server_public_key, price_url, price_onion_url, liquid, mainnet, max_reorg_blocks, name, network, p2pkh_version,
    p2sh_version, server_type, service_chain_code, service_pubkey, spv_multi, spv_servers, spv_enabled, tx_explorer_url,
    wamp_cert_pins, wamp_cert_roots, wamp_onion_url, wamp_url, greenlight_url, lightning, asset_registry_url,
    asset_registry_onion_url)

} // namespace ga::sdk::networks
