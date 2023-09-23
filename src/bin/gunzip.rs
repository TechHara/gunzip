use gunzip::error::Result;
use gunzip::gunzip;

fn usage(program: &str) {
    eprintln!("Usage: {}", program);
    eprintln!("\tDecompresses .gz file read from stdin and outputs to stdout");
    eprintln!("Example: {} < input.gz > output", program);
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let program = args.next().unwrap();
    if args.next().is_some() {
        usage(&program);
        std::process::exit(-1);
    }

    let reader = std::io::stdin();
    let writer = std::io::stdout();
    gunzip(reader, writer)
}
