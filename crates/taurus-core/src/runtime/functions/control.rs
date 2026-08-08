//! Control-flow handlers (`if`, `if_else`, `return`, `stop`).
//!
//! `if`/`if_else` execute branch nodes via runtime callbacks and forward their resulting signals.
//! This is required for block-style return semantics where `return` exits only the current call frame.

use crate::handler::argument::Argument;
use crate::handler::macros::args;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use tucana::shared::Value;
use tucana::shared::value::Kind;

taurus_macros::module! {
    identifier = "taurus-control",
    name(en_US = "Control"),
    description(en_US = "Flow control."),
    documentation = "",
    author = "CodeZero",
    icon = "tabler:arrow-ramp-right-2",
    version = "0.0.33",
}

taurus_macros::data_type! {
    identifier = "COMPARATOR",
    module = "taurus-control",
    name(en_US = "Comparator"),
    display_message(en_US = "Compare ${I}"),
    alias(en_US = "compare;comparator"),
    generic_keys = ["I"],
    type_string = "(left: I, right: I) => NUMBER",
    linked_data_type_identifiers = ["NUMBER"],
}

taurus_macros::data_type! {
    identifier = "CONSUMER",
    module = "taurus-control",
    name(en_US = "Consumer"),
    display_message(en_US = "Use ${T}"),
    alias(en_US = "use;consumer;consume;lambda"),
    generic_keys = ["T"],
    type_string = "(item: T) => void",
}

taurus_macros::data_type! {
    identifier = "PREDICATE",
    module = "taurus-control",
    name(en_US = "Predicate"),
    display_message(en_US = "Predicate of ${T}"),
    alias(en_US = "predicate"),
    generic_keys = ["T"],
    type_string = "(item: T) => BOOLEAN",
    linked_data_type_identifiers = ["BOOLEAN"],
}

taurus_macros::data_type! {
    identifier = "RUNNABLE",
    module = "taurus-control",
    name(en_US = "Node"),
    display_message(en_US = "Node"),
    alias(en_US = "node;function"),
    type_string = "() => void",
}

taurus_macros::data_type! {
    identifier = "TRANSFORM",
    module = "taurus-control",
    name(en_US = "Transform"),
    display_message(en_US = "Transform ${I} to ${R}"),
    alias(en_US = "transform"),
    generic_keys = ["I", "R"],
    type_string = "(item: I) => R",
}

#[taurus_macros::runtime_function(
    identifier = "std::control::stop",
    module = "taurus-control",
    signature = "(): void",
    name(en_US = "Stop"),
    description(
        en_US = "Terminates the current execution context entirely, halting all ongoing and future iterations. Once invoked, no additional steps or loops within this context will be executed. This node behaves like a global stop or termination signal within the flow."
    ),
    display_message(en_US = "Stop"),
    alias(en_US = "stop;control;std"),
    display_icon = "tabler:xbox-x"
)]
fn stop(
    _args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    Signal::Stop
}

#[taurus_macros::runtime_function(
    identifier = "std::control::value",
    module = "taurus-control",
    signature = "<T>(value: T): T",
    name(en_US = "Set Value"),
    description(en_US = "Saves the given value."),
    display_message(en_US = "Save ${value}"),
    alias(en_US = "value;save;create;control;std"),
    display_icon = "tabler:new-section"
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Value"),
    description(en_US = "The value to save.")
)]
fn value(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: Value);
    Signal::Success(value)
}

#[taurus_macros::runtime_function(
    identifier = "std::control::return",
    module = "taurus-control",
    signature = "<T>(value: T): T",
    name(en_US = "Return"),
    description(
        en_US = "Ends the current context and returns the specified value to the upper scope."
    ),
    display_message(en_US = "Return ${value}"),
    alias(en_US = "return;control;std"),
    display_icon = "tabler:arrow-back"
)]
#[parameter(
    runtime_name = "value",
    name(en_US = "Return Value"),
    description(en_US = "The value to be returned to the upper context.")
)]
fn r#return(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    args!(args => value: Value);
    // The executor decides how far this return unwinds (one frame).
    Signal::Return(value)
}

#[taurus_macros::runtime_function(
    identifier = "std::control::if",
    module = "taurus-control",
    signature = "(condition: BOOLEAN, runnable: RUNNABLE): void",
    name(en_US = "If"),
    description(
        en_US = "The 'If' runnable evaluates a boolean condition and, if it is true, executes the provided runnable. If the condition is false, execution continues without running the runnable."
    ),
    display_message(en_US = "If ${condition} is True do ${runnable}"),
    alias(en_US = "if;control;std"),
    display_icon = "tabler:arrow-ramp-right-2",
    linked_data_type_identifiers = ["BOOLEAN", "RUNNABLE"],
    param_modes = [Eager, Lazy],
)]
#[parameter(
    runtime_name = "condition",
    name(en_US = "Condition"),
    description(
        en_US = "Specifies the condition that determines whether the provided runnable should be executed. If this condition evaluates to true, the execution proceeds with the runnable defined in the second parameter."
    )
)]
#[parameter(
    runtime_name = "runnable",
    name(en_US = "Runnable"),
    description(en_US = "Defines the runnable that runs if the condition evaluates to true.")
)]
fn r#if(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    let [
        Argument::Eval(Value {
            kind: Some(Kind::BoolValue(bool)),
        }),
        Argument::Thunk(if_pointer),
    ] = args
    else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            format!("Expected a bool value but received {:?}", args),
        ));
    };

    if *bool {
        // Branch execution is delegated to the executor through `run`.
        ctx.push_runtime_trace_label(|| "branch=if".to_string());
        run(if_pointer, ctx)
    } else {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }
}

#[taurus_macros::runtime_function(
    identifier = "std::control::if_else",
    module = "taurus-control",
    signature = "(condition: BOOLEAN, runnable: RUNNABLE, else_runnable: RUNNABLE): void",
    name(en_US = "If-Else"),
    description(en_US = "Evaluates a condition and executes either the Then Runnable or the Else Runnable."),
    documentation(
        en_US = "Evaluates a boolean condition. If true, executes the Runnable. Otherwise executes the Else Runnable."
    ),
    display_message(en_US = "If ${condition} is True do ${then_runnable} else ${else_runnable}"),
    alias(en_US = "if_else;control;std;if;else"),
    display_icon = "tabler:arrow-ramp-right",
    linked_data_type_identifiers = ["BOOLEAN", "RUNNABLE"],
    param_modes = [Eager, Lazy, Lazy],
)]
#[parameter(
    runtime_name = "condition",
    name(en_US = "Condition"),
    description(
        en_US = "Boolean that determines which branch to execute. If true the Runnable runs otherwise the Else Runnable runs."
    )
)]
#[parameter(
    runtime_name = "runnable",
    name(en_US = "Runnable"),
    description(en_US = "Defines the runnable that runs if the condition evaluates to true.")
)]
#[parameter(
    runtime_name = "else_runnable",
    name(en_US = "Else Runnable"),
    description(en_US = "Defines the runnable that runs if the condition evaluates to false.")
)]
fn if_else(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut crate::handler::registry::ThunkRunner<'_>,
) -> Signal {
    let [
        Argument::Eval(Value {
            kind: Some(Kind::BoolValue(bool)),
        }),
        Argument::Thunk(if_pointer),
        Argument::Thunk(else_pointer),
    ] = args
    else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            format!("Expected a bool value but received {:?}", args),
        ));
    };

    if *bool {
        ctx.push_runtime_trace_label(|| "branch=if".to_string());
        run(if_pointer, ctx)
    } else {
        ctx.push_runtime_trace_label(|| "branch=else".to_string());
        run(else_pointer, ctx)
    }
}
