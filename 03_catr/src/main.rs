//main driver code binary
fn main() {
    //execute catr::run()
    //if it returns an Err(e), execute callback

    //use and_then to pass the Ok(Config) config to catr::run
    if let Err(e) = catr::get_args().and_then(catr::run) {
        eprintln!("{}", e); //prints to stderr
        std::process::exit(1);
    }
}
