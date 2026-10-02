use std::collections::HashMap;
use crate::ast::{Type, Param};
use crate::error::{BruteError, Result};
use crate::interpreter::Value;

/// Represents a trait definition
#[derive(Debug, Clone)]
pub struct Trait {
    /// Name of the trait
    pub name: String,

    /// Methods required by the trait
    pub methods: HashMap<String, TraitMethod>,

    /// Generic parameters for the trait
    pub generic_params: Vec<String>,
}

/// Represents a method signature in a trait
#[derive(Debug, Clone)]
pub struct TraitMethod {
    /// Name of the method
    pub name: String,

    /// Parameters of the method
    pub params: Vec<Param>,

    /// Return type of the method
    pub return_type: Type,

    /// Whether the method is async
    pub is_async: bool,

    /// Default implementation, if any
    pub default_impl: Option<Vec<crate::ast::Stmt>>,
}

/// Represents a trait implementation for a specific type
#[derive(Debug, Clone)]
pub struct TraitImpl {
    /// The trait being implemented
    pub trait_name: String,

    /// The type implementing the trait
    pub type_name: String,

    /// Methods implemented for the trait
    pub methods: HashMap<String, Value>,

    /// Generic parameters for the implementation
    pub generic_params: Vec<String>,
}

/// Registry for traits and their implementations
#[derive(Default, Clone)]
pub struct TraitRegistry {
    /// Map of trait name to trait definition
    traits: HashMap<String, Trait>,

    /// Map of (trait name, type name) to trait implementation
    impls: HashMap<(String, String), TraitImpl>,
}

impl TraitRegistry {
    /// Create a new trait registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a trait has been defined
    pub fn has_trait(&self, name: &str) -> bool {
        self.traits.contains_key(name)
    }

    /// Register a trait
    pub fn register_trait(&mut self, trait_def: Trait) {
        self.traits.insert(trait_def.name.clone(), trait_def);
    }

    /// Register a trait implementation
    pub fn register_impl(&mut self, impl_def: TraitImpl) -> Result<()> {
        // Check if the trait exists
        if !self.traits.contains_key(&impl_def.trait_name) {
            return Err(BruteError::TypeError(
                format!("Cannot implement unknown trait '{}'", impl_def.trait_name)
            ));
        }

        // Check if all required methods are implemented
        let trait_def = &self.traits[&impl_def.trait_name];

        for (method_name, method_def) in &trait_def.methods {
            if !impl_def.methods.contains_key(method_name) && method_def.default_impl.is_none() {
                return Err(BruteError::TypeError(
                    format!("Missing implementation for required method '{}' in trait '{}'",
                            method_name, impl_def.trait_name)
                ));
            }
        }

        // Register the implementation
        self.impls.insert((impl_def.trait_name.clone(), impl_def.type_name.clone()), impl_def);

        Ok(())
    }

    /// Check if a type implements a trait
    pub fn implements_trait(&self, trait_name: &str, type_name: &str) -> bool {
        self.impls.contains_key(&(trait_name.to_string(), type_name.to_string()))
    }

    /// All traits implemented for a type. `type_name` may be a base name
    /// (`Box`) while impls are registered under specialised names
    /// (`Box<int>`) — matching is done on the base part.
    pub fn traits_for_type(&self, type_name: &str) -> Vec<String> {
        let base = type_name.split('<').next().unwrap_or(type_name);
        self.impls.keys()
            .filter(|(_, ty)| ty.split('<').next().unwrap_or(ty) == base)
            .map(|(tr, _)| tr.clone())
            .collect()
    }

    /// Impls registered for the *exact* `type_name` (e.g. `Box<int>`).
    pub fn impls_for_exact(&self, type_name: &str) -> Vec<&TraitImpl> {
        self.impls.values().filter(|i| i.type_name == type_name).collect()
    }

    /// Impls whose registered type shares `type_name`'s base name
    /// (e.g. `Box` matches `Box<int>`, `Box<string>`).
    pub fn impls_for_base(&self, type_name: &str) -> Vec<&TraitImpl> {
        self.impls.values()
            .filter(|i| i.type_name.split('<').next().unwrap_or(&i.type_name) == type_name)
            .collect()
    }

    /// Get a method from a trait implementation
    pub fn get_trait_method(&self, trait_name: &str, type_name: &str, method_name: &str) -> Option<Value> {
        let key = (trait_name.to_string(), type_name.to_string());

        if let Some(impl_def) = self.impls.get(&key) {
            if let Some(method) = impl_def.methods.get(method_name) {
                return Some(method.clone());
            }
        }

        None
    }

    /// Create a trait definition from AST nodes
    pub fn create_trait_from_ast(&self,
                                name: &str,
                                methods: &[crate::ast::TraitMethod],
                                generic_params: &[String]) -> Trait {
        let mut trait_methods = HashMap::new();

        for method in methods {
            trait_methods.insert(method.name.clone(), TraitMethod {
                name:         method.name.clone(),
                params:       method.params.clone(),
                return_type:  method.return_type.clone(),
                is_async:     method.is_async,
                default_impl: method.body.clone(),
            });
        }

        Trait {
            name: name.to_string(),
            methods: trait_methods,
            generic_params: generic_params.to_vec(),
        }
    }
}
