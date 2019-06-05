//! This is all auto-generated. Do not touch.
#![rustfmt::skip]
#[allow(unused_imports)]
use kay::{ActorSystem, TypedID, RawID, Fate, Actor, TraitIDFrom, ActorOrActorTrait};
#[allow(unused_imports)]
use super::*;


pub struct SomeTraitID<A: Compact, B: Compact> {
    _raw_id: RawID, _marker: ::std::marker::PhantomData<Box<(A, B)>>
}

impl<A: Compact, B: Compact> Copy for SomeTraitID<A, B> {}
impl<A: Compact, B: Compact> Clone for SomeTraitID<A, B> { fn clone(&self) -> Self { *self } }
impl<A: Compact, B: Compact> ::std::fmt::Debug for SomeTraitID<A, B> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "SomeTraitID<A, B>({:?})", self._raw_id)
    }
}
impl<A: Compact, B: Compact> ::std::hash::Hash for SomeTraitID<A, B> {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        self._raw_id.hash(state);
    }
}
impl<A: Compact, B: Compact> PartialEq for SomeTraitID<A, B> {
    fn eq(&self, other: &SomeTraitID<A, B>) -> bool {
        self._raw_id == other._raw_id
    }
}
impl<A: Compact, B: Compact> Eq for SomeTraitID<A, B> {}

pub struct SomeTraitRepresentative<A: Compact, B: Compact>{ _marker: ::std::marker::PhantomData<Box<(A, B)>> }

impl<A: Compact, B: Compact> ActorOrActorTrait for SomeTraitRepresentative<A, B> {
    type ID = SomeTraitID<A, B>;
}

impl<A: Compact, B: Compact> TypedID for SomeTraitID<A, B> {
    type Target = SomeTraitRepresentative<A, B>;

    fn from_raw(id: RawID) -> Self {
        SomeTraitID { _raw_id: id, _marker: ::std::marker::PhantomData }
    }

    fn as_raw(&self) -> RawID {
        self._raw_id
    }
}

impl<A: Compact, B: Compact, Act: Actor + SomeTrait<A, B>> TraitIDFrom<Act> for SomeTraitID<A, B> {}

impl<A: Compact, B: Compact> SomeTraitID<A, B> {
    pub fn some_method(self, some_param: A, world: &mut World) {
        world.send(self.as_raw(), MSG_SomeTrait_some_method(some_param));
    }

    pub fn no_params_fate(self, world: &mut World) {
        world.send(self.as_raw(), MSG_SomeTrait_no_params_fate());
    }

    pub fn some_default_impl_method(self, world: &mut World) {
        world.send(self.as_raw(), MSG_SomeTrait_some_default_impl_method());
    }

    pub fn register_trait(system: &mut ActorSystem) {
        system.register_trait::<SomeTraitRepresentative<A, B>>();
        system.register_trait_message::<MSG_SomeTrait_some_method<A>>();
        system.register_trait_message::<MSG_SomeTrait_no_params_fate>();
        system.register_trait_message::<MSG_SomeTrait_some_default_impl_method>();
    }

    pub fn register_implementor<Act: Actor + SomeTrait<A, B>>(system: &mut ActorSystem) {
        system.register_implementor::<Act, SomeTraitRepresentative<A, B>>();
        system.add_handler::<Act, _, _>(
            |&MSG_SomeTrait_some_method::<A>(some_param), instance, world| {
                instance.some_method(some_param, world); Fate::Live
            }, false
        );

        system.add_handler::<Act, _, _>(
            |&MSG_SomeTrait_no_params_fate(), instance, world| {
                instance.no_params_fate(world)
            }, false
        );

        system.add_handler::<Act, _, _>(
            |&MSG_SomeTrait_some_default_impl_method(), instance, world| {
                instance.some_default_impl_method(world); Fate::Live
            }, false
        );
    }
}

#[derive(Compact, Clone)] #[allow(non_camel_case_types)]
struct MSG_SomeTrait_some_method<A: Compact>(pub A);
#[derive(Copy, Clone)] #[allow(non_camel_case_types)]
struct MSG_SomeTrait_no_params_fate();
#[derive(Copy, Clone)] #[allow(non_camel_case_types)]
struct MSG_SomeTrait_some_default_impl_method();

impl Actor for SomeActor {
    type ID = SomeActorID;

    fn id(&self) -> Self::ID {
        self.id
    }
    unsafe fn set_id(&mut self, id: RawID) {
        self.id = Self::ID::from_raw(id);
    }
}


pub struct SomeActorID {
    _raw_id: RawID
}

impl Copy for SomeActorID {}
impl Clone for SomeActorID { fn clone(&self) -> Self { *self } }
impl ::std::fmt::Debug for SomeActorID {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "SomeActorID({:?})", self._raw_id)
    }
}
impl ::std::hash::Hash for SomeActorID {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        self._raw_id.hash(state);
    }
}
impl PartialEq for SomeActorID {
    fn eq(&self, other: &SomeActorID) -> bool {
        self._raw_id == other._raw_id
    }
}
impl Eq for SomeActorID {}

impl TypedID for SomeActorID {
    type Target = SomeActor;

    fn from_raw(id: RawID) -> Self {
        SomeActorID { _raw_id: id }
    }

    fn as_raw(&self) -> RawID {
        self._raw_id
    }
}

impl SomeActorID {

}



impl Into<SomeTraitID<usize, isize>> for SomeActorID {
    fn into(self) -> SomeTraitID<usize, isize> {
        SomeTraitID::from_raw(self.as_raw())
    }
}

impl Into<ForeignTraitID<usize, isize>> for SomeActorID {
    fn into(self) -> ForeignTraitID<usize, isize> {
        ForeignTraitID::from_raw(self.as_raw())
    }
}

#[allow(unused_variables)]
#[allow(unused_mut)]
pub fn auto_setup<A: Compact, B: Compact>(system: &mut ActorSystem) {
    SomeTraitID::<A, B>::register_trait(system);
    SomeTraitID::<usize, isize>::register_implementor::<SomeActor>(system);
    ForeignTraitID::<usize, isize>::register_implementor::<SomeActor>(system);
}