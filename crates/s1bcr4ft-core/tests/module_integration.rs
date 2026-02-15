//! Integration tests for module resolution
//!
//! Tests cover:
//! - Loading modules from directory
//! - Dependency resolution
//! - Conflict detection
//! - Circular dependency detection

mod common;

use common::ModuleFixture;
use s1bcr4ft_core::module::{ModuleRegistry, ModuleResolver};

/// Test loading modules from directory
#[test]
fn test_load_modules_from_directory() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().expect("Failed to load modules");

    let modules = registry.list();
    assert!(!modules.is_empty());

    // Should have loaded our test modules
    assert!(registry.get("core/base-system").is_some());
    assert!(registry.get("development/languages/rust").is_some());
    assert!(registry.get("development/languages/go").is_some());
}

/// Test getting a specific module
#[test]
fn test_get_module_by_id() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let module = registry.get("core/base-system").expect("Module not found");

    assert_eq!(module.id, "core/base-system");
    assert_eq!(module.name, "Base System");
    assert_eq!(module.category, "core");
    assert!(module.packages.contains(&"base".to_string()));
    assert!(module.packages.contains(&"linux".to_string()));
}

/// Test getting non-existent module
#[test]
fn test_get_nonexistent_module() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let module = registry.get("nonexistent/module");
    assert!(module.is_none());
}

/// Test searching modules
#[test]
fn test_search_modules() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    // Search by name
    let results = registry.search("Rust");
    assert!(!results.is_empty());

    // Search by category
    let results = registry.search("red-team");
    assert!(!results.is_empty());

    // Search by description
    let results = registry.search("programming");
    assert!(!results.is_empty());
}

/// Test dependency resolution
#[test]
fn test_resolve_dependencies() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let resolver = ModuleResolver::new(&registry);

    // Resolve rust which depends on base-system
    let resolved = resolver
        .resolve(&["development/languages/rust".to_string()])
        .expect("Failed to resolve dependencies");

    // base-system should come before rust
    assert!(resolved.contains(&"core/base-system".to_string()));
    assert!(resolved.contains(&"development/languages/rust".to_string()));

    let base_index = resolved
        .iter()
        .position(|m| m == "core/base-system")
        .unwrap();
    let rust_index = resolved
        .iter()
        .position(|m| m == "development/languages/rust")
        .unwrap();

    assert!(
        base_index < rust_index,
        "Dependencies should be resolved before dependents"
    );
}

/// Test dependency resolution with multiple modules
#[test]
fn test_resolve_multiple_modules() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let resolver = ModuleResolver::new(&registry);

    let resolved = resolver
        .resolve(&[
            "development/languages/rust".to_string(),
            "development/languages/go".to_string(),
        ])
        .expect("Failed to resolve dependencies");

    // Should include both languages and base-system
    assert!(resolved.contains(&"core/base-system".to_string()));
    assert!(resolved.contains(&"development/languages/rust".to_string()));
    assert!(resolved.contains(&"development/languages/go".to_string()));
}

/// Test dependency resolution with transitive dependencies
#[test]
fn test_resolve_transitive_dependencies() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let resolver = ModuleResolver::new(&registry);

    // sliver-c2 -> go -> base-system
    let resolved = resolver
        .resolve(&["red-team/c2-frameworks/sliver-c2".to_string()])
        .expect("Failed to resolve dependencies");

    // All transitive dependencies should be included
    assert!(resolved.contains(&"core/base-system".to_string()));
    assert!(resolved.contains(&"development/languages/go".to_string()));
    assert!(resolved.contains(&"red-team/c2-frameworks/sliver-c2".to_string()));

    // Order should be correct
    let base_index = resolved
        .iter()
        .position(|m| m == "core/base-system")
        .unwrap();
    let go_index = resolved
        .iter()
        .position(|m| m == "development/languages/go")
        .unwrap();
    let sliver_index = resolved
        .iter()
        .position(|m| m == "red-team/c2-frameworks/sliver-c2")
        .unwrap();

    assert!(base_index < go_index);
    assert!(go_index < sliver_index);
}

/// Test conflict detection
#[test]
fn test_detect_conflicts() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let resolver = ModuleResolver::new(&registry);

    // These modules conflict with each other
    let result = resolver.check_conflicts(&[
        "conflict/module-a".to_string(),
        "conflict/module-b".to_string(),
    ]);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Conflict detected"));
}

/// Test no conflicts with compatible modules
#[test]
fn test_no_conflicts_compatible() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let resolver = ModuleResolver::new(&registry);

    let result = resolver.check_conflicts(&[
        "development/languages/rust".to_string(),
        "development/languages/go".to_string(),
    ]);

    assert!(result.is_ok());
}

/// Test circular dependency detection
#[test]
fn test_detect_circular_dependencies() {
    let fixture = ModuleFixture::with_circular_deps();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let resolver = ModuleResolver::new(&registry);

    let result = resolver.resolve(&["circular/a".to_string()]);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Circular dependency"));
}

/// Test empty module registry
#[test]
fn test_empty_registry() {
    let fixture = ModuleFixture::empty();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry
        .load_all()
        .expect("Failed to load from empty directory");

    assert!(registry.list().is_empty());
}

/// Test resolving non-existent module
#[test]
fn test_resolve_nonexistent_module() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let resolver = ModuleResolver::new(&registry);

    let result = resolver.resolve(&["nonexistent/module".to_string()]);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Module not found"));
}

/// Test module listing
#[test]
fn test_list_all_modules() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let modules = registry.list();

    // Should have all our test modules
    assert!(modules.len() >= 4);

    // All modules should have required fields
    for module in &modules {
        assert!(!module.id.is_empty());
        assert!(!module.name.is_empty());
        assert!(!module.version.is_empty());
    }
}

/// Test module with AUR packages
#[test]
fn test_module_with_aur_packages() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let module = registry
        .get("red-team/c2-frameworks/sliver-c2")
        .expect("Module not found");

    assert!(!module.aur_packages.is_empty());
    assert!(module.aur_packages.contains(&"sliver".to_string()));
}

/// Test resolving multiple times (caching/consistency)
#[test]
fn test_resolve_consistency() {
    let fixture = ModuleFixture::with_sample_modules();

    let mut registry = ModuleRegistry::new(fixture.path());
    registry.load_all().unwrap();

    let resolver = ModuleResolver::new(&registry);

    let first = resolver
        .resolve(&["development/languages/rust".to_string()])
        .unwrap();
    let second = resolver
        .resolve(&["development/languages/rust".to_string()])
        .unwrap();

    assert_eq!(first, second);
}
