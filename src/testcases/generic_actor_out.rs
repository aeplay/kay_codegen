//! This is all auto-generated. Do not touch.
#![rustfmt::skip]
#[allow(unused_imports)]
use kay::{ActorSystem, TypedID, RawID, Fate, Actor, TraitIDFrom, ActorOrActorTrait};
#[allow(unused_imports)]
use super::*;



impl<A: Compact, B: Compact> Actor for SomeActor<A, B> {
    type ID = SomeActorID<A, B>;

    fn id(&self) -> Self::ID {
        self.id
    }
    unsafe fn set_id(&mut self, id: RawID) {
        self.id = Self::ID::from_raw(id);
    }
}


pub struct SomeActorID<A: Compact, B: Compact> {
    _raw_id: RawID, _marker: ::std::marker::PhantomData<Box<(A, B)>>
}

impl<A: Compact, B: Compact> Copy for SomeActorID<A, B> {}
impl<A: Compact, B: Compact> Clone for SomeActorID<A, B> { fn clone(&self) -> Self { *self } }
impl<A: Compact, B: Compact> ::std::fmt::Debug for SomeActorID<A, B> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "SomeActorID<A, B>({:?})", self._raw_id)
    }
}
impl<A: Compact, B: Compact> ::std::hash::Hash for SomeActorID<A, B> {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        self._raw_id.hash(state);
    }
}
impl<A: Compact, B: Compact> PartialEq for SomeActorID<A, B> {
    fn eq(&self, other: &SomeActorID<A, B>) -> bool {
        self._raw_id == other._raw_id
    }
}
impl<A: Compact, B: Compact> Eq for SomeActorID<A, B> {}

impl<A: Compact, B: Compact> TypedID for SomeActorID<A, B> {
    type Target = SomeActor<A, B>;

    fn from_raw(id: RawID) -> Self {
        SomeActorID { _raw_id: id, _marker: ::std::marker::PhantomData }
    }

    fn as_raw(&self) -> RawID {
        self._raw_id
    }
}

impl<A: Compact, B: Compact> SomeActorID<A, B> {
    pub fn some_method(self, some_param: usize, thing: B, world: &mut World) {
        world.send(self.as_raw(), MSG_SomeActor_some_method::<B>(some_param, thing));
    }

    pub fn no_params_fate(self, world: &mut World) {
        world.send(self.as_raw(), MSG_SomeActor_no_params_fate());
    }

    pub fn init_ish(some_param: usize, world: &mut World) -> Self {
        let id = SomeActorID::<A, B>::from_raw(world.allocate_instance_id::<SomeActor<A, B>>());
        let swarm = world.local_broadcast::<SomeActor<A, B>>();
        world.send(swarm, MSG_SomeActor_init_ish::<A, B>(id, some_param));
        id
    }
}

#[derive(Compact, Clone)] #[allow(non_camel_case_types)]
struct MSG_SomeActor_some_method<B: Compact>(pub usize, pub B);
#[derive(Copy, Clone)] #[allow(non_camel_case_types)]
struct MSG_SomeActor_no_params_fate();
#[derive(Compact, Clone)] #[allow(non_camel_case_types)]
struct MSG_SomeActor_init_ish<A: Compact, B: Compact>(pub SomeActorID<A, B>, pub usize);


#[allow(unused_variables)]
#[allow(unused_mut)]
pub fn auto_setup<A: Compact, B: Compact>(system: &mut ActorSystem) {


    system.add_handler::<SomeActor<A, B>, _, _>(
        |&MSG_SomeActor_some_method::<B>(some_param, ref thing), instance, world| {
            instance.some_method(some_param, thing, world); Fate::Live
        }, false
    );

    system.add_handler::<SomeActor<A, B>, _, _>(
        |&MSG_SomeActor_no_params_fate(), instance, world| {
            instance.no_params_fate(world)
        }, false
    );

    system.add_spawner::<SomeActor<A, B>, _, _>(
        |&MSG_SomeActor_init_ish::<A, B>(id, some_param), world| {
            SomeActor::<A, B>::init_ish(id, some_param, world)
        }, false
    );
}