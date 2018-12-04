#![recursion_limit = "512"]

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

extern crate syn;
#[cfg_attr(test, macro_use)]
extern crate quote;
extern crate glob;
extern crate unindent;
use unindent::unindent;
use glob::glob;

use std::fs::{File, metadata};
use std::io::Read;
use std::io::Write;

extern crate ordermap;
use ordermap::OrderMap;

mod generators;
mod parsers;
use parsers::parse;

pub fn scan_and_generate(src_prefix: &str) {
    for maybe_mod_path in glob(&format!("{}/**/mod.rs", src_prefix)).unwrap() {
        if let Ok(mod_path) = maybe_mod_path {
            //println!("cargo:warning={:?}", mod_path);
            let auto_path = mod_path
                .clone()
                .to_str()
                .unwrap()
                .replace("mod.rs", "kay_auto.rs");
            if let Ok(src_meta) = metadata(&mod_path) {
                let regenerate = match metadata(&auto_path) {
                    Ok(auto_meta) => src_meta.modified().unwrap() > auto_meta.modified().unwrap(),
                    _ => true,
                };

                if regenerate {
                    let maybe_auto_file = if let Ok(ref mut file) = File::open(&mod_path) {
                        let mut file_str = String::new();
                        file.read_to_string(&mut file_str).unwrap();
                        match parse(&file_str) {
                            Ok(model) => {
                                if model.actors.is_empty() && model.traits.is_empty() {
                                    None
                                } else {
                                    Some(generate(&model))
                                }
                            }
                            Err(error) => {
                                println!(
                                    "cargo:warning=kay_codegen parse error in {}: {}",
                                    mod_path.to_str().unwrap_or("??"),
                                    error
                                );
                                None
                            },
                        }
                    } else {
                        panic!("couldn't load");
                    };

                    if let Some(auto_file) = maybe_auto_file {
                        if let Ok(ref mut file) = File::create(&auto_path) {
                            file.write_all(auto_file.as_bytes()).unwrap();
                        }
                    }
                }
            } else {
                panic!("couldn't load");
            };
        }
    }
}

type ActorName = syn::Type;
type TraitName = syn::Path;

#[derive(Default)]
pub struct Model {
    pub actors: OrderMap<ActorName, ActorDef>,
    pub traits: OrderMap<TraitName, TraitDef>,
}

#[derive(Default)]
pub struct ActorDef {
    pub handlers: Vec<Handler>,
    pub impls: Vec<TraitName>,
    pub defined_here: bool,
}

#[derive(Default)]
pub struct TraitDef {
    pub handlers: Vec<Handler>,
}

#[derive(Clone)]
pub struct Handler {
    name: syn::Ident,
    arguments: Vec<syn::FnArg>,
    scope: HandlerType,
    critical: bool,
    returns_fate: bool,
    from_trait: Option<TraitName>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum HandlerType {
    Handler,
    Init,
}

pub fn generate(model: &Model) -> String {
    let traits_msgs = model.generate_traits();
    let actors_msgs = model.generate_actor_ids_messages_and_conversions();
    let setup = model.generate_setups();

    use generators::ind;

    unindent(&format!(r#"
        //! This is all auto-generated. Do not touch.
        #![cfg_attr(rustfmt, rustfmt_skip)]
        #[allow(unused_imports)]
        use kay::{{ActorSystem, TypedID, RawID, Fate, Actor, TraitIDFrom, ActorOrActorTrait}};
        #[allow(unused_imports)]
        use super::*;

        {traits_msgs}

        {actors_msgs}

        {setup}"#, traits_msgs=ind(&traits_msgs, 2), actors_msgs=ind(&actors_msgs, 2), setup=ind(&setup, 2)))
}

#[test]
fn simple_actor() {
    let input = quote!(
        pub struct SomeActor {
            id: Option<SomeActorID>,
            field: usize
        }

        impl SomeActor {
            pub fn some_method(&mut self, some_param: usize, world: &mut World) {
                self.id().some_method(42, world);
            }

            pub fn no_params_fate(&mut self, world: &mut World) -> Fate {
                Fate::Die
            }

            pub fn init_ish(id: SomeActorID, some_param: usize, world: &mut World) -> SomeActor {
                SomeActor {
                    id: Some(id),
                    field: some_param
                }
            }
        }
    );
    let expected = unindent(r#"
        //! This is all auto-generated. Do not touch.
        #![cfg_attr(rustfmt, rustfmt_skip)]
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

        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
        pub struct SomeActorID {
            _raw_id: RawID
        }

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
            pub fn some_method(&self, some_param: usize, world: &mut World) {
                world.send(self.as_raw(), MSG_SomeActor_some_method(some_param));
            }
            
            pub fn no_params_fate(&self, world: &mut World) {
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
        }"#);

    let output = generate(&parse(&input.into_string()).unwrap());

    println!("{}", output);

    assert_eq!(expected, output);
}

#[test]
fn trait_and_impl() {
    let input = quote!(
        pub struct SomeActor {
            _id: Option<SomeActorID>,
            field: usize
        }

        trait SomeTrait {
            fn some_method(&mut self, some_param: usize, world: &mut World);
            fn no_params_fate(&mut self, world: &mut World) -> Fate;
            fn some_default_impl_method(&mut self, world: &mut World) {
                self.some_method(3, world);
            }
        }

        impl SomeTrait for SomeActor {
            fn some_method(&mut self, some_param: usize, world: &mut World) {
                self.id().some_method(42, world);
            }

            fn no_params_fate(&mut self, world: &mut World) -> Fate {
                Fate::Die
            }
        }

        impl ForeignTrait for SomeActor {
            fn simple(&mut self, some_param: usize, world: &mut World) {
                self.id().some_method(some_param, world);
            }
        }

        // This shouldn't generate any RawID
        impl Deref for SomeActor {
            type Target = usize;
            fn deref(&self) -> &usize {
                &self.field
            }
        }
    );
    let expected = unindent(r#"
        //! This is all auto-generated. Do not touch.
        #![cfg_attr(rustfmt, rustfmt_skip)]
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
            pub fn some_method(&self, some_param: usize, world: &mut World) {
                world.send(self.as_raw(), MSG_SomeTrait_some_method(some_param));
            }
            
            pub fn no_params_fate(&self, world: &mut World) {
                world.send(self.as_raw(), MSG_SomeTrait_no_params_fate());
            }
            
            pub fn some_default_impl_method(&self, world: &mut World) {
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

        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
        pub struct SomeActorID {
            _raw_id: RawID
        }

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
        }"#);

    let output = generate(&parse(&input.into_string()).unwrap());

    println!("{}", output);

    assert_eq!(expected, output);
}
