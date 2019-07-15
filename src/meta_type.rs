use crate::{
	TypeId,
	HasTypeId,
	TypeDef,
	HasTypeDef,
	Metadata,
	form::MetaForm,
};
use core::{
	any::TypeId as AnyTypeId,
	cmp::Ordering,
	hash::{Hash, Hasher},
	fmt::{
		Debug,
		Formatter,
		Error as FmtError,
	},
};

/// A metatype abstraction.
/// 
/// Allows to store compile-time type information at runtime.
/// This again allows to derive type ID and type definition from it.
///
/// This needs a conversion to another representation of types
/// in order to be serializable.
#[derive(Clone, Copy)]
pub struct MetaType {
	/// Function pointer to type ID.
	fn_type_id: fn() -> TypeId<MetaForm>,
	/// Function pointer to type definition.
	fn_type_def: fn() -> TypeDef<MetaForm>,
	// The standard type ID (ab)used in order to provide
	// cheap implementations of the standard traits
	// such as `PartialEq`, `PartialOrd`, `Debug` and `Hash`.
	any_id: AnyTypeId,
}

impl PartialEq for MetaType {
	fn eq(&self, other: &Self) -> bool {
		self.any_id == other.any_id
	}
}

impl Eq for MetaType {}

impl PartialOrd for MetaType {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.any_id.partial_cmp(&other.any_id)
	}
}

impl Ord for MetaType {
	fn cmp(&self, other: &Self) -> Ordering {
		self.any_id.cmp(&other.any_id)
	}
}

impl Hash for MetaType {
	fn hash<H>(&self, state: &mut H)
	where
		H: Hasher,
	{
		self.any_id.hash(state)
	}
}

impl Debug for MetaType {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		self.any_id.fmt(f)
	}
}

impl MetaType {
	pub fn new<T>() -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self {
			fn_type_id: <T as HasTypeId>::type_id,
			fn_type_def: <T as HasTypeDef>::type_def,
			any_id: AnyTypeId::of::<T>(),
		}
	}

	pub fn of<T>(_elem: &T) -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::new::<T>()
	}

	pub fn type_id(&self) -> TypeId<MetaForm> {
		(self.fn_type_id)()
	}

	pub fn type_def(&self) -> TypeDef<MetaForm> {
		(self.fn_type_def)()
	}

	pub fn any_id(&self) -> AnyTypeId {
		self.any_id
	}
}