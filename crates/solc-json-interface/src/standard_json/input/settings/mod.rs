//! The `solc --standard-json` input settings.

pub mod metadata;
pub mod metadata_hash;
pub mod optimizer;
pub mod polkavm;
pub mod selection;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use self::metadata::Metadata;
use self::optimizer::Optimizer;
use self::polkavm::PolkaVM;
use self::selection::Selection;

/// The `solc --standard-json` input settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// The target EVM version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_version: Option<revive_common::EVMVersion>,
    /// The linker library addresses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libraries: Option<BTreeMap<String, BTreeMap<String, String>>>,
    /// The sorted list of remappings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remappings: Option<BTreeSet<String>>,
    /// The output selection filters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_selection: Option<Selection>,
    /// Whether to compile via IR. Only for testing with solc >=0.8.13.
    #[serde(
        rename = "viaIR",
        skip_serializing_if = "Option::is_none",
        skip_deserializing
    )]
    pub via_ir: Option<bool>,
    /// The optimizer settings.
    pub optimizer: Optimizer,
    /// The metadata settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// The resolc custom PolkaVM settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polkavm: Option<PolkaVM>,
}

impl Settings {
    /// A shortcut constructor.
    pub fn new(
        evm_version: Option<revive_common::EVMVersion>,
        libraries: BTreeMap<String, BTreeMap<String, String>>,
        remappings: Option<BTreeSet<String>>,
        output_selection: Selection,
        optimizer: Optimizer,
        metadata: Option<Metadata>,
        polkavm: Option<PolkaVM>,
    ) -> Self {
        Self {
            evm_version,
            libraries: Some(libraries),
            remappings,
            output_selection: Some(output_selection),
            optimizer,
            metadata,
            via_ir: Some(true),
            polkavm,
        }
    }

    /// Sets the necessary defaults.
    pub fn normalize(&mut self) {
        self.polkavm = None;
        self.optimizer.normalize();
    }

    /// Parses the library list and returns their double hashmap with path and name as keys.
    pub fn parse_libraries(
        input: Vec<String>,
    ) -> anyhow::Result<BTreeMap<String, BTreeMap<String, String>>> {
        let mut libraries = BTreeMap::new();
        for (index, library) in input.into_iter().enumerate() {
            let mut path_and_address = library.split('=');
            let path = path_and_address
                .next()
                .ok_or_else(|| anyhow::anyhow!("The library #{} path is missing", index))?;
            let mut file_and_contract = path.split(':');
            let file = file_and_contract
                .next()
                .ok_or_else(|| anyhow::anyhow!("The library `{}` file name is missing", path))?;
            let contract = file_and_contract.next().ok_or_else(|| {
                anyhow::anyhow!("The library `{}` contract name is missing", path)
            })?;
            let address = path_and_address
                .next()
                .ok_or_else(|| anyhow::anyhow!("The library `{}` address is missing", path))?;
            libraries
                .entry(file.to_owned())
                .or_insert_with(BTreeMap::new)
                .insert(contract.to_owned(), address.to_owned());
        }
        Ok(libraries)
    }
}
