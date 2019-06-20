// Copyright 2019
//     by  Centrality Investments Ltd.
//     and Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::form::{Form, FreeForm};
use derive_more::From;
use serde::Serialize;

/// Implementors return their meta type identifiers.
pub trait HasTypeId {
	/// Returns the static type identifier for `Self`.
	fn type_id() -> TypeId;
}

/// Represents the namespace of a type definition.
///
/// This consists of several segments that each have to be a valid Rust identifier.
/// The first segment represents the crate name in which the type has been defined.
///
/// Rust prelude type may have an empty namespace definition.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
pub struct Namespace<F: Form = FreeForm> {
	/// The segments of the namespace.
	segments: Vec<F::String>,
}

/// An error that may be encountered upon constructing namespaces.
#[derive(PartialEq, Eq, Debug)]
pub enum NamespaceError {
	/// If the module path does not at least have one segment.
	MissingSegments,
	/// If a segment within a module path is not a proper Rust identifier.
	InvalidIdentifier {
		/// The index of the errorneous segment.
		segment: usize,
	},
}

impl Namespace {
	/// Creates a new namespace from the given segments.
	pub fn new<S>(segments: S) -> Result<Self, NamespaceError>
	where
		S: IntoIterator<Item = <FreeForm as Form>::String>,
	{
		let segments = segments.into_iter().collect::<Vec<_>>();
		if segments.len() == 0 {
			return Err(NamespaceError::MissingSegments);
		}
		if let Some(err_at) = segments.iter().position(|seg| {
			/// Returns `true` if the given string is a proper Rust identifier.
			fn is_rust_identifier(s: &str) -> bool {
				// Only ascii encoding is allowed.
				// Note: Maybe this check is superseeded by the `head` and `tail` check.
				println!("is_rust_identifier? \"{}\"", s);
				if !s.is_ascii() {
					return false;
				}
				if let Some((&head, tail)) = s.as_bytes().split_first() {
					// Check if head and tail make up a proper Rust identifier.
					let head_ok = head == b'_' || head >= b'a' && head <= b'z' || head >= b'A' && head <= b'Z';
					let tail_ok = tail.iter().all(|&ch| {
						ch == b'_' || ch >= b'a' && ch <= b'z' || ch >= b'A' && ch <= b'Z' || ch >= b'0' && ch <= b'9'
					});
					head_ok && tail_ok
				} else {
					// String is empty and thus not a valid Rust identifier.
					false
				}
			}
			!is_rust_identifier(seg)
		}) {
			return Err(NamespaceError::InvalidIdentifier { segment: err_at });
		}
		Ok(Self { segments })
	}

	/// Creates a new namespace from the given module path.
	///
	/// # Note
	///
	/// Module path is generally obtained from the `module_path!` Rust macro.
	pub fn from_str(module_path: <FreeForm as Form>::String) -> Result<Self, NamespaceError> {
		Self::new(module_path.split("::"))
	}

	/// Creates the prelude namespace.
	pub fn prelude() -> Self {
		Self { segments: vec![] }
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Serialize, Debug)]
pub enum TypeId<F: Form = FreeForm> {
	Custom(TypeIdCustom<F>),
	Slice(TypeIdSlice<F>),
	Array(TypeIdArray<F>),
	Tuple(TypeIdTuple<F>),
	Primitive(TypeIdPrimitive),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TypeIdPrimitive {
	Bool,
	Str,
	U8,
	U16,
	U32,
	U64,
	U128,
	I8,
	I16,
	I32,
	I64,
	I128,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
pub struct TypeIdCustom<F: Form = FreeForm> {
	name: F::String,
	namespace: Namespace<F>,
	#[serde(rename = "type")]
	type_params: Vec<F::TypeId>,
}

impl TypeIdCustom {
	pub fn new<T>(name: &'static str, namespace: Namespace, type_params: T) -> Self
	where
		T: IntoIterator<Item = TypeId>,
	{
		Self {
			name,
			namespace,
			type_params: type_params.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
pub struct TypeIdArray<F: Form = FreeForm> {
	pub len: u16,
	#[serde(rename = "type")]
	pub type_param: Box<TypeId<F>>,
}

impl TypeIdArray {
	pub fn new<T>(len: u16, type_param: T) -> Self
	where
		T: Into<TypeId>,
	{
		Self {
			len,
			type_param: Box::new(type_param.into()),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
pub struct TypeIdTuple<F: Form = FreeForm> {
	#[serde(rename = "type")]
	pub type_params: Vec<TypeId<F>>,
}

impl TypeIdTuple {
	pub fn new<T>(type_params: T) -> Self
	where
		T: IntoIterator<Item = TypeId>,
	{
		Self {
			type_params: type_params.into_iter().collect(),
		}
	}

	pub fn unit() -> Self {
		Self::new(vec![])
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
pub struct TypeIdSlice<F: Form = FreeForm> {
	#[serde(rename = "type")]
	type_param: Box<TypeId<F>>,
}

impl TypeIdSlice {
	pub fn new<T>(type_param: T) -> Self
	where
		T: Into<TypeId>,
	{
		Self {
			type_param: Box::new(type_param.into()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn namespace_ok() {
		assert_eq!(
			Namespace::new(vec!["hello"]),
			Ok(Namespace {
				segments: vec!["hello"]
			})
		);
		assert_eq!(
			Namespace::new(vec!["Hello", "World"]),
			Ok(Namespace {
				segments: vec!["Hello", "World"]
			})
		);
		assert_eq!(Namespace::new(vec!["_"]), Ok(Namespace { segments: vec!["_"] }));
	}

	#[test]
	fn namespace_err() {
		assert_eq!(Namespace::new(vec![]), Err(NamespaceError::MissingSegments));
		assert_eq!(
			Namespace::new(vec![""]),
			Err(NamespaceError::InvalidIdentifier { segment: 0 })
		);
		assert_eq!(
			Namespace::new(vec!["1"]),
			Err(NamespaceError::InvalidIdentifier { segment: 0 })
		);
		assert_eq!(
			Namespace::new(vec!["Hello", ", World!"]),
			Err(NamespaceError::InvalidIdentifier { segment: 1 })
		);
	}

	#[test]
	fn namespace_from_str() {
		assert_eq!(
			Namespace::from_str("hello::world"),
			Ok(Namespace {
				segments: vec!["hello", "world"]
			})
		);
		assert_eq!(
			Namespace::from_str("::world"),
			Err(NamespaceError::InvalidIdentifier { segment: 0 })
		);
	}
}