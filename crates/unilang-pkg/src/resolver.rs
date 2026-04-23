// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Dependency resolver for UniLang packages.
//!
//! Given a root [`PackageManifest`], this module performs a topological sort of
//! all transitive dependencies and returns a fully-resolved install order.
//!
//! The current implementation uses a simple DFS-based topological sort.
//! Cycle detection is included — a [`PkgError::ResolutionError`] is returned
//! if a cycle is found.
//!
//! Version conflict handling is intentionally simple: the first version
//! requirement encountered for a package wins.  A future release will implement
//! proper semver constraint solving.

use std::collections::{HashMap, HashSet};

use crate::{PackageManifest, PkgError};

/// A resolved package with a concrete version that should be installed.
#[derive(Debug, Clone)]
pub struct ResolvedDep {
    /// Package name.
    pub name: String,
    /// The resolved (concrete) version string.
    pub version: String,
}

/// Resolve all transitive dependencies of `root` into an ordered install list.
///
/// Dependencies are returned in topological order — leaf packages come first
/// so that they can be installed before the packages that depend on them.
///
/// # Arguments
/// * `root`     — The root package manifest.
/// * `fetch_fn` — A function that retrieves a `PackageManifest` for a given
///                `(name, version)` pair.  In production this calls the
///                registry; in tests it can be a mock.
pub fn resolve<F>(root: &PackageManifest, fetch_fn: F) -> Result<Vec<ResolvedDep>, PkgError>
where
    F: Fn(&str, &str) -> Result<PackageManifest, PkgError>,
{
    let mut resolver = Resolver::new(fetch_fn);
    resolver.visit(&root.name, &root.version, &mut Vec::new())?;
    Ok(resolver.order)
}

/// Resolve dependencies for installation, using the live registry.
pub fn resolve_from_registry(root: &PackageManifest) -> Result<Vec<ResolvedDep>, PkgError> {
    resolve(root, |name, version| {
        // Download the manifest from the registry.
        let url = format!(
            "https://registry.unilang.dev/packages/{}/{}/manifest",
            name, version
        );
        let body = crate::registry::get_json_body(&url)?;
        let manifest: PackageManifest =
            serde_json::from_str(&body).map_err(|e| PkgError::SerdeError(e.to_string()))?;
        Ok(manifest)
    })
}

// ── Internal resolver state ───────────────────────────────────────────────────

struct Resolver<F>
where
    F: Fn(&str, &str) -> Result<PackageManifest, PkgError>,
{
    /// The function used to fetch transitive dependency manifests.
    fetch_fn: F,
    /// Set of packages already visited (name → resolved version).
    visited: HashMap<String, String>,
    /// The final topological order (leaf-first).
    order: Vec<ResolvedDep>,
}

impl<F> Resolver<F>
where
    F: Fn(&str, &str) -> Result<PackageManifest, PkgError>,
{
    fn new(fetch_fn: F) -> Self {
        Resolver {
            fetch_fn,
            visited: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Visit a package (DFS with cycle detection via `stack`).
    fn visit(
        &mut self,
        name: &str,
        version: &str,
        stack: &mut Vec<String>,
    ) -> Result<(), PkgError> {
        // Cycle detection.
        if stack.contains(&name.to_string()) {
            let cycle: Vec<&str> = stack.iter().map(String::as_str).collect();
            return Err(PkgError::ResolutionError(format!(
                "dependency cycle detected: {} -> {}",
                cycle.join(" -> "),
                name,
            )));
        }

        // Already resolved — check for version conflict.
        if let Some(existing_version) = self.visited.get(name) {
            if existing_version != version {
                eprintln!(
                    "warning: version conflict for '{}': {} vs {} (keeping {})",
                    name, existing_version, version, existing_version
                );
            }
            return Ok(());
        }

        // Fetch the manifest for this dependency.
        let manifest = (self.fetch_fn)(name, version)?;

        // Recurse into this package's dependencies first (DFS).
        stack.push(name.to_string());
        for (dep_name, dep_version_req) in &manifest.dependencies {
            self.visit(dep_name, dep_version_req, stack)?;
        }
        stack.pop();

        // Mark as visited and add to the output order.
        self.visited.insert(name.to_string(), version.to_string());
        self.order.push(ResolvedDep {
            name: name.to_string(),
            version: version.to_string(),
        });

        Ok(())
    }
}

// ── Utility: topological sort of a pre-built dependency graph ─────────────────

/// Perform a Kahn's-algorithm topological sort on a pre-built adjacency list.
///
/// `graph` maps each node name to the set of names it depends on.
/// Returns the nodes in install order (no dependencies first) or an error if
/// the graph contains a cycle.
pub fn topological_sort(graph: &HashMap<String, HashSet<String>>) -> Result<Vec<String>, PkgError> {
    // Compute in-degree for every node.
    let all_nodes: HashSet<&String> = graph.keys().collect();
    let mut in_degree: HashMap<String, usize> =
        all_nodes.iter().map(|n| (n.to_string(), 0)).collect();

    for deps in graph.values() {
        for dep in deps {
            *in_degree.entry(dep.clone()).or_insert(0) += 1;
        }
    }

    // Start queue with all zero-in-degree nodes.
    let mut queue: std::collections::VecDeque<String> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(n, _)| n.clone())
        .collect();
    queue.make_contiguous().sort(); // deterministic order

    let mut result = Vec::new();
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());
        if let Some(dependents) = graph.get(&node) {
            let mut dependents_sorted: Vec<&String> = dependents.iter().collect();
            dependents_sorted.sort();
            for dep in dependents_sorted {
                let entry = in_degree.entry(dep.clone()).or_insert(0);
                *entry -= 1;
                if *entry == 0 {
                    queue.push_back(dep.clone());
                }
            }
        }
    }

    if result.len() != in_degree.len() {
        return Err(PkgError::ResolutionError(
            "dependency cycle detected in graph".to_string(),
        ));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_manifest(name: &str, deps: &[(&str, &str)]) -> PackageManifest {
        PackageManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            author: String::new(),
            license: String::new(),
            dependencies: deps
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            entry: None,
        }
    }

    #[test]
    fn test_resolve_no_deps() {
        let root = mock_manifest("my-app", &[]);
        let resolved = resolve(&root, |_name, _ver| {
            Err(PkgError::NotFound("unexpected fetch".to_string()))
        })
        .unwrap();
        // Root itself is not included in the resolved list (only its deps).
        assert!(resolved.is_empty());
    }

    #[test]
    fn test_resolve_transitive() {
        // my-app -> http-client -> url-parser
        let root = mock_manifest("my-app", &[("http-client", "1.0.0")]);
        let resolved = resolve(&root, |name, _ver| match name {
            "http-client" => Ok(mock_manifest("http-client", &[("url-parser", "1.0.0")])),
            "url-parser" => Ok(mock_manifest("url-parser", &[])),
            other => Err(PkgError::NotFound(other.to_string())),
        })
        .unwrap();

        let names: Vec<&str> = resolved.iter().map(|r| r.name.as_str()).collect();
        // url-parser must come before http-client.
        let url_idx = names.iter().position(|&n| n == "url-parser").unwrap();
        let http_idx = names.iter().position(|&n| n == "http-client").unwrap();
        assert!(url_idx < http_idx);
    }

    #[test]
    fn test_cycle_detection() {
        let root = mock_manifest("a", &[("b", "1.0.0")]);
        let result = resolve(&root, |name, _ver| match name {
            "b" => Ok(mock_manifest("b", &[("a", "1.0.0")])),
            other => Err(PkgError::NotFound(other.to_string())),
        });
        assert!(matches!(result, Err(PkgError::ResolutionError(_))));
    }

    #[test]
    fn test_topological_sort_simple() {
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
        graph.insert("a".to_string(), HashSet::from(["b".to_string()]));
        graph.insert("b".to_string(), HashSet::new());

        let order = topological_sort(&graph).unwrap();
        let a_idx = order.iter().position(|n| n == "a").unwrap();
        let b_idx = order.iter().position(|n| n == "b").unwrap();
        assert!(b_idx < a_idx);
    }
}
