//! Facet-based capability security for tuple spaces.
//!
//! Facets provide restricted access to tuple spaces through:
//! - Namespace isolation
//! - Permission-based access control

use crate::error::{Error, Result};
use crate::tuplespace::{Tuple, TuplePattern};
use smol_str::SmolStr;
use std::sync::{Arc, Mutex};

/// Permission flags for tuple space operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TuplePermissions {
    /// Permission to write tuples (out operation)
    pub can_out: bool,
    /// Permission to remove tuples (in operation)
    pub can_in: bool,
    /// Permission to read tuples (rd operation)
    pub can_rd: bool,
}

impl TuplePermissions {
    /// Create permissions with all operations allowed.
    pub fn all() -> Self {
        Self {
            can_out: true,
            can_in: true,
            can_rd: true,
        }
    }

    /// Create read-only permissions (rd only).
    pub fn readonly() -> Self {
        Self {
            can_out: false,
            can_in: false,
            can_rd: true,
        }
    }

    /// Create write-only permissions (out only).
    pub fn writeonly() -> Self {
        Self {
            can_out: true,
            can_in: false,
            can_rd: false,
        }
    }
}

/// A facet-restricted view of a tuple space.
///
/// Facets provide capability-based access control to tuple spaces,
/// allowing namespace isolation and permission restrictions.
#[derive(Debug, Clone)]
pub struct TupleSpaceFacet {
    /// The underlying tuple space
    space: Arc<Mutex<crate::tuplespace::store::TupleSpace>>,
    /// Namespace restriction (None = no restriction)
    namespace: Option<SmolStr>,
    /// Operation permissions
    permissions: TuplePermissions,
}

impl TupleSpaceFacet {
    /// Create a new facet with full permissions and no namespace restriction.
    pub fn new(space: Arc<Mutex<crate::tuplespace::store::TupleSpace>>) -> Self {
        Self {
            space,
            namespace: None,
            permissions: TuplePermissions::all(),
        }
    }

    /// Create a facet restricted to a specific namespace.
    pub fn with_namespace(mut self, namespace: SmolStr) -> Self {
        self.namespace = Some(namespace);
        self
    }

    /// Create a facet with specific permissions.
    pub fn with_permissions(mut self, permissions: TuplePermissions) -> Self {
        self.permissions = permissions;
        self
    }

    /// Create a read-only facet.
    pub fn readonly(self) -> Self {
        self.with_permissions(TuplePermissions::readonly())
    }

    /// Create a write-only facet.
    pub fn writeonly(self) -> Self {
        self.with_permissions(TuplePermissions::writeonly())
    }

    /// Write a tuple to the space.
    pub fn out(&mut self, type_name: SmolStr, data: crate::value::Value) -> Result<()> {
        if !self.permissions.can_out {
            return Err(Error::Runtime(
                "permission denied: out operation not allowed".to_string(),
            ));
        }

        let mut tuple = Tuple::new(type_name, data);
        // Apply namespace restriction
        if let Some(ns) = &self.namespace {
            tuple.namespace = Some(ns.clone());
        }

        let mut space = self.space.lock().unwrap();
        space.out(tuple)
    }

    /// Remove a matching tuple (blocking).
    pub fn r#in(&mut self, pattern: &TuplePattern) -> Result<Tuple> {
        if !self.permissions.can_in {
            return Err(Error::Runtime(
                "permission denied: in operation not allowed".to_string(),
            ));
        }

        let restricted_pattern = self.apply_namespace_restriction(pattern.clone());
        let mut space = self.space.lock().unwrap();
        space.r#in(&restricted_pattern)
    }

    /// Read a matching tuple (blocking, non-destructive).
    pub fn rd(&mut self, pattern: &TuplePattern) -> Result<Tuple> {
        if !self.permissions.can_rd {
            return Err(Error::Runtime(
                "permission denied: rd operation not allowed".to_string(),
            ));
        }

        let restricted_pattern = self.apply_namespace_restriction(pattern.clone());
        let mut space = self.space.lock().unwrap();
        space.rd(&restricted_pattern)
    }

    /// Non-blocking remove.
    pub fn inp(&mut self, pattern: &TuplePattern) -> Result<Option<Tuple>> {
        if !self.permissions.can_in {
            return Err(Error::Runtime(
                "permission denied: inp operation not allowed".to_string(),
            ));
        }

        let restricted_pattern = self.apply_namespace_restriction(pattern.clone());
        let mut space = self.space.lock().unwrap();
        space.inp(&restricted_pattern)
    }

    /// Non-blocking read.
    pub fn rdp(&mut self, pattern: &TuplePattern) -> Result<Option<Tuple>> {
        if !self.permissions.can_rd {
            return Err(Error::Runtime(
                "permission denied: rdp operation not allowed".to_string(),
            ));
        }

        let restricted_pattern = self.apply_namespace_restriction(pattern.clone());
        let mut space = self.space.lock().unwrap();
        space.rdp(&restricted_pattern)
    }

    /// Apply namespace restriction to a pattern.
    fn apply_namespace_restriction(&self, pattern: TuplePattern) -> TuplePattern {
        if let Some(ns) = &self.namespace {
            // Force the pattern to only match our namespace
            match pattern {
                TuplePattern::TypeAndData { type_name, data } => TuplePattern::Full {
                    namespace: ns.clone(),
                    type_name,
                    data,
                },
                TuplePattern::Full {
                    namespace: _,
                    type_name,
                    data,
                } => TuplePattern::Full {
                    namespace: ns.clone(),
                    type_name,
                    data,
                },
                TuplePattern::Any => TuplePattern::Full {
                    namespace: ns.clone(),
                    type_name: SmolStr::new("*"),
                    data: crate::tuplespace::Pattern::Wildcard,
                },
            }
        } else {
            // No restriction, keep pattern as-is
            pattern
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuplespace::Pattern;
    use crate::value::Value;
    use smol_str::SmolStr;

    #[test]
    fn test_permissions_all() {
        let perms = TuplePermissions::all();
        assert!(perms.can_out);
        assert!(perms.can_in);
        assert!(perms.can_rd);
    }

    #[test]
    fn test_permissions_readonly() {
        let perms = TuplePermissions::readonly();
        assert!(!perms.can_out);
        assert!(!perms.can_in);
        assert!(perms.can_rd);
    }

    #[test]
    fn test_permissions_writeonly() {
        let perms = TuplePermissions::writeonly();
        assert!(perms.can_out);
        assert!(!perms.can_in);
        assert!(!perms.can_rd);
    }

    #[test]
    fn test_facet_namespace_restriction() {
        use crate::tuplespace::store::TupleSpace;
        use std::sync::Arc;

        let space = Arc::new(Mutex::new(TupleSpace::new()));
        let mut facet = TupleSpaceFacet::new(space.clone()).with_namespace(SmolStr::new("user_1"));

        // Write through facet - should get namespace applied
        facet
            .out(SmolStr::new("event"), Value::String("data".into()))
            .unwrap();

        // Directly check the space - tuple should have namespace
        let mut space_guard = space.lock().unwrap();
        let pattern = TuplePattern::Full {
            namespace: SmolStr::new("user_1"),
            type_name: SmolStr::new("event"),
            data: Pattern::Wildcard,
        };
        assert!(space_guard.rdp(&pattern).unwrap().is_some());

        // Different namespace should not match
        let pattern2 = TuplePattern::Full {
            namespace: SmolStr::new("user_2"),
            type_name: SmolStr::new("event"),
            data: Pattern::Wildcard,
        };
        assert!(space_guard.rdp(&pattern2).unwrap().is_none());
    }

    #[test]
    fn test_facet_readonly_permission() {
        use crate::tuplespace::store::TupleSpace;
        use std::sync::Arc;

        let space = Arc::new(Mutex::new(TupleSpace::new()));
        let mut facet = TupleSpaceFacet::new(space.clone()).readonly();

        // Add a tuple directly to space
        {
            let mut space_guard = space.lock().unwrap();
            space_guard
                .out(Tuple::new(SmolStr::new("event"), Value::Int(42)))
                .unwrap();
        }

        // Readonly should allow rd
        let pattern = TuplePattern::Any;
        assert!(facet.rd(&pattern).is_ok());

        // readonly should reject out
        assert!(facet.out(SmolStr::new("event"), Value::Int(99)).is_err());
    }

    #[test]
    fn test_facet_writeonly_permission() {
        use crate::tuplespace::store::TupleSpace;
        use std::sync::Arc;

        let space = Arc::new(Mutex::new(TupleSpace::new()));
        let mut facet = TupleSpaceFacet::new(space).writeonly();

        // Write should succeed
        assert!(facet.out(SmolStr::new("event"), Value::Int(42)).is_ok());

        // Read should fail
        let pattern = TuplePattern::Any;
        assert!(facet.rd(&pattern).is_err());
    }
}
