use {Model, HandlerType};
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

impl Model {
    pub fn generate_setups(&self) -> String {
        let trait_registrations = self.traits.keys().map(|trait_name|
            format!("{trait_name}ID::register_trait(system);", trait_name=pth(trait_name))
        ).collect::<Vec<_>>().join("\n");

        let actor_setups = self.actors.iter().map(|(actor_name, actor_def)| {
            let impl_registrations = actor_def.impls.iter().map(|trait_name|
                format!("{trait_name}ID::register_implementor::<{actor_name}>(system);", trait_name=pth(trait_name), actor_name=pth_t(actor_name))
            ).collect::<Vec<_>>().join("\n");

            let handler_registrations = actor_def.handlers.iter().filter_map(|handler|
                if handler.from_trait.is_none() {
                    let msg_name = format!("{}_{}", typ_to_message_prefix(actor_name), handler.name);
                    let is_critical = if handler.critical {"true"} else {"false"};
                    let msg_args = handler.arguments.iter().filter_map(arg_ref_to_bind_as_ref_without_world).collect::<Vec<_>>().join(", ");
                    let handler_params = handler.arguments.iter().map(arg_as_value).collect::<Vec<_>>().join(", ");

                    Some(match handler.scope {
                        HandlerType::Handler => {
                            let maybe_return = if handler.returns_fate {""} else {"; Fate::Live"};

                            unindent(&format!(r#"
                                system.add_handler::<{actor_name}, _, _>(
                                    |&{msg_name}({msg_args}), instance, world| {{
                                        instance.{handler_name}({handler_params}){maybe_return}
                                    }}, {is_critical}
                                );"#, actor_name=pth_t(actor_name), msg_name=msg_name, msg_args=msg_args, handler_name=handler.name, handler_params=handler_params, maybe_return=maybe_return, is_critical=is_critical))
                        },
                        HandlerType::Init => {
                            unindent(&format!(r#"
                                system.add_spawner::<{actor_name}, _, _>(
                                    |&{msg_name}(id, {msg_args}), world| {{
                                        {actor_name}::{handler_name}(id, {handler_params})
                                    }}, {is_critical}
                                );"#, actor_name=pth_t(actor_name), msg_name=msg_name, msg_args=msg_args, handler_name=handler.name, handler_params=handler_params, is_critical=is_critical))
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

        unindent(&format!(r#"
            #[allow(unused_variables)]
            #[allow(unused_mut)]
            pub fn auto_setup(system: &mut ActorSystem) {{
                {trait_registrations}
                {actor_setups}
            }}"#, trait_registrations=ind(&trait_registrations, 4), actor_setups=ind(&actor_setups, 4)))
    }

    pub fn generate_traits(&self) -> String {
        #[cfg(feature = "serde-serialization")]
        let trait_id_derives = unindent(r#"
            #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)] #[serde(transparent)]"#);

        #[cfg(not(feature = "serde-serialization"))]
        let trait_id_derives = unindent(r#"
            #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]"#);

        self.traits.iter().map(|(trait_name, trait_def)| {
            let handler_send_impls = trait_def.handlers.iter().map(|handler| {
                let handler_args = handler.arguments.iter().map(arg_as_ident_val_and_type).collect::<Vec<_>>().join(", ");
                let msg_prefix = trait_to_message_prefix(trait_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);
                let msg_params = handler.arguments.iter().filter_map(arg_as_value_without_world).collect::<Vec<_>>().join(", ");

                unindent(&format!(r#"
                    pub fn {handler_name}(&self, {handler_args}) {{
                        world.send(self.as_raw(), {msg_name}({msg_params}));
                    }}"#, handler_name=handler.name, handler_args=handler_args, msg_name=msg_name, msg_params=msg_params))
            }).collect::<Vec<_>>().join("\n\n");

            let trait_msg_registrations = trait_def.handlers.iter().map(|handler| {
                let msg_prefix = trait_to_message_prefix(trait_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);
                format!("system.register_trait_message::<{}>();", msg_name)
            }).collect::<Vec<_>>().join("\n");

            let implementor_handler_registrations = trait_def.handlers.iter().map(|handler| {
                let msg_prefix = trait_to_message_prefix(trait_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);

                let msg_args = handler.arguments.iter().filter_map(arg_ref_to_bind_as_ref_without_world).collect::<Vec<_>>().join(", ");
                let handler_params = handler.arguments.iter().map(arg_as_value).collect::<Vec<_>>().join(", ");

                let maybe_return = if handler.returns_fate {""} else {"; Fate::Live"};
                let is_critical = if handler.critical {"true"} else {"false"};

                unindent(&format!(r#"
                    system.add_handler::<A, _, _>(
                        |&{msg_name}({msg_args}), instance, world| {{
                            instance.{handler_name}({handler_params}){maybe_return}
                        }}, {is_critical}
                    );"#, msg_name=msg_name, msg_args=msg_args, handler_name=handler.name, handler_params=handler_params, maybe_return=maybe_return, is_critical=is_critical))
            }).collect::<Vec<_>>().join("\n\n");

            let msg_struct_defs = trait_def.handlers.iter().map(|handler| {
                let msg_prefix = trait_to_message_prefix(trait_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);

                let msg_param_types = handler.arguments.iter().filter_map(arg_as_pub_type_without_world).collect::<Vec<_>>().join(", ");
                let msg_derives = if msg_param_types.is_empty() {
                    "#[derive(Copy, Clone)]"
                } else {
                    "#[derive(Compact, Clone)]"
                };

                unindent(&format!(r#"
                    {msg_derives} #[allow(non_camel_case_types)]
                    struct {msg_name}({msg_param_types});"#, msg_derives=msg_derives, msg_name=msg_name, msg_param_types=msg_param_types))
            }).collect::<Vec<_>>().join("\n");

            unindent(&format!(r#"
                {trait_id_derives}
                pub struct {trait_name}ID {{
                    _raw_id: RawID
                }}

                pub struct {trait_name}Representative;

                impl ActorOrActorTrait for {trait_name}Representative {{
                    type ID = {trait_name}ID;
                }}

                impl TypedID for {trait_name}ID {{
                    type Target = {trait_name}Representative;

                    fn from_raw(id: RawID) -> Self {{
                        {trait_name}ID {{ _raw_id: id }}
                    }}

                    fn as_raw(&self) -> RawID {{
                        self._raw_id
                    }}
                }}

                impl<A: Actor + {trait_name}> TraitIDFrom<A> for {trait_name}ID {{}}

                impl {trait_name}ID {{
                    {handler_send_impls}

                    pub fn register_trait(system: &mut ActorSystem) {{
                        system.register_trait::<{trait_name}Representative>();
                        {trait_msg_registrations}
                    }}

                    pub fn register_implementor<A: Actor + {trait_name}>(system: &mut ActorSystem) {{
                        system.register_implementor::<A, {trait_name}Representative>();
                        {implementor_handler_registrations}
                    }}
                }}

                {msg_struct_defs}"#, trait_id_derives=trait_id_derives, trait_name=pth(trait_name), handler_send_impls=ind(&handler_send_impls, 5), trait_msg_registrations=&ind(&trait_msg_registrations, 6), implementor_handler_registrations=&ind(&implementor_handler_registrations, 6), msg_struct_defs=ind(&msg_struct_defs, 4)))
        }).collect::<Vec<_>>().join("\n")
    }

    pub fn generate_actor_ids_messages_and_conversions(&self) -> String {
        #[cfg(feature = "serde-serialization")]
        let actor_id_derives = unindent(r#"
            #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)] #[serde(transparent)]"#);

        #[cfg(not(feature = "serde-serialization"))]
        let actor_id_derives = unindent(r#"
            #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]"#);

        self.actors.iter().map(|(actor_name, actor_def)| {
            let id_def_if_defined_here = if actor_def.defined_here {
                unindent(&format!(r#"
                    impl Actor for {actor_name} {{
                        type ID = {actor_name}ID;

                        fn id(&self) -> Self::ID {{
                            self.id
                        }}
                        unsafe fn set_id(&mut self, id: RawID) {{
                            self.id = Self::ID::from_raw(id);
                        }}
                    }}

                    {actor_id_derives}
                    pub struct {actor_name}ID {{
                        _raw_id: RawID
                    }}

                    impl TypedID for {actor_name}ID {{
                        type Target = {actor_name};

                        fn from_raw(id: RawID) -> Self {{
                            {actor_name}ID {{ _raw_id: id }}
                        }}

                        fn as_raw(&self) -> RawID {{
                            self._raw_id
                        }}
                    }}"#, actor_name=pth_t(actor_name), actor_id_derives=actor_id_derives))
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
                            pub fn {handler_name}(&self, {handler_args}) {{
                                world.send(self.as_raw(), {msg_name}({msg_params}));
                            }}"#, handler_name=handler.name, handler_args=handler_args, msg_name=msg_name, msg_params=msg_params)))
                    },
                    HandlerType::Init => {
                        Some(unindent(&format!(r#"
                            pub fn {handler_name}({handler_args}) -> Self {{
                                let id = {actor_name}ID::from_raw(world.allocate_instance_id::<{actor_name}>());
                                let swarm = world.local_broadcast::<{actor_name}>();
                                world.send(swarm, {msg_name}(id, {msg_params}));
                                id
                            }}"#, handler_name=handler.name, handler_args=handler_args, actor_name=pth_t(actor_name), msg_name=msg_name, msg_params=msg_params)))
                    }
                }
            }).collect::<Vec<_>>().join("\n\n");

            let msg_defs = actor_def.handlers.iter().filter_map(|handler| {
                if handler.from_trait.is_some() {return None};
                let msg_prefix = typ_to_message_prefix(actor_name);
                let msg_name = format!("{}_{}", msg_prefix, handler.name);
                let msg_param_types = handler.arguments.iter().filter_map(arg_as_pub_type_without_world).collect::<Vec<_>>().join(", ");

                let msg_derives = if msg_param_types.is_empty() {
                    "#[derive(Copy, Clone)]"
                } else {
                    "#[derive(Compact, Clone)]"
                };

                let maybe_id_type = match handler.scope {
                    HandlerType::Handler => "".to_owned(),
                    HandlerType::Init => format!("pub {}ID, ", pth_t(actor_name))
                };

                Some(unindent(&format!(r#"
                    {msg_derives} #[allow(non_camel_case_types)]
                    struct {msg_name}({maybe_id_type}{msg_param_types});"#, msg_derives=msg_derives, msg_name=msg_name, maybe_id_type=maybe_id_type, msg_param_types=msg_param_types)))
            }).collect::<Vec<_>>().join("\n");

            let id_conversion_impls = actor_def.impls.iter().map(|trait_name| {
                unindent(&format!(r#"
                    impl Into<{trait_name}ID> for {actor_name}ID {{
                        fn into(self) -> {trait_name}ID {{
                            {trait_name}ID::from_raw(self.as_raw())
                        }}
                    }}"#, trait_name=pth(trait_name), actor_name=pth_t(actor_name)))
            }).collect::<Vec<_>>().join("\n\n");

            unindent(&format!(r#"
                {id_def_if_defined_here}

                impl {actor_name}ID {{
                    {handler_defs}
                }}

                {msg_defs}

                {id_conversion_impls}"#, id_def_if_defined_here=ind(&id_def_if_defined_here, 4), actor_name=pth_t(actor_name), handler_defs=ind(&handler_defs, 5), msg_defs=ind(&msg_defs, 4), id_conversion_impls=ind(&id_conversion_impls, 4)))
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
        Pat::Ident(PatIdent{ref ident, ..}) => ident.to_string().trim_left_matches("_").to_owned(),
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