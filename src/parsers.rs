use {Model, TraitName, Handler, HandlerType};
use syn::*;

pub fn parse(file: &str) -> ::std::result::Result<Model, parse::Error> {
    let mut model = Model::default();

    let parsed = parse_file(file)?;

    for item in &parsed.items {
        match item {
            Item::Struct(ItemStruct{ident, generics, ..}) => {
                let ident_as_seg: PathSegment = ident.clone().into();
                let actor_def = model
                    .actors
                    .entry(TypePath{qself: None, path: ::syn::Path::from(ident_as_seg)}.into())
                    .or_insert_with(Default::default);
                actor_def.defined_here = true;
                actor_def.generics = generics.clone();
            }
            Item::Impl(ItemImpl{trait_: ref maybe_trait, self_ty: ref actor_name, items: ref impl_items, generics, ..}) => {
                let actor_name_no_args = match **actor_name {
                   Type::Path(TypePath{ref path, ..}) => TypePath{qself: None, path: ::syn::Path::from(path.segments[0].ident.clone())},
                    _ => unimplemented!(),
                };
                let actor_def = model
                    .actors
                    .entry(actor_name_no_args.into())
                    .or_insert_with(Default::default);
                actor_def.generics = generics.clone();
                let actor_path = match **actor_name {
                    Type::Path(TypePath{ref path, ..}) => path,
                    _ => unimplemented!(),
                };
                if let Some((_, ref trait_name, _)) = *maybe_trait {
                    let new_actor_handlers = handlers_from_impl_items(
                        impl_items,
                        &Some(trait_name.clone()),
                        &actor_path,
                    );
                    actor_def.impls.push(trait_name.clone());
                    actor_def.handlers.extend(new_actor_handlers);
                } else {
                    actor_def
                        .handlers
                        .extend(handlers_from_impl_items(impl_items, &None, actor_path));
                }
            }
            Item::Trait(ItemTrait{ident, items: ref trait_items, ..}) => {
                let trait_name: TraitName = ::syn::Path::from(PathSegment::from(ident.clone()));
                let trait_def = model
                    .traits
                    .entry(trait_name.clone())
                    .or_insert_with(Default::default);
                let as_segment: PathSegment = ident.clone().into();
                trait_def.handlers.extend(handlers_from_trait_items(
                    trait_items,
                    &::syn::Path::from(as_segment),
                ));
            }
            _ => {}
        }
    }

    for (_, actor_def) in &mut model.actors {
        // TODO: this is a horrible hack, figure out a way to distinguish ActorTraits globally
        actor_def.impls.retain(|trait_name| {
            ![
                "Deref",
                "DerefMut",
                "Default",
                "Clone",
                "Into",
                "From",
                "Add",
                "AddAssign",
                "Sum",
            ].contains(&trait_name.segments.last().unwrap().value().ident.to_string().as_str())
        });
    }

    model.actors.retain(|ref _name, ref actor_def| {
        !actor_def.handlers.is_empty() || !actor_def.impls.is_empty()
    });

    model
        .traits
        .retain(|ref _name, ref trait_def| !trait_def.handlers.is_empty());

    Ok(model)
}

fn handlers_from_impl_items(
    impl_items: &[ImplItem],
    with_trait: &Option<TraitName>,
    parent_path: &::syn::Path,
) -> Vec<Handler> {
    impl_items
        .iter()
        .filter_map(|impl_item| {
            if let ImplItem::Method(ImplItemMethod {
                ref vis,
                ref sig,
                ref attrs,
                ..
            }) = *impl_item
            {
                match (with_trait, vis) {
                    (Some(_), _) | (_, Visibility::Public(_)) => {
                        handler_from(sig, attrs, with_trait, parent_path)
                    },
                    _ => None
                }
            } else {
                None
            }
        })
        .collect()
}

fn handlers_from_trait_items(trait_items: &[TraitItem], parent_path: &::syn::Path) -> Vec<Handler> {
    trait_items
        .iter()
        .filter_map(|trait_item| {
            if let TraitItem::Method(TraitItemMethod {
                ref sig,
                ..
            }) = *trait_item
            {
                handler_from(sig, &[], &None, parent_path)
            } else {
                None
            }
        })
        .collect()
}

fn handler_from(
    sig: &MethodSig,
    attrs: &[Attribute],
    from_trait: &Option<TraitName>,
    parent_path: &::syn::Path,
) -> Option<Handler> {
    check_handler(sig, parent_path).and_then(|(args, scope)| {
        let returns_fate = match sig.decl.output {
            ReturnType::Default => false,
            ReturnType::Type(_, ref type_box) => {
                match &**type_box {
                    Type::Path(TypePath{qself: _, path: ::syn::Path { ref segments, .. }}) => {
                        if segments.iter().any(|s| s.ident.to_string() == "Fate") {
                            true
                        } else if scope == HandlerType::Init {
                            false
                        } else {
                            return None;
                        }
                    },
                    _ => return None
                }
            }
        };

        let is_critical = attrs.iter().any(|attr| {
            format!("{:?}", attr.path) == "doc" && attr.tts.to_string() == "/// Critical"
        });

        Some(Handler {
            name: sig.ident.clone(),
            arguments: args,
            scope,
            critical: is_critical,
            returns_fate,
            from_trait: from_trait.clone(),
        })
    })
}

pub fn check_handler<'a>(
    sig: &'a MethodSig,
    parent_path: &::syn::Path,
) -> Option<(Vec<FnArg>, HandlerType)> {
    if let Some(FnArg::Captured(ArgCaptured{ty: Type::Reference(TypeReference{elem: ref ty_box, ref mutability, ..}), ..})) = sig.decl.inputs.last().map(|pair| pair.value().clone()) {
        if let (Some(_), Type::Path(TypePath{ref path, ..})) = (mutability, &**ty_box)
        {
            if path.segments.last().unwrap().value().ident.to_string() == "World" {
                match sig.decl.inputs.first().map(|pair| pair.value().clone()) {
                    Some(&FnArg::SelfRef(..)) => {
                        let args = sig.decl.inputs.iter().skip(1).cloned().collect();
                        Some((args, HandlerType::Handler))
                    }
                    Some(&FnArg::SelfValue(_)) => None,
                    _ => {
                        match sig.decl.output {
                            ReturnType::Type(_, ref type_box) =>
                                match &**type_box {
                                    Type::Path(TypePath{path: ref ret_ty_path, ..}) if ret_ty_path.segments.last().unwrap().value().ident.to_string() == "Self" || *ret_ty_path == *parent_path => {
                                        let args = sig.decl.inputs.iter().skip(1).cloned().collect();
                                        Some((args, HandlerType::Init))
                                    },
                                    _ => None,
                                },
                            _ => None,
                        }
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
