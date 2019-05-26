#![recursion_limit = "512"]

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

extern crate syn;
extern crate glob;
extern crate unindent;
use unindent::unindent;
use glob::glob;

use std::fs::{File, metadata};
use std::io::Read;
use std::io::Write;

extern crate indexmap;
use indexmap::IndexMap;

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

#[derive(Default, Debug)]
pub struct Model {
    pub actors: IndexMap<ActorName, ActorDef>,
    pub traits: IndexMap<TraitName, TraitDef>,
}

#[derive(Default, Debug)]
pub struct ActorDef {
    pub handlers: Vec<Handler>,
    pub impls: Vec<TraitName>,
    pub defined_here: bool,
    pub generics: syn::Generics
}

#[derive(Default, Debug)]
pub struct TraitDef {
    pub handlers: Vec<Handler>,
    pub generics: syn::Generics
}

#[derive(Clone, Debug)]
pub struct Handler {
    name: syn::Ident,
    arguments: Vec<syn::FnArg>,
    scope: HandlerType,
    critical: bool,
    returns_fate: bool,
    from_trait: Option<TraitName>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
        #![rustfmt::skip]
        #[allow(unused_imports)]
        use kay::{{ActorSystem, TypedID, RawID, Fate, Actor, TraitIDFrom, ActorOrActorTrait}};
        #[allow(unused_imports)]
        use super::*;

        {traits_msgs}

        {actors_msgs}

        {setup}"#, traits_msgs=ind(&traits_msgs, 2), actors_msgs=ind(&actors_msgs, 2), setup=ind(&setup, 2)))
}

#[cfg(test)]
fn normalize_empty_lines(string: &str) -> String {
    string.replace("        \n", "\n").replace("    \n", "\n")
}

#[test]
fn simple_actor() {
    let input = include_str!("./testcases/simple_actor_in.rs");
    let expected = include_str!("./testcases/simple_actor_out.rs");

    let output = generate(&parse(input).unwrap());

    assert_eq!(normalize_empty_lines(&expected), normalize_empty_lines(&output));
}

#[test]
fn trait_and_impl() {
    let input = include_str!("./testcases/trait_and_impl_in.rs");
    let expected = include_str!("./testcases/trait_and_impl_out.rs");

    let output = generate(&parse(input).unwrap());

    assert_eq!(normalize_empty_lines(&expected), normalize_empty_lines(&output));
}

#[test]
fn generic_actor() {
    let input = include_str!("./testcases/generic_actor_in.rs");
    let expected = include_str!("./testcases/generic_actor_out.rs");

    let output = generate(&parse(input).unwrap());

    assert_eq!(normalize_empty_lines(&expected), normalize_empty_lines(&output));
}

#[test]
fn generic_trait_and_impl() {
    let input = include_str!("./testcases/generic_trait_and_impl_in.rs");
    let expected = include_str!("./testcases/generic_trait_and_impl_out.rs");

    let output = generate(&parse(input).unwrap());

    assert_eq!(normalize_empty_lines(&expected), normalize_empty_lines(&output));
}
