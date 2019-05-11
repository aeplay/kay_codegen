//! This is all auto-generated. Do not touch.
#![rustfmt::skip]
#[allow(unused_imports)]
use kay::{ActorSystem, TypedID, RawID, Fate, Actor, TraitIDFrom, ActorOrActorTrait};
#[allow(unused_imports)]
use super::*;



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
    pub fn some_method(self, some_param: usize, world: &mut World) {
        world.send(self.as_raw(), MSG_SomeActor_some_method(some_param));
    }

    pub fn no_params_fate(self, world: &mut World) {
        world.send(self.as_raw(), MSG_SomeActor_no_params_fate());
    }

    pub fn init_ish(some_param: usize, world: &mut World) -> Self {
        let id = SomeActorID::from_raw(world.allocate_instance_id::<SomeActor>());
        let swarm = world.local_broadcast::<SomeActor>();
        world.send(swarm, MSG_SomeActor_init_ish(id, some_param));
        id
    }
}

#[derive(Compact, Clone)] #[allow(non_camel_case_types)]
struct MSG_SomeActor_some_method(pub usize);
#[derive(Copy, Clone)] #[allow(non_camel_case_types)]
struct MSG_SomeActor_no_params_fate();
#[derive(Compact, Clone)] #[allow(non_camel_case_types)]
struct MSG_SomeActor_init_ish(pub SomeActorID, pub usize);


#[allow(unused_variables)]
#[allow(unused_mut)]
pub fn auto_setup(system: &mut ActorSystem) {


    system.add_handler::<SomeActor, _, _>(
        |&MSG_SomeActor_some_method(some_param), instance, world| {
            instance.some_method(some_param, world); Fate::Live
        }, false
    );

    system.add_handler::<SomeActor, _, _>(
        |&MSG_SomeActor_no_params_fate(), instance, world| {
            instance.no_params_fate(world)
        }, false
    );

    system.add_spawner::<SomeActor, _, _>(
        |&MSG_SomeActor_init_ish(id, some_param), world| {
            SomeActor::init_ish(id, some_param, world)
        }, false
    );
}