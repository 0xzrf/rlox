use cli::run;

fn main() {
    if let Err(e) = run() {
        println!("Error occured: {}", e);
    }
}
