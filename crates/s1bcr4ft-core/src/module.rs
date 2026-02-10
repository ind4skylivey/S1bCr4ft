use crate::error::{Result, S1bCr4ftError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Module metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// Module identifier (e.g., "core/base-system")
    pub id: String,

    /// Module name
    pub name: String,

    /// Description
    pub description: String,

    /// Category
    pub category: String,

    /// Version
    pub version: String,

    /// Dependencies (other module IDs)
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Conflicts with (module IDs)
    #[serde(default)]
    pub conflicts: Vec<String>,

    /// Packages to install
    pub packages: Vec<String>,

    /// AUR packages
    #[serde(default)]
    pub aur_packages: Vec<String>,

    /// System commands to run
    #[serde(default)]
    pub commands: Vec<String>,

    /// Files to create/modify
    #[serde(default)]
    pub files: HashMap<PathBuf, String>,
}

/// Module registry
pub struct ModuleRegistry {
    modules: HashMap<String, Module>,
    module_dir: PathBuf,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new<P: AsRef<Path>>(module_dir: P) -> Self {
        Self {
            modules: HashMap::new(),
            module_dir: module_dir.as_ref().to_path_buf(),
        }
    }

    /// Load all modules from directory
    pub fn load_all(&mut self) -> Result<()> {
        if !self.module_dir.exists() {
            return Ok(());
        }

        for entry in walkdir::WalkDir::new(&self.module_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "module.yml" {
                let module = self.load_module(entry.path())?;
                self.modules.insert(module.id.clone(), module);
            }
        }

        Ok(())
    }

    /// Load a single module
    fn load_module<P: AsRef<Path>>(&self, path: P) -> Result<Module> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| S1bCr4ftError::module(format!("Failed to read module: {}", e)))?;

        let module: Module = serde_yaml::from_str(&content)?;
        Ok(module)
    }

    /// Get module by ID
    pub fn get(&self, id: &str) -> Option<&Module> {
        self.modules.get(id)
    }

    /// List all modules
    pub fn list(&self) -> Vec<&Module> {
        self.modules.values().collect()
    }

    /// Search modules by query
    pub fn search(&self, query: &str) -> Vec<&Module> {
        let query_lower = query.to_lowercase();
        self.modules
            .values()
            .filter(|m| {
                m.name.to_lowercase().contains(&query_lower)
                    || m.description.to_lowercase().contains(&query_lower)
                    || m.id.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

/// Module dependency resolver
pub struct ModuleResolver<'a> {
    registry: &'a ModuleRegistry,
}

impl<'a> ModuleResolver<'a> {
    pub fn new(registry: &'a ModuleRegistry) -> Self {
        Self { registry }
    }

    /// Resolve module dependencies in topological order
    pub fn resolve(&self, module_ids: &[String]) -> Result<Vec<String>> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for id in module_ids {
            self.visit(id, &mut resolved, &mut visited, &mut visiting)?;
        }

        Ok(resolved)
    }

    fn visit(
        &self,
        id: &str,
        resolved: &mut Vec<String>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
    ) -> Result<()> {
        if visited.contains(id) {
            return Ok(());
        }

        if visiting.contains(id) {
            return Err(S1bCr4ftError::Dependency(format!(
                "Circular dependency detected: {}",
                id
            )));
        }

        let module = self
            .registry
            .get(id)
            .ok_or_else(|| S1bCr4ftError::module(format!("Module not found: {}", id)))?;

        visiting.insert(id.to_string());

        for dep in &module.dependencies {
            self.visit(dep, resolved, visited, visiting)?;
        }

        visiting.remove(id);
        visited.insert(id.to_string());
        resolved.push(id.to_string());

        Ok(())
    }

    /// Check for conflicts
    pub fn check_conflicts(&self, module_ids: &[String]) -> Result<()> {
        let modules: Vec<_> = module_ids
            .iter()
            .map(|id| {
                self.registry
                    .get(id)
                    .ok_or_else(|| S1bCr4ftError::module(format!("Module not found: {}", id)))
            })
            .collect::<Result<_>>()?;

        for module in modules.iter() {
            for conflict in &module.conflicts {
                if module_ids.contains(conflict) {
                    return Err(S1bCr4ftError::Dependency(format!(
                        "Conflict detected: {} conflicts with {}",
                        module.id, conflict
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registry() {
        let registry = ModuleRegistry::new("/tmp/modules");
        assert_eq!(registry.list().len(), 0);
    }
}
