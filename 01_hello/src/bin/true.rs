//Rust implementation of true UNIX utility

fn main() {
    std::process::exit(0); //the true utility always returns 0 (exit with no errors)
                           //the variable $? indicates the exit result of the previous command

    //after executing true, echo $? will output 0

    //Rust programs exit with 0 code as default
}
