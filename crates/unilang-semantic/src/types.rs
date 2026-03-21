// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Type system for UniLang semantic analysis.
//!
//! Supports gradual typing: `Dynamic` is assignable to/from any type,
//! enabling Python-style flexibility alongside Java-style static checks.

use unilang_parser::ast::TypeExpr;

/// Represents a resolved type in the UniLang type system.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Double,
    Bool,
    String,
    Char,
    Void,
    Null,
    /// Python-style untyped value.
    Dynamic,
    /// User-defined class type.
    Class(std::string::String),
    /// Array type `T[]`.
    Array(Box<Type>),
    /// Generic type like `List<T>` or `Map<K, V>`.
    Generic(std::string::String, Vec<Type>),
    /// Optional type `?T`.
    Optional(Box<Type>),
    /// Union type `T | U`.
    Union(Vec<Type>),
    /// Function type `(params) -> return`.
    Function(Vec<Type>, Box<Type>),
    /// Not yet resolved.
    Unknown,
    /// Error recovery placeholder.
    Error,
}

impl Type {
    /// Returns `true` if `self` can accept a value of type `other`.
    ///
    /// Gradual typing: `Dynamic` is assignable to/from anything.
    /// Supports implicit widening conversions and cross-syntax interop.
    pub fn is_assignable_from(&self, other: &Type) -> bool {
        // Error and Dynamic always pass
        if matches!(self, Type::Error)
            || matches!(other, Type::Error)
            || matches!(self, Type::Dynamic)
            || matches!(other, Type::Dynamic)
            || matches!(self, Type::Unknown)
            || matches!(other, Type::Unknown)
        {
            return true;
        }

        match (self, other) {
            // Same primitive types
            (Type::Int, Type::Int)
            | (Type::Float, Type::Float)
            | (Type::Double, Type::Double)
            | (Type::Bool, Type::Bool)
            | (Type::String, Type::String)
            | (Type::Char, Type::Char)
            | (Type::Void, Type::Void) => true,

            // Null is assignable to Optional, class types, and Dynamic
            (Type::Optional(_), Type::Null) => true,
            (Type::Class(_), Type::Null) => true,

            // Numeric widening: Int → Float → Double
            (Type::Float, Type::Int) | (Type::Double, Type::Int) | (Type::Double, Type::Float) => {
                true
            }

            // Implicit narrowing with coercion (e.g., int x = 5.0)
            (Type::Int, Type::Float) | (Type::Int, Type::Double) | (Type::Float, Type::Double) => {
                true
            }

            // Implicit toString: String accepts any type
            (Type::String, _) => true,

            // Same class name
            (Type::Class(a), Type::Class(b)) => a == b,

            // Array covariance (simplified)
            (Type::Array(a), Type::Array(b)) => a.is_assignable_from(b),

            // Optional unwrapping: T? accepts T
            (Type::Optional(inner), other_ty) => inner.is_assignable_from(other_ty),

            // Union: target union accepts if any member accepts
            (Type::Union(members), other_ty) => {
                members.iter().any(|m| m.is_assignable_from(other_ty))
            }

            // Source union: each member must be assignable to target
            (target, Type::Union(members)) => members.iter().all(|m| target.is_assignable_from(m)),

            _ => false,
        }
    }

    /// Returns `true` if a value of this type can be implicitly coerced to `target`.
    ///
    /// This is broader than `is_assignable_from` — it includes:
    /// - Numeric widening: Int → Float → Double
    /// - String conversion: any type can be coerced to String (for print/concat)
    /// - Dynamic compatibility: Dynamic interops with everything
    pub fn can_coerce_to(&self, target: &Type) -> bool {
        // Error/Unknown/Dynamic always coerce
        if matches!(self, Type::Error | Type::Unknown | Type::Dynamic)
            || matches!(target, Type::Error | Type::Unknown | Type::Dynamic)
        {
            return true;
        }

        // Same type always coerces
        if self == target {
            return true;
        }

        // Any type can coerce to String (implicit toString)
        if matches!(target, Type::String) {
            return true;
        }

        // Numeric widening and narrowing
        if self.is_numeric() && target.is_numeric() {
            return true;
        }

        // Null coerces to Optional
        if matches!(self, Type::Null) && matches!(target, Type::Optional(_)) {
            return true;
        }

        // Null coerces to Dynamic
        if matches!(self, Type::Null) && matches!(target, Type::Dynamic) {
            return true;
        }

        // Delegate to is_assignable_from for the rest
        target.is_assignable_from(self)
    }

    /// Returns the common type for binary operations between `a` and `b`.
    ///
    /// - Int op Int → Int
    /// - Int op Float → Float (promote Int)
    /// - Int op Double → Double
    /// - Float op Double → Double
    /// - Dynamic op anything → Dynamic
    /// - String + anything → String (concat)
    pub fn coercion_result(a: &Type, b: &Type) -> Type {
        // Same type → same type
        if a == b {
            return a.clone();
        }
        // Dynamic interop
        if matches!(a, Type::Dynamic) || matches!(b, Type::Dynamic) {
            return Type::Dynamic;
        }
        // Error recovery
        if matches!(a, Type::Error) || matches!(b, Type::Error) {
            return Type::Error;
        }
        // Unknown passes through
        if matches!(a, Type::Unknown) || matches!(b, Type::Unknown) {
            return Type::Dynamic;
        }
        // Numeric promotion
        if a.is_numeric() && b.is_numeric() {
            return match (a, b) {
                (Type::Double, _) | (_, Type::Double) => Type::Double,
                (Type::Float, _) | (_, Type::Float) => Type::Float,
                _ => Type::Int,
            };
        }
        // String concatenation: String + anything → String
        if matches!(a, Type::String) || matches!(b, Type::String) {
            return Type::String;
        }
        Type::Error
    }

    /// Returns `true` if this type is numeric (Int, Float, Double).
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Double)
    }

    /// Compute the common type of two types for binary operations.
    pub fn common_type(a: &Type, b: &Type) -> Type {
        if a == b {
            return a.clone();
        }
        if matches!(a, Type::Dynamic) || matches!(b, Type::Dynamic) {
            return Type::Dynamic;
        }
        if matches!(a, Type::Error) || matches!(b, Type::Error) {
            return Type::Error;
        }
        if a.is_numeric() && b.is_numeric() {
            // Widening: Int < Float < Double
            return match (a, b) {
                (Type::Double, _) | (_, Type::Double) => Type::Double,
                (Type::Float, _) | (_, Type::Float) => Type::Float,
                _ => Type::Int,
            };
        }
        // String concatenation
        if matches!(a, Type::String) || matches!(b, Type::String) {
            return Type::String;
        }
        Type::Error
    }

    /// Convert an AST `TypeExpr` into a resolved `Type`.
    pub fn from_type_expr(te: &TypeExpr) -> Type {
        match te {
            TypeExpr::Named(name) => match name.as_str() {
                "int" | "Int" | "Integer" | "long" | "Long" => Type::Int,
                "float" | "Float" => Type::Float,
                "double" | "Double" => Type::Double,
                "bool" | "Bool" | "Boolean" | "boolean" => Type::Bool,
                "str" | "String" | "string" => Type::String,
                "char" | "Char" | "Character" => Type::Char,
                "void" | "None" => Type::Void,
                other => Type::Class(other.to_string()),
            },
            TypeExpr::Qualified(parts) => Type::Class(parts.join(".")),
            TypeExpr::Array(inner) => Type::Array(Box::new(Type::from_type_expr(&inner.node))),
            TypeExpr::Optional(inner) => {
                Type::Optional(Box::new(Type::from_type_expr(&inner.node)))
            }
            TypeExpr::Union(members) => Type::Union(
                members
                    .iter()
                    .map(|m| Type::from_type_expr(&m.node))
                    .collect(),
            ),
            TypeExpr::Generic(base, args) => {
                let base_name = match &base.node {
                    TypeExpr::Named(n) => n.clone(),
                    _ => "Unknown".to_string(),
                };
                let type_args = args.iter().map(|a| Type::from_type_expr(&a.node)).collect();
                Type::Generic(base_name, type_args)
            }
            TypeExpr::Tuple(members) => Type::Generic(
                "Tuple".to_string(),
                members
                    .iter()
                    .map(|m| Type::from_type_expr(&m.node))
                    .collect(),
            ),
            TypeExpr::Inferred => Type::Unknown,
        }
    }

    /// Human-readable display name for error messages.
    pub fn display_name(&self) -> std::string::String {
        match self {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Double => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Char => "char".to_string(),
            Type::Void => "void".to_string(),
            Type::Null => "null".to_string(),
            Type::Dynamic => "dynamic".to_string(),
            Type::Class(name) => name.clone(),
            Type::Array(inner) => format!("{}[]", inner.display_name()),
            Type::Generic(name, args) => {
                let args_str: Vec<_> = args.iter().map(|a| a.display_name()).collect();
                format!("{}<{}>", name, args_str.join(", "))
            }
            Type::Optional(inner) => format!("?{}", inner.display_name()),
            Type::Union(members) => {
                let parts: Vec<_> = members.iter().map(|m| m.display_name()).collect();
                parts.join(" | ")
            }
            Type::Function(params, ret) => {
                let params_str: Vec<_> = params.iter().map(|p| p.display_name()).collect();
                format!("({}) -> {}", params_str.join(", "), ret.display_name())
            }
            Type::Unknown => "unknown".to_string(),
            Type::Error => "<error>".to_string(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
