#[cfg(not(target_os = "macos"))]
fn main() {
    panic!("This example only supports macOS.");
}

#[cfg(target_os = "macos")]
fn main() {
    use window_observer::platform_impl::macos::binding_ax_function::{
        ax_is_process_trusted, ax_is_process_trusted_with_options,
    };

    println!("Is process trusted: {}", ax_is_process_trusted());

    println!("Prompt if process is not trusted");
    ax_is_process_trusted_with_options(true);
}
