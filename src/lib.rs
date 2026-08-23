//! lava-arch — composition layer for the lava suite.
//!
//! Tatara-lisp surface:
//!
//! ```lisp
//! (deflava-architecture aws-vpc-network
//!   :inputs (:cidr-block :availability-zones)
//!   :body
//!   (let* ((vpc (aws-vpc :cidr-block cidr-block :enable-dns-support #t))
//!          (igw (aws-internet-gateway :vpc-id (out vpc :id)))
//!          (public-subnets
//!            (map (lambda (az i)
//!                   (aws-subnet :vpc-id (out vpc :id)
//!                               :availability-zone az
//!                               :cidr-block (cidr-subnet cidr-block 8 i)
//!                               :map-public-ip-on-launch #t))
//!                 availability-zones (iota (length availability-zones)))))
//!     (lava-output :vpc-id (out vpc :id))
//!     (lava-output :public-subnet-ids
//!                  (map (lambda (s) (out s :id)) public-subnets))))
//! ```
//!
//! The lisp expression evaluates to a typed [`Builder`] value (this
//! crate) → `lava_core::Architecture` → terraform.json → magma applies.
//!
//! ## Why a builder ↔ Architecture split
//!
//! The DSL form is mutation-heavy (push resources, declare outputs)
//! but the typed substrate ([`lava_core::Architecture`]) is value-
//! oriented + immutable. `Builder` holds the accumulator; `finish()`
//! consumes it + produces the typed value. Same Lisp expression
//! always produces the same Architecture (referential transparency).

#![allow(clippy::module_name_repetitions)]

use indexmap::IndexMap;
use lava_core::{Architecture, ProviderRef, Resource, ResourceRef, Value};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Mutation-friendly accumulator. tatara-vm calls into Builder while
/// evaluating a (deflava-architecture …) body; final `finish()` returns
/// the immutable [`lava_core::Architecture`].
#[derive(Debug, Default)]
pub struct Builder {
    name: String,
    resources: Vec<Resource>,
    outputs: IndexMap<String, Value>,
    providers: Vec<ProviderRef>,
    /// Adopt-not-create declarations. Kept beside resources rather than
    /// derived from them: whether an object already exists is a fact about
    /// the live world, not about the architecture, so only the caller can
    /// know it.
    imports: Vec<lava_core::Import>,
}

impl Builder {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// Add a resource. Returns a [`ResourceRef`] pointing at the
    /// resource's `id` attribute — chains into other resources'
    /// attribute slots via `(out resource :attribute)` in lisp.
    pub fn add_resource(&mut self, r: Resource) -> ResourceRef {
        let rref = r.out("id");
        self.resources.push(r);
        rref
    }

    /// Declare a terraform `import` block — adopt the existing object `id`
    /// into the address `to`.
    ///
    /// Without this an architecture describing a LIVE estate plans to create
    /// what is already there; for a catalogue of ~1000 repositories that is a
    /// thousand `422 name already exists` failures behind a plan that looked
    /// entirely reasonable.
    pub fn add_import(&mut self, to: impl Into<String>, id: impl Into<String>) {
        self.imports.push(lava_core::Import { to: to.into(), id: id.into() });
    }

    /// Declare a typed output. Downstream architectures consume via
    /// `(architecture-output upstream :vpc-id)`.
    pub fn output(&mut self, key: impl Into<String>, value: Value) {
        self.outputs.insert(key.into(), value);
    }

    /// Pin a provider configuration. Used when resources need a
    /// non-default provider (e.g. `aws.us-west-2`).
    pub fn provider(&mut self, p: ProviderRef) {
        self.providers.push(p);
    }

    /// Consume the builder, producing an immutable Architecture.
    #[must_use]
    pub fn finish(self) -> Architecture {
        Architecture {
            name: self.name,
            resources: self.resources,
            data_sources: Vec::new(),
            outputs: self.outputs,
            providers: self.providers,
            locals: indexmap::IndexMap::new(),
            imports: self.imports,
        }
    }
}

/// Stored architecture spec. The deflava-architecture form expands to
/// this typed value (name + input parameters + body source). Stored
/// for later instantiation by [`Library::build`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureSpec {
    pub name: String,
    /// Named inputs the body consumes — analog of pangea
    /// architectures' `config` hash.
    pub inputs: Vec<String>,
    /// Frozen tatara-lisp source for the body. Re-evaluated per
    /// instantiation with input bindings supplied.
    pub body: String,
}

/// Library of registered architectures. The deflava-architecture form
/// registers into a Library; consumers call `Library::build(name, args)`
/// to materialize a typed Architecture instance.
#[derive(Debug, Default)]
pub struct Library {
    architectures: IndexMap<String, ArchitectureSpec>,
}

impl Library {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an architecture spec. Operators call this once per
    /// deflava-architecture form at load time.
    pub fn register(&mut self, spec: ArchitectureSpec) {
        self.architectures.insert(spec.name.clone(), spec);
    }

    /// Look up a registered architecture by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ArchitectureSpec> {
        self.architectures.get(name)
    }

    /// List every registered architecture in insertion order.
    #[must_use]
    pub fn names(&self) -> Vec<&String> {
        self.architectures.keys().collect()
    }
}

#[derive(Debug, Error)]
pub enum ArchError {
    #[error("architecture not found: {0}")]
    NotFound(String),
    #[error("missing required input: {0}")]
    MissingInput(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    /// End-to-end: build a 2-resource architecture via Builder, render
    /// to Terraform JSON, verify the wire shape matches what magma +
    /// every tfplugin provider would consume.
    #[test]
    fn builder_renders_to_terraform_json_via_lava_core() {
        let mut b = Builder::new("vpc-with-igw");
        let mut vpc_attrs = IndexMap::new();
        vpc_attrs.insert("cidr_block".to_string(), Value::s("10.0.0.0/16"));
        let vpc = Resource {
            type_id: "aws_vpc".to_string(),
            name: "main".to_string(),
            attributes: vpc_attrs,
            depends_on: vec![],
            provider: None,
            multiplicity: None,
        };
        let vpc_ref = b.add_resource(vpc);
        let mut igw_attrs = IndexMap::new();
        igw_attrs.insert("vpc_id".to_string(), Value::Ref(vpc_ref.clone()));
        let igw = Resource {
            type_id: "aws_internet_gateway".to_string(),
            name: "igw".to_string(),
            attributes: igw_attrs,
            depends_on: vec![],
            provider: None,
            multiplicity: None,
        };
        b.add_resource(igw);
        b.output("vpc_id".to_string(), Value::Ref(vpc_ref));
        let arch = b.finish();
        let json = arch.render_terraform_json().unwrap();
        assert_eq!(
            json["resource"]["aws_internet_gateway"]["igw"]["vpc_id"],
            "${aws_vpc.main.id}"
        );
        assert_eq!(json["output"]["vpc_id"]["value"], "${aws_vpc.main.id}");
    }

    #[test]
    fn library_registers_architectures_in_insertion_order() {
        let mut lib = Library::new();
        lib.register(ArchitectureSpec {
            name: "vpc-network".to_string(),
            inputs: vec!["cidr_block".to_string()],
            body: "(...)".to_string(),
        });
        lib.register(ArchitectureSpec {
            name: "iam-roles".to_string(),
            inputs: vec![],
            body: "(...)".to_string(),
        });
        assert_eq!(lib.names(), vec![&"vpc-network".to_string(), &"iam-roles".to_string()]);
        assert!(lib.get("vpc-network").is_some());
        assert!(lib.get("missing").is_none());
    }
}
