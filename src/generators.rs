use {Model, HandlerType, Handler};
use syn::*;
use syn::export::ToTokens;
use unindent::unindent;

// indents all but the first line
pub fn ind(block: &str, levels: usize) -> String {
    let mut indent = "\n".to_owned();
    for _ in 0..levels {
        indent += "    ";
    }
    block.lines().collect::<Vec<_>>().join(&indent)
}

fn pth_t(typ: &Type) -> String {
    if let Type::Path(TypePath{ref path, ..}) = typ {
        pth(path)
    } else {
        unimplemented!()
    }
}

fn pth(path: &Path) -> String {
    // TODO: handle initial colons and path arguments
    path.segments.iter().map(|segment| segment.ident.to_string()).collect::<Vec<_>>().join("::")
}

fn generics_from_path(path: &Path) -> String {
    path.segments.last().unwrap().value().arguments.clone().into_token_stream().to_string().replace("< ", "<").replace(" >", ">").replace(" ,", ",")
}

fn to_short_generics(generics: &syn::Generics) -> String {
    generics.split_for_impl().1.into_token_stream().to_string().replace("< ", "<").replace(" >", ">").replace(" :", ":").replace(" ,", ",")
}

fn to_full_generics(generics: &syn::Generics) -> String {
    generics.split_for_impl().0.into_token_stream().to_string().replace("< ", "<").replace(" >", ">").replace(" :", ":").replace(" ,", ",")
}

impl Handler {
    fn used_generics(&self, short_generics: String, full_generics: String, return_short: bool) -> String {
        match self.scope {
            HandlerType::Handler => {
                let used_generics_inner = full_generics.replace("<", "").replace(">", "").split(", ")
                    .zip(short_generics.replace("<", "").replace(">", "").split(", "))
                    .filter_map(|(long, short)|
                        if self.arguments.iter().any(|arg|
                            match arg {
                                ::syn::FnArg::Captured(::syn::ArgCaptured{ty, ..}) => {
                                    ty.into_token_stream().to_string().contains(short)
                                },
                                _ => false
                            }
                        ) {
                            Some(if return_short {short} else {long})
                        } else {
                            None
                        }
                    ).collect::<Vec<_>>().join(", ");

                if used_generics_inner.is_empty() {
                    used_generics_inner
                } else {
                    format!("<{}>", used_generics_inner)
                }
            },
            HandlerType::Init => if return_short {short_generics.clone()} else {full_generics.clone()}
        }
    }
}

impl Model {
    pub fn generate_setups(&self) -> String {
        let trait_registrations = self.traits.iter().map(|(trait_name, trait_def)| {
            let short_generics = to_short_generics(&trait_def.generics);
            let short_generics_turbofish = if short_generics.is_empty() {short_generics} else {format!("::{}", short_generics)};
            format!("{trait_name}ID{short_generics_turbofish}::register_trait(system);", trait_name=pth(trait_name), short_generics_turbofish=short_generics_turbofish)

        }).collect::<Vec<_>>().join("\n");

        let actor_setups = self.actors.iter().map(|(actor_name, actor_def)| {
            let impl_registrations = actor_def.impls.iter().map(|trait_name| {
                let impl_generics = generics_from_path(trait_name);
                let impl_generics_turbofish = if impl_generics.is_empty() {impl_generics} else {format!("::{}", impl_generics)};
                format!("{trait_name}ID{impl_generics_turbofish}::register_implementor::<{actor_name}>(system);", impl_generics_turbofish=impl_generics_turbofish, trait_name=pth(trait_name), actor_name=pth_t(actor_name))
            }).collect::<Vec<_>>().join("\n");

            let handler_registrations = actor_def.handlers.iter().filter_map(|handler|
                if handler.from_trait.is_none() {
                    let msg_name = format!("{}_{}", typ_to_message_prefix(actor_name), handler.name);
                    let is_critical = if handler.critical {"true"} else {"false"};
                    let msg_args = handler.arguments.iter().filter_map(arg_ref_to_bind_as_ref_without_world).collect::<Vec<_>>().join(", ");
                    let handler_params = handler.arguments.iter().map(arg_as_value).collect::<Vec<_>>().join(", ");
                    let short_generics = to_short_generics(&actor_def.generics);
                    let full_generics = to_full_generics(&actor_def.generics);
                    let used_generics = handler.used_generics(short_generics.clone(), full_generics.clone(), true);
                    let used_generics_turbofish = if used_generics.is_empty() {used_generics} else {format!("::{}", used_generics)};
                    let short_generics_turbofish = if short_generics.is_empty() {short_generics.clone()} else {format!("::{}", short_generics)};

                    Some(match handler.scope {
                        HandlerType::Handler => {
                            let maybe_return = if handler.returns_fate {""} else {"; Fate::Live"};

                            unindent(&format!(r#"
                                system.add_handler::<{actor_name}{short_generics}, _, _>(
                                    |&{msg_name}{used_generics_turbofish}({msg_args}), instance, world| {{
                                        instance.{handler_name}({handler_params}){maybe_return}
                                    }}, {is_critical}
                                );"#, actor_name=pth_t(actor_name), short_generics=short_generics, msg_name=msg_name, used_generics_turbofish=used_generics_turbofish, msg_args=msg_args, handler_name=handler.name, handler_params=handler_params, maybe_return=maybe_return, is_critical=is_critical))
                        },
                        HandlerType::Init => {
                            unindent(&format!(r#"
                                system.add_spawner::<{actor_name}{short_generics}, _, _>(
                                    |&{msg_name}{used_generics_turbofish}(id, {msg_args}), world| {{
                                        {actor_name}{short_generics_turbofish}::{handler_name}(id, {handler_params})
                                    }}, {is_critical}
                                );"#, actor_name=pth_t(actor_name), short_generics=short_generics, short_generics_turbofish=short_generics_turbofish, msg_name=msg_name, used_generics_turbofish=used_generics_turbofish, msg_args=msg_args, handler_name=handler.name, handler_params=handler_params, is_critical=is_critical))
                        }
                    })
                } else {
                    None
                }
            ).collect::<Vec<_>>().join("\n\n");

            unindent(&format!(r#"
                {impl_registrations}
                {handler_registrations}"#, impl_registrations=ind(&impl_registrations, 4), handler_registrations=ind(&handler_registrations, 4)))
        }).collect::<Vec<_>>().join("\n");

        let all_generics_inner = self.actors.iter().filter_map(|(_, actor_def)| {
            let full_generics = to_full_generics(&actor_def.generics);
            if full_generics.is_empty() {
                None
            } else {
                Some(full_generics.replace("<", "").replace(">", ""))
            }
        }).chain(self.traits.iter().filter_map(|(_, trait_def)| {
            let full_generics = to_full_generics(&trait_def.generics);
            if full_generics.is_empty() {
                None
            } else {
                Some(full_generics.replace("<", "").replace(">", ""))
            }
        })).collect::<Vec<_>>().join(", ");

        let all_generics = if all_generics_inner.is_empty() {all_generics_inner} else {format!("<{}>", all_generics_inner)};

        unindent(&format!(r#"
            #[allow(unused_variables)]
            #[allow(unused_mut)]
            pub fn auto_setup{all_generics}(system: &mut ActorSystem) {{
                {trait_registrations}
                {actor_setups}
            }}"#, trait_registrations=ind(&trait_registrations, 4), actor_setups=ind(&actor_setups, 4), all_generics=all_generics))
    }

    pub fn generate_traits(&self) -> String {
        #[cfg(feature = "serde-serialization")]
        let trait_id_derives = unindent(r#"
            #[derive(Serialize, Deserialize)] #[serde(transparent)]"#);

        #[cfg(not(feature = "serde-serialization"))]
        let trait_id_derives = "".to_owned();

        self.traits.iter().map(|(trait_name, trait_def)| {
            let full_generics = to_full_generics(&trait_def.generics);
            let full_generics_inner_comma = if full_generics.is_empty() {"".to_owned()} else {format!("{}, ", full_generics.replace("<", "").replace(">", ""))};
            let short_generics = to_short_generics(&trait_def.generics);

            let handler_send_impls = trait_def.handlers.iter().map(|handler| {
                let handler_args = handler.arguments.iter().map(arg_as_ident_val_and_type).collect::<Vec<_>>().join(", ");
                let msg_prefix = trait_to_message_prefix(trait_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);
                let msg_params = handler.arguments.iter().filter_map(arg_as_value_without_world).collect::<Vec<_>>().join(", ");

                unindent(&format!(r#"
                    pub fn {handler_name}(self, {handler_args}) {{
                        world.send(self.as_raw(), {msg_name}({msg_params}));
                    }}"#, handler_name=handler.name, handler_args=handler_args, msg_name=msg_name, msg_params=msg_params))
            }).collect::<Vec<_>>().join("\n\n");

            let trait_msg_registrations = trait_def.handlers.iter().map(|handler| {
                let msg_prefix = trait_to_message_prefix(trait_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);
                let used_generics = handler.used_generics(short_generics.clone(), full_generics.clone(), true);
                format!("system.register_trait_message::<{}{}>();", msg_name, used_generics)
            }).collect::<Vec<_>>().join("\n");

            let implementor_handler_registrations = trait_def.handlers.iter().map(|handler| {
                let msg_prefix = trait_to_message_prefix(trait_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);

                let msg_args = handler.arguments.iter().filter_map(arg_ref_to_bind_as_ref_without_world).collect::<Vec<_>>().join(", ");
                let handler_params = handler.arguments.iter().map(arg_as_value).collect::<Vec<_>>().join(", ");

                let maybe_return = if handler.returns_fate {""} else {"; Fate::Live"};
                let is_critical = if handler.critical {"true"} else {"false"};

                let used_generics = handler.used_generics(short_generics.clone(), full_generics.clone(), true);
                let used_generics_turbofish = if used_generics.is_empty() {used_generics} else {format!("::{}", used_generics)};

                unindent(&format!(r#"
                    system.add_handler::<Act, _, _>(
                        |&{msg_name}{used_generics_turbofish}({msg_args}), instance, world| {{
                            instance.{handler_name}({handler_params}){maybe_return}
                        }}, {is_critical}
                    );"#, msg_name=msg_name, msg_args=msg_args, used_generics_turbofish=used_generics_turbofish, handler_name=handler.name, handler_params=handler_params, maybe_return=maybe_return, is_critical=is_critical))
            }).collect::<Vec<_>>().join("\n\n");

            let msg_struct_defs = trait_def.handlers.iter().map(|handler| {
                let msg_prefix = trait_to_message_prefix(trait_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);

                let msg_param_types = handler.arguments.iter().filter_map(arg_as_pub_type_without_world).collect::<Vec<_>>().join(", ");
                let used_generics = handler.used_generics(short_generics.clone(), full_generics.clone(), false);

                let msg_derives = if msg_param_types.is_empty() && used_generics.is_empty() {
                    "#[derive(Copy, Clone)]"
                } else {
                    "#[derive(Compact, Clone)]"
                };

                unindent(&format!(r#"
                    {msg_derives} #[allow(non_camel_case_types)]
                    struct {msg_name}{used_generics}({msg_param_types});"#, msg_derives=msg_derives, msg_name=msg_name, used_generics=used_generics, msg_param_types=msg_param_types))
            }).collect::<Vec<_>>().join("\n");

            let (maybe_marker, maybe_marker_only, maybe_marker_init) = if full_generics.is_empty() {
                ("".to_owned(), "".to_owned(), "")
            } else {
                (
                    format!(", _marker: ::std::marker::PhantomData<Box<({})>>", short_generics.replace("<", "").replace(">", "")),
                    format!("{{ _marker: ::std::marker::PhantomData<Box<({})>> }}", short_generics.replace("<", "").replace(">", "")),
                    ", _marker: ::std::marker::PhantomData"
                )
            };

            unindent(&format!(r#"
                {trait_id_derives}
                pub struct {trait_name}ID{full_generics} {{
                    _raw_id: RawID{maybe_marker}
                }}

                impl{full_generics} Copy for {trait_name}ID{short_generics} {{}}
                impl{full_generics} Clone for {trait_name}ID{short_generics} {{ fn clone(&self) -> Self {{ *self }} }}
                impl{full_generics} ::std::fmt::Debug for {trait_name}ID{short_generics} {{
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{
                        write!(f, "{trait_name}ID{short_generics}({{:?}})", self._raw_id)
                    }}
                }}
                impl{full_generics} ::std::hash::Hash for {trait_name}ID{short_generics} {{
                    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {{
                        self._raw_id.hash(state);
                    }}
                }}
                impl{full_generics} PartialEq for {trait_name}ID{short_generics} {{
                    fn eq(&self, other: &{trait_name}ID{short_generics}) -> bool {{
                        self._raw_id == other._raw_id
                    }}
                }}
                impl{full_generics} Eq for {trait_name}ID{short_generics} {{}}

                pub struct {trait_name}Representative{full_generics}{maybe_marker_only};

                impl{full_generics} ActorOrActorTrait for {trait_name}Representative{short_generics} {{
                    type ID = {trait_name}ID{short_generics};
                }}

                impl{full_generics} TypedID for {trait_name}ID{short_generics} {{
                    type Target = {trait_name}Representative{short_generics};

                    fn from_raw(id: RawID) -> Self {{
                        {trait_name}ID {{ _raw_id: id{maybe_marker_init} }}
                    }}

                    fn as_raw(&self) -> RawID {{
                        self._raw_id
                    }}
                }}

                impl<{full_generics_inner_comma}Act: Actor + {trait_name}{short_generics}> TraitIDFrom<Act> for {trait_name}ID{short_generics} {{}}

                impl{full_generics} {trait_name}ID{short_generics} {{
                    {handler_send_impls}

                    pub fn register_trait(system: &mut ActorSystem) {{
                        system.register_trait::<{trait_name}Representative{short_generics}>();
                        {trait_msg_registrations}
                    }}

                    pub fn register_implementor<Act: Actor + {trait_name}{short_generics}>(system: &mut ActorSystem) {{
                        system.register_implementor::<Act, {trait_name}Representative{short_generics}>();
                        {implementor_handler_registrations}
                    }}
                }}

                {msg_struct_defs}"#, trait_id_derives=trait_id_derives, trait_name=pth(trait_name), full_generics=full_generics, full_generics_inner_comma=full_generics_inner_comma, short_generics=short_generics, maybe_marker=maybe_marker, maybe_marker_only=maybe_marker_only, maybe_marker_init=maybe_marker_init, handler_send_impls=ind(&handler_send_impls, 5), trait_msg_registrations=&ind(&trait_msg_registrations, 6), implementor_handler_registrations=&ind(&implementor_handler_registrations, 6), msg_struct_defs=ind(&msg_struct_defs, 4)))
        }).collect::<Vec<_>>().join("\n")
    }

    pub fn generate_actor_ids_messages_and_conversions(&self) -> String {
        #[cfg(feature = "serde-serialization")]
        let actor_id_derives = unindent(r#"
            #[derive(Serialize, Deserialize)] #[serde(transparent)]"#);

        #[cfg(not(feature = "serde-serialization"))]
        let actor_id_derives = "".to_owned();

        self.actors.iter().map(|(actor_name, actor_def)| {
            let full_generics = to_full_generics(&actor_def.generics);
            let short_generics = to_short_generics(&actor_def.generics);
            let short_generics_turbofish = if short_generics.is_empty() {short_generics.clone()} else {format!("::{}", short_generics)};

            let (maybe_marker, maybe_marker_init) = if full_generics.is_empty() {
                ("".to_owned(), "")
            } else {
                (
                    format!(", _marker: ::std::marker::PhantomData<Box<({})>>", short_generics.replace("<", "").replace(">", "")),
                    ", _marker: ::std::marker::PhantomData"
                )
            };

            let id_def_if_defined_here = if actor_def.defined_here {
                unindent(&format!(r#"
                    impl{full_generics} Actor for {actor_name}{short_generics} {{
                        type ID = {actor_name}ID{short_generics};

                        fn id(&self) -> Self::ID {{
                            self.id
                        }}
                        unsafe fn set_id(&mut self, id: RawID) {{
                            self.id = Self::ID::from_raw(id);
                        }}
                    }}

                    {actor_id_derives}
                    pub struct {actor_name}ID{full_generics} {{
                        _raw_id: RawID{maybe_marker}
                    }}

                    impl{full_generics} Copy for {actor_name}ID{short_generics} {{}}
                    impl{full_generics} Clone for {actor_name}ID{short_generics} {{ fn clone(&self) -> Self {{ *self }} }}
                    impl{full_generics} ::std::fmt::Debug for {actor_name}ID{short_generics} {{
                        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{
                            write!(f, "{actor_name}ID{short_generics}({{:?}})", self._raw_id)
                        }}
                    }}
                    impl{full_generics} ::std::hash::Hash for {actor_name}ID{short_generics} {{
                        fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {{
                            self._raw_id.hash(state);
                        }}
                    }}
                    impl{full_generics} PartialEq for {actor_name}ID{short_generics} {{
                        fn eq(&self, other: &{actor_name}ID{short_generics}) -> bool {{
                            self._raw_id == other._raw_id
                        }}
                    }}
                    impl{full_generics} Eq for {actor_name}ID{short_generics} {{}}

                    impl{full_generics} TypedID for {actor_name}ID{short_generics} {{
                        type Target = {actor_name}{short_generics};

                        fn from_raw(id: RawID) -> Self {{
                            {actor_name}ID {{ _raw_id: id{maybe_marker_init} }}
                        }}

                        fn as_raw(&self) -> RawID {{
                            self._raw_id
                        }}
                    }}"#, actor_name=pth_t(actor_name), short_generics=short_generics, full_generics=full_generics, actor_id_derives=actor_id_derives, maybe_marker=maybe_marker, maybe_marker_init=maybe_marker_init))
            } else {"".to_owned()};

            let handler_defs = actor_def.handlers.iter().filter_map(|handler| {
                if handler.from_trait.is_some() {return None};
                let msg_prefix = typ_to_message_prefix(actor_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);
                let msg_params = handler.arguments.iter().filter_map(arg_as_value_without_world).collect::<Vec<_>>().join(", ");
                let handler_args = handler.arguments.iter().map(arg_as_ident_val_and_type).collect::<Vec<_>>().join(", ");

                match handler.scope {
                    HandlerType::Handler => {
                        Some(unindent(&format!(r#"
                            pub fn {handler_name}(self, {handler_args}) {{
                                world.send(self.as_raw(), {msg_name}({msg_params}));
                            }}"#, handler_name=handler.name, handler_args=handler_args, msg_name=msg_name, msg_params=msg_params)))
                    },
                    HandlerType::Init => {
                        Some(unindent(&format!(r#"
                            pub fn {handler_name}({handler_args}) -> Self {{
                                let id = {actor_name}ID{short_generics_turbofish}::from_raw(world.allocate_instance_id::<{actor_name}{short_generics}>());
                                let swarm = world.local_broadcast::<{actor_name}{short_generics}>();
                                world.send(swarm, {msg_name}(id, {msg_params}));
                                id
                            }}"#, handler_name=handler.name, short_generics_turbofish=short_generics_turbofish, short_generics=short_generics, handler_args=handler_args, actor_name=pth_t(actor_name), msg_name=msg_name, msg_params=msg_params)))
                    }
                }
            }).collect::<Vec<_>>().join("\n\n");

            let msg_defs = actor_def.handlers.iter().filter_map(|handler| {
                if handler.from_trait.is_some() {return None};
                let msg_prefix = typ_to_message_prefix(actor_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);
                let msg_param_types = handler.arguments.iter().filter_map(arg_as_pub_type_without_world).collect::<Vec<_>>().join(", ");
                let used_generics = handler.used_generics(short_generics.clone(), full_generics.clone(), false);

                let msg_derives = if msg_param_types.is_empty() && used_generics.is_empty() {
                    "#[derive(Copy, Clone)]"
                } else {
                    "#[derive(Compact, Clone)]"
                };

                let maybe_id_type = match handler.scope {
                    HandlerType::Handler => "".to_owned(),
                    HandlerType::Init => format!("pub {}ID{}, ", pth_t(actor_name), short_generics)
                };


                Some(unindent(&format!(r#"
                    {msg_derives} #[allow(non_camel_case_types)]
                    struct {msg_name}{used_generics}({maybe_id_type}{msg_param_types});"#, msg_derives=msg_derives, used_generics=used_generics, msg_name=msg_name, maybe_id_type=maybe_id_type, msg_param_types=msg_param_types)))
            }).collect::<Vec<_>>().join("\n");

            let id_conversion_impls = actor_def.impls.iter().map(|trait_name| {
                let impl_generics = generics_from_path(trait_name);
                unindent(&format!(r#"
                    impl Into<{trait_name}ID{impl_generics}> for {actor_name}ID{short_generics} {{
                        fn into(self) -> {trait_name}ID{impl_generics} {{
                            {trait_name}ID::from_raw(self.as_raw())
                        }}
                    }}"#, trait_name=pth(trait_name), actor_name=pth_t(actor_name), impl_generics=impl_generics, short_generics=short_generics))
            }).collect::<Vec<_>>().join("\n\n");

            unindent(&format!(r#"
                {id_def_if_defined_here}

                impl{full_generics} {actor_name}ID{short_generics} {{
                    {handler_defs}
                }}

                {msg_defs}

                {id_conversion_impls}"#, id_def_if_defined_here=ind(&id_def_if_defined_here, 4), actor_name=pth_t(actor_name), short_generics=short_generics, full_generics=full_generics, handler_defs=ind(&handler_defs, 5), msg_defs=ind(&msg_defs, 4), id_conversion_impls=ind(&id_conversion_impls, 4)))
        }).collect::<Vec<_>>().join("\n")
    }
}

fn typ_to_message_prefix(typ: &Type) -> String {
    let segments = if let Type::Path(TypePath{path: Path{ref segments, .. }, ..}) = *typ {
        segments
    } else {
        unimplemented!()
    };

    let prefixed = segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("_");
    format!("MSG_{}", prefixed)
}

fn trait_to_message_prefix(path: &Path) -> String {
    format!("MSG_{}", path.segments.last().unwrap().value().ident)
}

fn is_type_world(ty: &Type) -> bool {
    match ty {
        Type::Reference(TypeReference{ref elem,.. }) => is_type_world(elem),
        Type::Path(TypePath{ref path, ..}) => {
            path.segments.last().unwrap().value().ident == "World"
        },
        _ => false
    }
}

fn arg_pat_to_ident(pat: &Pat, ty: &Type) -> String {
    match pat {
        Pat::Ident(PatIdent{ref ident, ..}) => ident.to_string().trim_start_matches("_").to_owned(),
        Pat::Wild(PatWild{..}) => if is_type_world(ty) {"world".to_owned()} else {"_".to_owned()},
        _ => unimplemented!("{:?}", pat)
    }
}

fn arg_ref_to_bind_as_ref_without_world(arg: &FnArg) -> Option<String> {
    match *arg {
        FnArg::Captured(ArgCaptured{ref pat, ref ty, ..}) => {
            let ident = arg_pat_to_ident(pat, ty);

            match *ty {
                Type::Reference(_) => {
                    if is_type_world(ty) {
                        None
                    } else {
                        Some(format!("ref {}", ident))
                    }
                }
                _ => Some(ident.to_string())
            }
        },
        _ => unimplemented!("{:?}", arg),
    }
}

fn arg_as_ident_val_and_type(arg: &FnArg) -> String {
    match arg {
        FnArg::Captured(captured_arg) => {
            let (pat, ty_string) = match captured_arg {
                ArgCaptured{ref pat, ty: Type::Reference(TypeReference{elem: ref refd_ty, ..}), ..} => {
                    if is_type_world(&captured_arg.ty) {
                        (pat, "&mut World".to_owned())
                    } else {
                        (pat, refd_ty.into_token_stream().to_string())
                    }
                },
                ArgCaptured{ref pat, ty: ref other_ty, ..} => {
                    (pat, other_ty.into_token_stream().to_string())
                }
            };

            let ident = arg_pat_to_ident(pat, &captured_arg.ty);

            format!("{}: {}", ident, ty_string)
        },
        _ => unimplemented!("{:?}", arg),
    }
}

fn arg_as_value(arg: &FnArg) -> String {
    match *arg {
        FnArg::Captured(ArgCaptured{ref pat, ref ty, ..}) => arg_pat_to_ident(pat, ty),
        _ => unimplemented!("{:?}", arg),
    }
}

fn arg_as_value_without_world(arg: &FnArg) -> Option<String> {
    match *arg {
        FnArg::Captured(ArgCaptured{ref pat, ref ty, ..}) => {
            if is_type_world(ty) {
                None
            } else {
                Some(arg_pat_to_ident(pat, ty))
            }
        }
        _ => unimplemented!(),
    }
}

fn arg_as_pub_type_without_world(arg: &FnArg) -> Option<String> {
    match *arg {
        FnArg::Captured(ArgCaptured{ref ty, ..}) => {
            if is_type_world(ty) {
                None
            } else {
                if let Type::Reference(TypeReference{elem: ref refd_type, ..}) = ty {
                    Some(refd_type.into_token_stream().to_string())
                } else {
                    Some(ty.into_token_stream().to_string())
                }
            }
        }
        _ => unimplemented!(),
    }.map(|string| format!("pub {}", string))
}