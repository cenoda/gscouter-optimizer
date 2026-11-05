mod artifact;
mod discovery;
mod evaluate;
mod program;
mod search;

fn main() {
    if let Err(e) = search::cli() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
