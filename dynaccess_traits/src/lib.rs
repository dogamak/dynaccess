pub trait Field<S> {
    type Type;
    fn set(struct_: &mut S, value: Self::Type);
    fn get(struct_: &S) -> &Self::Type;
    fn get_mut(struct_: &mut S) -> &mut Self::Type;
}

pub trait FieldAccessors {
    fn set<F,V>(&mut self, field: F, value: V)
        where F: Field<Self, Type=V>,
              Self: Sized;
    fn get<F,V>(& self, field: F) -> &V
        where F: Field<Self, Type=V>,
              Self: Sized;
    fn get_mut<F,V>(&mut self, field: F) -> &mut V
        where F: Field<Self, Type=V>,
              Self: Sized;
}
