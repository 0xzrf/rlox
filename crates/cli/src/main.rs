use cli::run;

fn main() {
    match run() {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(e) => {
            eprint!("Error Occured: {}", e);
            std::process::exit(65);
        }
    }
}
