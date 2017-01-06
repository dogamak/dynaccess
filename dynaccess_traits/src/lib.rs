//! This crate contains traits used by `#[derive(Dynaccess)]`.

/// Trait that is implemented for all unit structs representing
/// struct fields.
pub trait Field<S> {
    /// The type of the field.
    type Type;

    /// Visits a struct and sets the value of the field corresponding to
    /// the implementor of this trait.
    fn set(struct_: &mut S, value: Self::Type);
    
    /// Visits a struct and returns a reference to the value of the field
    /// corresponding to the implementor of this trait.
    fn get(struct_: &S) -> &Self::Type;

    /// Visits a struct and returns a mutable reference to the value of
    /// the field corresponding to the implementor of this trait.
    fn get_mut(struct_: &mut S) -> &mut Self::Type;
}

/// Trait that is implemented for the struct `#[derive(Dynaccess)]` is used on.
pub trait FieldAccessors {
    /// Calls `F::set(self, value)` to set the value of the field the passed
    /// unit struct represents.
    fn set<F,V>(&mut self, field: F, value: V)
        where F: Field<Self, Type=V>,
              Self: Sized;

    /// Calls `F::get(self, value)` to get a reference to the value of the field
    /// the passed unit struct represents.
    fn get<F,V>(& self, field: F) -> &V
        where F: Field<Self, Type=V>,
              Self: Sized;
    
    /// Calls `F::get_mut(self, value)` to get a mutable reference to the value
    /// of the field the passed unit struct represents.
    fn get_mut<F,V>(&mut self, field: F) -> &mut V
        where F: Field<Self, Type=V>,
              Self: Sized;
}
