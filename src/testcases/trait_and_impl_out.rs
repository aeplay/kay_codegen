//! This is all auto-generated. Do not touch.
#![rustfmt::skip]
#[allow(unused_imports)]
use kay::{ActorSystem, TypedID, RawID, Fate, Actor, TraitIDFrom, ActorOrActorTrait};
#[allow(unused_imports)]
use super::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SomeTraitID {
    _raw_id: RawID
}

pub struct SomeTraitRepresentative;

impl ActorOrActorTrait for SomeTraitRepresentative {
    type ID = SomeTraitID;
}

impl TypedID for SomeTraitID {
    type Target = SomeTraitRepresentative;

    fn from_raw(id: RawID) -> Self {
        SomeTraitID { _raw_id: id }
    }

    fn as_raw(&self) -> RawID {
        self._raw_id
    }
}

impl<A: Actor + SomeTrait> TraitIDFrom<A> for SomeTraitID {}

impl SomeTraitID {
    pub fn some_method(self, some_param: usize, world: &mut World) {
        world.send(self.as_raw(), MSG_SomeTrait_some_method(some_param));
    }

    pub fn no_params_fate(self, world: &mut World) {
        world.send(self.as_raw(), MSG_SomeTrait_no_params_fate());
    }

    pub fn some_default_impl_method(self, world: &mut World) {
        world.send(self.as_raw(), MSG_SomeTrait_some_default_impl_method());
    }

    pub fn register_trait(system: &mut ActorSystem) {
        system.register_trait::<SomeTraitRepresentative>();
        system.register_trait_message::<MSG_SomeTrait_some_method>();
        system.register_trait_message::<MSG_SomeTrait_no_params_fate>();
        system.register_trait_message::<MSG_SomeTrait_some_default_impl_method>();
    }

    pub fn register_implementor<A: Actor + SomeTrait>(system: &mut ActorSystem) {
        system.register_implementor::<A, SomeTraitRepresentative>();
        system.add_handler::<A, _, _>(
            |&MSG_SomeTrait_some_method(some_param), instance, world| {
                instance.some_method(some_param, world); Fate::Live
            }, false
        );

        system.add_handler::<A, _, _>(
            |&MSG_SomeTrait_no_params_fate(), instance, world| {
                instance.no_params_fate(world)
            }, false
        );

        system.add_handler::<A, _, _>(
            |&MSG_SomeTrait_some_default_impl_method(), instance, world| {
                instance.some_default_impl_method(world); Fate::Live
            }, false
        );
    }
}

#[derive(Compact, Clone)] #[allow(non_camel_case_types)]
struct MSG_SomeTrait_some_method(pub usize);
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



impl Into<SomeTraitID> for SomeActorID {
    fn into(self) -> SomeTraitID {
        SomeTraitID::from_raw(self.as_raw())
    }
}

impl Into<ForeignTraitID> for SomeActorID {
    fn into(self) -> ForeignTraitID {
        ForeignTraitID::from_raw(self.as_raw())
    }
}

#[allow(unused_variables)]
#[allow(unused_mut)]
pub fn auto_setup(system: &mut ActorSystem) {
    SomeTraitID::register_trait(system);
    SomeTraitID::register_implementor::<SomeActor>(system);
    ForeignTraitID::register_implementor::<SomeActor>(system);
}