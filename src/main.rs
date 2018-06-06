#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate sc2_kill_calculator;

fn main_impl() -> Result<(), sc2_kill_calculator::error::Error> {
    stderrlog::new()
        .module(module_path!())
        .init()
        .expect("StdErrLog::init() failed");
    sc2_kill_calculator::rocket()?.launch();
    Ok(())
}

fn main() {
    main_impl().expect("An error occurred")
}
