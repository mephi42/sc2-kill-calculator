extern crate sc2_kill_calculator;

fn main_impl() -> Result<(), sc2_kill_calculator::error::Error> {
    sc2_kill_calculator::rocket()?.launch();
    Ok(())
}

fn main() {
    main_impl().expect("An error occurred")
}
