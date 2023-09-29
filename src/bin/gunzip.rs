use gunzip::error::Result;
use gunzip::gunzip;

fn usage(program: &str) {
    eprintln!("Usage: {} [-t]", program);
    eprintln!("\tDecompresses .gz file read from stdin and outputs to stdout");
    eprintln!("\t-t: employ two threads");
    eprintln!("Example: {} < input.gz > output", program);
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let program = args.next().unwrap();
    let multithread = match args.next() {
        Some(arg) => {
            if arg == "-t" {
                true
            } else {
                usage(&program);
                std::process::exit(-1);
            }
        }
        None => false,
    };

    let reader = std::io::stdin();
    let writer = std::io::stdout();
    gunzip(reader, writer, multithread)
}
