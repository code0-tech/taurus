//! Development-only module for exercising flow-type features end-to-end
//! without touching a real domain module. Only registered while Taurus runs
//! with `ENVIRONMENT=development` (the default) -- see `dev_only` on
//! `taurus_macros::module!`.

taurus_macros::module! {
    identifier = "taurus-dev",
    name(en_US = "Development"),
    description(en_US = "Development-only flow types and definitions, not shipped to staging or production."),
    documentation = "",
    author = "CodeZero",
    icon = "tabler:test-pipe",
    version = "0.0.1",
    dev_only,
}

taurus_macros::flow_type! {
    identifier = "MANUAL",
    module = "taurus-dev",
    signature = "(): void",
    name(en_US = "Manual"),
    description(en_US = "A flow started manually, with no configurable settings."),
    display_message(en_US = "Manual"),
    alias(en_US = "manual;trigger;dev;test"),
    display_icon = "tabler:test-pipe",
}
