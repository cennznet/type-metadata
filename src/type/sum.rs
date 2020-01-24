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

use crate::tm_std::*;

use crate::{NamedField, form::{CompactForm, Form, MetaForm}, IntoCompact, Registry, UnnamedField, TypePath};
use derive_more::From;
use serde::Serialize;

/// A Sum type (consisting of variants) e.g an enum
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, From)]
#[serde(bound = "F::Type: Serialize")]
#[serde(rename_all = "lowercase")]
pub enum TypeSum<F: Form = MetaForm> {
	// todo: [AJ] potentially extract struct TypeSumVariants<Variant>
	/// A C-like enum type.
	ClikeEnum(TypeSumClikeEnum<F>),
	/// A Rust enum, aka tagged union.
	Enum(TypeSumEnum<F>),
}

impl IntoCompact for TypeSum {
	type Output = TypeSum<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			TypeSum::ClikeEnum(clike_enum) =>
				TypeSum::ClikeEnum(clike_enum.into_compact(registry)),
			TypeSum::Enum(r#enum) =>
				TypeSum::Enum(r#enum.into_compact(registry)),
		}
	}
}
/// A C-like enum type.
///
/// # Example
///
/// ```
/// enum Days {
///     Monday,
///     Tuesday,
///     Wednesday,
///     Thursday = 42, // Also allows to manually set the discriminant!
///     Friday,
///     Saturday,
///     Sunday,
/// }
/// ```
/// or an empty enum (for marker purposes)TypeStruct
/// ```
/// enum JustAMarker {}
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, From)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeSumClikeEnum<F: Form = MetaForm> {
	/// The path of the C-like enum
	path: TypePath<F>,
	/// The variants of the C-like enum.
	#[serde(rename = "variants")]
	variants: Vec<ClikeEnumVariant<F>>,
}

impl IntoCompact for TypeSumClikeEnum {
	type Output = TypeSumClikeEnum<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeSumClikeEnum {
			path: self.path.into_compact(registry),
			variants: self
				.variants
				.into_iter()
				.map(|variant| variant.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeSumClikeEnum {
	/// Creates a new C-like enum from the given variants.
	pub fn new<V>(path: TypePath, variants: V) -> Self
		where
			V: IntoIterator<Item = ClikeEnumVariant>,
	{
		Self {
			path,
			variants: variants.into_iter().collect(),
		}
	}
}

/// A C-like enum variant.
///
/// # Example
///
/// ```
/// enum Food {
///     Pizza,
/// //  ^^^^^ this is a C-like enum variant
///     Salad = 1337,
/// //  ^^^^^ this as well
///     Apple,
/// //  ^^^^^ and this
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct ClikeEnumVariant<F: Form = MetaForm> {
	/// The name of the variant.
	name: F::String,
	/// The discriminant of the variant.
	///
	/// # Note
	///
	/// Even though setting the discriminant is optional
	/// every C-like enum variant has a discriminant specified
	/// upon compile-time.
	discriminant: u64,
}

impl IntoCompact for ClikeEnumVariant {
	type Output = ClikeEnumVariant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		ClikeEnumVariant {
			name: registry.register_string(self.name),
			discriminant: self.discriminant,
		}
	}
}

impl ClikeEnumVariant {
	/// Creates a new C-like enum variant.
	pub fn new<D>(name: <MetaForm as Form>::String, discriminant: D) -> Self
		where
			D: Into<u64>,
	{
		Self {
			name,
			discriminant: discriminant.into(),
		}
	}
}

/// A Rust enum, aka tagged union.
///
/// # Examples
///
/// ```
/// enum MyEnum {
///     RustAllowsForClikeVariants,
///     AndAlsoForTupleStructs(i32, bool),
///     OrStructs {
///         with: i32,
///         named: bool,
///         fields: [u8; 32],
///     },
///     ItIsntPossibleToSetADiscriminantThough,
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct TypeSumEnum<F: Form = MetaForm> {
	/// The path of the enum
	path: TypePath<F>,
	/// The variants of the enum.
	variants: Vec<EnumVariant<F>>,
}

impl IntoCompact for TypeSumEnum {
	type Output = TypeSumEnum<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		TypeSumEnum {
			path: self.path.into_compact(registry),
			variants: self
				.variants
				.into_iter()
				.map(|variant| variant.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl TypeSumEnum {
	/// Creates a new Rust enum from the given variants.
	pub fn new<V>(path: TypePath, variants: V) -> Self
		where
			V: IntoIterator<Item = EnumVariant>,
	{
		Self {
			path,
			variants: variants.into_iter().collect(),
		}
	}
}

/// A Rust enum variant.
///
/// This can either be a unit struct, just like in C-like enums,
/// a tuple-struct with unnamed fields,
/// or a struct with named fields.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, From)]
#[serde(bound = "F::Type: Serialize")]
#[serde(rename_all = "lowercase")]
pub enum EnumVariant<F: Form = MetaForm> {
	/// A unit struct variant.
	Unit(EnumVariantUnit<F>),
	/// A struct variant with named fields.
	Struct(EnumVariantStruct<F>),
	/// A tuple-struct variant with unnamed fields.
	TupleStruct(EnumVariantTupleStruct<F>),
}

impl IntoCompact for EnumVariant {
	type Output = EnumVariant<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			EnumVariant::Unit(unit) => unit.into_compact(registry).into(),
			EnumVariant::Struct(r#struct) => r#struct.into_compact(registry).into(),
			EnumVariant::TupleStruct(tuple_struct) => tuple_struct.into_compact(registry).into(),
		}
	}
}

/// An unit struct enum variant.
///
/// These are similar to the variants in C-like enums.
///
/// # Example
///
/// ```
/// enum Operation {
///     Zero,
/// //  ^^^^ this is a unit struct enum variant
///     Add(i32, i32),
///     Minus { source: i32 }
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct EnumVariantUnit<F: Form = MetaForm> {
	/// The name of the variant.
	name: F::String,
}

impl IntoCompact for EnumVariantUnit {
	type Output = EnumVariantUnit<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		EnumVariantUnit {
			name: registry.register_string(self.name),
		}
	}
}

impl EnumVariantUnit {
	/// Creates a new unit struct variant.
	pub fn new(name: &'static str) -> Self {
		Self { name }
	}
}

/// A struct enum variant with named fields.
///
/// # Example
///
/// ```
/// enum Operation {
///     Zero,
///     Add(i32, i32),
///     Minus { source: i32 }
/// //  ^^^^^^^^^^^^^^^^^^^^^ this is a struct enum variant
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct EnumVariantStruct<F: Form = MetaForm> {
	/// The name of the struct variant.
	name: F::String,
	/// The fields of the struct variant.
	fields: Vec<NamedField<F>>,
}

impl IntoCompact for EnumVariantStruct {
	type Output = EnumVariantStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		EnumVariantStruct {
			name: registry.register_string(self.name),
			fields: self
				.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl EnumVariantStruct {
	/// Creates a new struct variant from the given fields.
	pub fn new<F>(name: <MetaForm as Form>::String, fields: F) -> Self
		where
			F: IntoIterator<Item = NamedField>,
	{
		Self {
			name,
			fields: fields.into_iter().collect(),
		}
	}
}

/// A tuple struct enum variant.
///
/// # Example
///
/// ```
/// enum Operation {
///     Zero,
///     Add(i32, i32),
/// //  ^^^^^^^^^^^^^ this is a tuple-struct enum variant
///     Minus {
///         source: i32,
///     }
/// }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct EnumVariantTupleStruct<F: Form = MetaForm> {
	/// The name of the variant.
	name: F::String,
	/// The fields of the variant.
	#[serde(rename = "types")]
	fields: Vec<UnnamedField<F>>,
}

impl IntoCompact for EnumVariantTupleStruct {
	type Output = EnumVariantTupleStruct<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		EnumVariantTupleStruct {
			name: registry.register_string(self.name),
			fields: self
				.fields
				.into_iter()
				.map(|field| field.into_compact(registry))
				.collect::<Vec<_>>(),
		}
	}
}

impl EnumVariantTupleStruct {
	/// Creates a new tuple struct enum variant from the given fields.
	pub fn new<F>(name: <MetaForm as Form>::String, fields: F) -> Self
		where
			F: IntoIterator<Item = UnnamedField>,
	{
		Self {
			name,
			fields: fields.into_iter().collect(),
		}
	}
}
