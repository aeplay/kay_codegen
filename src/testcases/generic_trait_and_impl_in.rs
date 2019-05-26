pub struct SomeActor {
    id: SomeActorID,
    field: usize
}

trait SomeTrait<A: Compact, B: Compact> {
    fn some_method(&mut self, some_param: A, world: &mut World);
    fn no_params_fate(&mut self, world: &mut World) -> Fate;
    fn some_default_impl_method(&mut self, world: &mut World) {
        self.some_method(3, world);
    }
}

impl SomeTrait<usize, isize> for SomeActor {
    fn some_method(&mut self, some_param: usize, world: &mut World) {
        self.id().some_method(42, world);
    }

    fn no_params_fate(&mut self, world: &mut World) -> Fate {
        Fate::Die
    }
}

impl ForeignTrait<usize, isize> for SomeActor {
    fn simple(&mut self, some_param: usize, world: &mut World) {
        self.id().some_method(some_param, world);
    }
}