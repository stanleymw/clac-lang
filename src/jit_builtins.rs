use thiserror::Error;

pub(crate) extern "C" fn quit() {
    std::process::exit(0);
}

#[derive(Debug, Error)]
#[repr(i64)]
pub(crate) enum CompiledExecutionError {
    #[error("An error occured! Clac exiting.")]
    Error,
}

pub(crate) extern "C" fn error(err: CompiledExecutionError) {
    eprintln!("{}", err);
    std::process::exit(0);
}

pub(crate) extern "C" fn print_value(val: crate::types::Value) {
    println!("{}", val)
}
