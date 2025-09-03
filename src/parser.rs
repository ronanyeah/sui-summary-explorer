use anyhow::{anyhow, Context, Result};
use move_core_types::account_address::AccountAddress;
use move_symbol_pool::Symbol;
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::Path};

// Constants from move-cli/src/base/summary.rs
const ADDRESS_MAPPING_FILENAME: &str = "address_mapping";
const METADATA_FILENAME: &str = "root_package_metadata";
const JSON_EXT: &str = "json";

/// Parse summary files from a directory path and return the structures
/// that match the output_summaries function signature.
pub fn parse_summaries(summaries_dir: &Path) -> Result<move_model_2::summary::Packages> {
    // Validate directory exists
    if !summaries_dir.exists() {
        return Err(anyhow!(
            "Summaries directory does not exist: {}",
            summaries_dir.display()
        ));
    }

    if !summaries_dir.is_dir() {
        return Err(anyhow!(
            "Path is not a directory: {}",
            summaries_dir.display()
        ));
    }

    // Parse address mapping
    let address_mapping =
        parse_address_mapping(summaries_dir).context("Failed to parse address mapping")?;

    // Parse additional metadata (optional)
    let _additional_metadata = parse_metadata::<serde_json::Value>(summaries_dir)
        .context("Failed to parse additional metadata")?;

    // Parse packages and modules
    let packages =
        parse_packages(summaries_dir, &address_mapping).context("Failed to parse packages")?;

    Ok(packages)
}

/// Parse the address_mapping.json file
fn parse_address_mapping(summaries_dir: &Path) -> Result<BTreeMap<Symbol, AccountAddress>> {
    let mapping_file = summaries_dir
        .join(ADDRESS_MAPPING_FILENAME)
        .with_extension(JSON_EXT);

    if !mapping_file.exists() {
        return Err(anyhow!(
            "Address mapping file not found: {}",
            mapping_file.display()
        ));
    }

    let content = fs::read_to_string(&mapping_file).with_context(|| {
        format!(
            "Failed to read address mapping file: {}",
            mapping_file.display()
        )
    })?;

    // The JSON contains String representations of addresses, we need to convert to AccountAddress
    let string_mapping: BTreeMap<Symbol, String> =
        serde_json::from_str(&content).with_context(|| {
            format!(
                "Failed to parse address mapping JSON from: {}",
                mapping_file.display()
            )
        })?;

    let mut address_mapping = BTreeMap::new();
    for (symbol, addr_str) in string_mapping {
        let address = AccountAddress::from_hex_literal(&addr_str).with_context(|| {
            format!(
                "Failed to parse address '{}' for symbol '{}'",
                addr_str, symbol
            )
        })?;
        address_mapping.insert(symbol, address);
    }

    Ok(address_mapping)
}

/// Parse the optional root_package_metadata.json file
fn parse_metadata<T: for<'de> Deserialize<'de>>(summaries_dir: &Path) -> Result<Option<T>> {
    let metadata_file = summaries_dir
        .join(METADATA_FILENAME)
        .with_extension(JSON_EXT);

    if !metadata_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&metadata_file)
        .with_context(|| format!("Failed to read metadata file: {}", metadata_file.display()))?;

    let metadata: T = serde_json::from_str(&content).with_context(|| {
        format!(
            "Failed to parse metadata JSON from: {}",
            metadata_file.display()
        )
    })?;

    Ok(Some(metadata))
}

/// Parse all package directories and their module files
fn parse_packages(
    summaries_dir: &Path,
    address_mapping: &BTreeMap<Symbol, AccountAddress>,
) -> Result<move_model_2::summary::Packages> {
    let mut packages = BTreeMap::new();

    // Read all entries in the summaries directory
    let entries = fs::read_dir(summaries_dir).with_context(|| {
        format!(
            "Failed to read summaries directory: {}",
            summaries_dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip files (we only want package directories)
        if !path.is_dir() {
            continue;
        }

        let package_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("Invalid package directory name")?;

        let package_symbol = Symbol::from(package_name);

        let pkg_read = address_mapping.get(&package_symbol);

        let Some(package_address) = pkg_read else {
            println!(
                "Ignoring package '{}' - symbol not found in address_mapping.json",
                package_symbol
            );
            continue;
        };

        // Parse all modules in this package directory
        let modules = parse_modules_in_package(&path)
            .with_context(|| format!("Failed to parse modules in package: {}", package_name))?;

        let package = move_model_2::summary::Package {
            name: Some(Symbol::from(package_name)),
            modules,
        };

        packages.insert(*package_address, package);
    }

    Ok(move_model_2::summary::Packages { packages })
}

/// Parse all module JSON files in a package directory
fn parse_modules_in_package(
    package_dir: &Path,
) -> Result<BTreeMap<Symbol, move_model_2::summary::Module>> {
    let mut modules = BTreeMap::new();

    let entries = fs::read_dir(package_dir).with_context(|| {
        format!(
            "Failed to read package directory: {}",
            package_dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        let module_name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .context("Invalid module file name")?;

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read module file: {}", path.display()))?;

        let module: move_model_2::summary::Module = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse module JSON from: {}", path.display()))?;

        modules.insert(Symbol::from(module_name), module);
    }

    Ok(modules)
}
