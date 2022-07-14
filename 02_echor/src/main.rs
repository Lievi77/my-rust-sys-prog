// fn main() {
//     //prints the arguments given
//     println!("{:?}", std::env::args()); //only works with {:?} that formats debugging context

//     //putting a flag before the arguments will not work
//     //since cargo thinks its an argument for itself
//     //to avoid this error, separate the arguments with -- :
//     // ! cargo run -- -n hello world
// }

//to parse arguments, we will use clap
use clap::{Arg, Command};
fn main() {
    //a underscore preceding a variable name has a purpose
    // it tells the rust compliler that we do not intend to use this variable soon
    let matches = Command::new("echor")
        .version("0.1.0")
        .author("Lev")
        .about("Rust echo")
        .arg(
            Arg::new("text")
                .value_name("TEXT")
                .help("Input text")
                .required(true)
                .min_values(1)
                .allow_invalid_utf8(true),
        )
        .arg(
            Arg::new("omit_newline")
                .help("Do not print newline")
                .takes_value(false)
                .short('n'),
        )
        .get_matches();

    //use formatter {:#?} to print newlines and indentations
    // called pretty-printing because well.. it's pretty
    //println!("{:#?}", matches);

    //the stdout and stderr are symbolized by 1,2 respectively
    //cargo run 1>out 2>stderror
    //a good system program will print error to stderr and correct ouput to stdout

    //extract the arguments captured by clap
    let text: Vec<String> = matches.values_of_lossy("text").unwrap();
    //Warning!!!
    // If you call Option::unwrap on a None, it will cause a panic that will
    //crash your program. You should only call unwrap if you are positive the
    //value is the Some variant.

    //omit_newline option
    let omit_newline: bool = matches.is_present("omit_newline"); //note that this option does not take any value.

    //Rustacean way to write a simple if expression
    //let ending = if omit_newline { " " } else { "\n" };
    //if is an expression, not a statement
    //an expression retuns a value, a statetment does not.
    //an if statement without an else clause will return the unit ()
    print!(
        "{}{}",
        text.join(" "), //we can join vectors of strings
        if omit_newline { " " } else { "\n" }
    );
}
