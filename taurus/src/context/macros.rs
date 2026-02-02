/// Pulls typed parameters from a slice of `Argument` using your `TryFromArgument`
/// impls. Fails early with your `Signal::Failure(RuntimeError::simple(...))`.
macro_rules! args {
    ($args_ident:ident => $( $name:ident : $ty:ty ),+ $(,)?) => {
        // Arity check
        let __expected: usize = 0usize $(+ { let _ = ::core::any::type_name::<$ty>(); 1usize })*;
        if $args_ident.len() != __expected {
            return $crate::context::signal::Signal::Failure(
                $crate::error::RuntimeError::simple(
                    "InvalidArgumentRuntimeError",
                    format!("Expected {__expected} args but received {}", $args_ident.len()),
                )
            );
        }

        // Typed extraction
        let mut __i: usize = 0;
        $(
            let $name: $ty = match <
                $ty as $crate::context::argument::TryFromArgument
            >::try_from_argument(& $args_ident[__i]) {
                Ok(v) => v,
                Err(sig) => {
                    log::debug!(
                        "Failed to parse argument '{}' (index {}, type {})",
                        stringify!($name),
                        __i,
                        ::core::any::type_name::<$ty>(),
                    );
                    return sig;
                }
            };
            __i += 1;
        )+
    };
}

/// Asserts there are no arguments.
macro_rules! no_args {
    ($args_ident:ident) => {
        if !$args_ident.is_empty() {
            return $crate::context::signal::Signal::Failure($crate::error::RuntimeError::simple(
                "InvalidArgumentRuntimeError",
                format!("Expected 0 args but received {}", $args_ident.len()),
            ));
        }
    };
}

pub(crate) use args;
pub(crate) use no_args;
