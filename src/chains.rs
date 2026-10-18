// src/chains.rs

/*!
 * Chains — pluggable detectors for the supported top-L1 daemons.
 *
 * Each detector is READ-ONLY: it locates the chain's CLI binary and asks it
 * for version/status. Detection never mutates the node.
 */

pub mod detect;

use serde::{Deserialize, Serialize};

/// A supported chain.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Dash,
    Bitcoin,
    BitcoinCash,
    Avalanche,
    Dogecoin,
    Litecoin,
}

impl Chain {
    /// All supported chains, for iteration during detection.
    pub const ALL: [Chain; 6] = [
        Chain::Dash,
        Chain::Bitcoin,
        Chain::BitcoinCash,
        Chain::Avalanche,
        Chain::Dogecoin,
        Chain::Litecoin,
    ];

    /// The CLI binary used to query this chain's node.
    pub fn cli(self) -> &'static str {
        match self {
            Chain::Dash => "dash-cli",
            Chain::Bitcoin => "bitcoin-cli",
            // bitcoin-cli serves BCHN/BU builds too; refined by detector.
            Chain::BitcoinCash => "bitcoin-cli",
            // Avalanche is queried over its HTTP admin API, not a CLI.
            Chain::Avalanche => "avalanchego",
            Chain::Dogecoin => "dogecoin-cli",
            Chain::Litecoin => "litecoin-cli",
        }
    }

    /// Human label.
    pub fn display(self) -> &'static str {
        match self {
            Chain::Dash => "Dash",
            Chain::Bitcoin => "Bitcoin",
            Chain::BitcoinCash => "Bitcoin Cash",
            Chain::Avalanche => "Avalanche",
            Chain::Dogecoin => "Dogecoin",
            Chain::Litecoin => "Litecoin",
        }
    }

    /// Machine name used in telemetry and the dashboard.
    pub fn name(self) -> &'static str {
        match self {
            Chain::Dash => "dash",
            Chain::Bitcoin => "bitcoin",
            Chain::BitcoinCash => "bitcoincash",
            Chain::Avalanche => "avalanche",
            Chain::Dogecoin => "dogecoin",
            Chain::Litecoin => "litecoin",
        }
    }
}
