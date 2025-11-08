#[derive(Clone, Debug)]
pub enum Argument {
    // Eval => Evaluated Value
    // - can be consumed directly by a function
    Eval(tucana::shared::Value),
    // Thunk of NodeFunction identifier
    // - used for lazy execution of nodes
    Thunk(i64)
}

#[derive(Clone, Copy, Debug)]
pub enum ParameterNode {
    Eager,
    Lazy
}