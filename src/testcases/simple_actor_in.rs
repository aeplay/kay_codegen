pub struct SomeActor {
    id: SomeActorID,
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