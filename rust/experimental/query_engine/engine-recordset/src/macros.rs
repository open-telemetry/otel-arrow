#[macro_use]
mod macros {
    //#[macro_export]
    macro_rules! either {
        ($test:expr => $true_expr:expr; $false_expr:expr) => {
            if $test { $true_expr } else { $false_expr }
        };
    }
}
