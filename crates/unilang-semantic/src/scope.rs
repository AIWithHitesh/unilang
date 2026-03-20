// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Scope management for name resolution.
//!
//! Maintains a stack of lexical scopes. Each scope tracks its own symbols
//! and has a parent pointer for walking up the chain.

use std::collections::HashMap;

use unilang_common::span::Span;

use crate::types::Type;

/// Describes what kind of symbol a name refers to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Function,
    Class,
    Parameter,
    Method,
    Field,
}

/// A resolved symbol in a scope.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub kind: SymbolKind,
    pub mutable: bool,
    pub span: Span,
}

/// What kind of scope this is (affects control-flow validation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Module,
    Function,
    Class,
    Block,
    Loop,
}

/// A single lexical scope.
#[derive(Debug)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>,
    pub kind: ScopeKind,
}

impl Scope {
    fn new(kind: ScopeKind, parent: Option<usize>) -> Self {
        Self {
            symbols: HashMap::new(),
            parent,
            kind,
        }
    }
}

/// Stack of nested scopes with parent pointers for chain-walking resolution.
#[derive(Debug)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
    current: usize,
}

impl ScopeStack {
    pub fn new() -> Self {
        let module_scope = Scope::new(ScopeKind::Module, None);
        Self {
            scopes: vec![module_scope],
            current: 0,
        }
    }

    /// Push a new scope of the given kind, with the current scope as parent.
    pub fn push_scope(&mut self, kind: ScopeKind) {
        let parent = self.current;
        let idx = self.scopes.len();
        self.scopes.push(Scope::new(kind, Some(parent)));
        self.current = idx;
    }

    /// Pop the current scope, returning to the parent.
    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current].parent {
            self.current = parent;
        }
    }

    /// Define a symbol in the current scope.
    /// Returns `Err(existing_span)` if the name is already defined in the current scope.
    pub fn define(&mut self, name: &str, symbol: Symbol) -> Result<(), Span> {
        let scope = &mut self.scopes[self.current];
        if let Some(existing) = scope.symbols.get(name) {
            return Err(existing.span);
        }
        scope.symbols.insert(name.to_string(), symbol);
        Ok(())
    }

    /// Resolve a name by walking up the parent chain.
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        let mut idx = self.current;
        loop {
            if let Some(sym) = self.scopes[idx].symbols.get(name) {
                return Some(sym);
            }
            match self.scopes[idx].parent {
                Some(parent) => idx = parent,
                None => return None,
            }
        }
    }

    /// Check if we are inside a scope of the given kind (walking up).
    pub fn is_inside(&self, kind: ScopeKind) -> bool {
        let mut idx = self.current;
        loop {
            if self.scopes[idx].kind == kind {
                return true;
            }
            match self.scopes[idx].parent {
                Some(parent) => idx = parent,
                None => return false,
            }
        }
    }

    /// Returns the current scope kind.
    pub fn current_kind(&self) -> ScopeKind {
        self.scopes[self.current].kind
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}
