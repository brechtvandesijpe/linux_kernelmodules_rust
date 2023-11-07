cmd_samples/rust/modules.order := {  :; } | awk '!x[$$0]++' - > samples/rust/modules.order
