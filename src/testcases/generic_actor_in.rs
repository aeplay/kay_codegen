pub struct SomeActor<A: Compact, B: Compact> {
    id: SomeActorID<A, B>,
    field: usize,
    thing: B
}

impl<A: Compact, B: Compact> SomeActor<A, B> {
    pub fn some_method(&mut self, some_param: usize, thing: &B, world: &mut World) {
        self.id().some_method(42, world);
    }

    pub fn no_params_fate(&mut self, world: &mut World) -> Fate {
        Fate::Die
    }

    pub fn init_ish(id: SomeActorID<A, B>, some_param: usize, world: &mut World) -> SomeActor<A, B> {
        SomeActor {
            id: id,
            field: some_param,
            thing: B::new()
        }
    }
}