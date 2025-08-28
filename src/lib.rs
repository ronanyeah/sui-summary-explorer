pub mod parser;
pub mod service;

use std::sync::Arc;

#[derive(Clone)]
pub struct PackageTree {
    packages: Arc<move_model_2::summary::Packages>,
}

impl PackageTree {
    pub fn new(dir: &std::path::Path) -> anyhow::Result<Self> {
        let pkg_path = std::path::Path::new(dir);
        let packages = parser::parse_summaries(pkg_path)?;
        Ok(Self {
            packages: Arc::new(packages),
        })
    }
    pub fn list_packages(&self) -> Vec<String> {
        let pkg = self
            .packages
            .packages
            .values()
            .filter_map(|pkg| pkg.name)
            .map(|pkg| pkg.to_string())
            .collect();
        pkg
    }
    pub fn list_modules(&self, package: &str) -> Vec<String> {
        let pkg = self.get_package(package).unwrap();
        let ms = pkg
            .modules
            .values()
            .map(|pkg| pkg.id.name.to_string())
            .collect();
        ms
    }
    pub fn get_package<'a>(&'a self, package: &str) -> Option<&'a move_model_2::summary::Package> {
        let pkg = self
            .packages
            .packages
            .values()
            .find(|pkg| pkg.name.map_or(false, |name| name.to_string() == package));
        pkg
    }
    pub fn get_module<'a>(
        &'a self,
        package: &str,
        module: &str,
    ) -> Option<&'a move_model_2::summary::Module> {
        let pkg = self.get_package(package)?;
        let res = pkg
            .modules
            .values()
            .find(|module_| module_.id.name.to_string() == module);
        res
    }
    pub fn get_function<'a>(
        &'a self,
        package: &str,
        module: &str,
        function_name: &str,
    ) -> Option<&'a move_model_2::summary::Function> {
        let mdd = self.get_module(package, module)?;
        mdd.functions
            .iter()
            .find(|(name, _)| name.to_string() == function_name)
            .map(|(_, v)| v)
    }
    pub fn get_enum<'a>(
        &'a self,
        package: &str,
        module: &str,
        enum_name: &str,
    ) -> Option<&'a move_model_2::summary::Enum> {
        let mdd = self.get_module(package, module)?;
        mdd.enums
            .iter()
            .find(|(name, _)| name.to_string() == enum_name)
            .map(|(_, v)| v)
    }
    pub fn get_struct<'a>(
        &'a self,
        package: &str,
        module: &str,
        struct_name: &str,
    ) -> Option<&'a move_model_2::summary::Struct> {
        let mdd = self.get_module(package, module)?;
        mdd.structs
            .iter()
            .find(|(name, _)| name.to_string() == struct_name)
            .map(|(_, v)| v)
    }
    pub fn get_definition(
        &self,
        package: &str,
        module: &str,
        definition_name: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let definition = if let Some(val) = self.get_function(package, module, definition_name) {
            serde_json::to_value(val)?
        } else if let Some(val) = self.get_struct(package, module, definition_name) {
            serde_json::to_value(val)?
        } else if let Some(val) = self.get_enum(package, module, definition_name) {
            serde_json::to_value(val)?
        } else {
            anyhow::bail!("definition not found")
        };

        Ok(definition)
    }
}
