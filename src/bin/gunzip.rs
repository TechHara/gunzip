use gunzip::error::Result;
use gunzip::Decompressor;

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
    let mut writer = std::io::stdout();

    let mut decompressor = Decompressor::new(reader, multithread);
    std::io::copy(&mut decompressor, &mut writer)?;
    Ok(())
}
