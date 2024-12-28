use gunzip::{MultithreadDecompressor, SingleThreadDecompressor};
use std::io;

fn usage(program: &str) {
    eprintln!("Usage: {} [-t]", program);
    eprintln!("\tDecompresses .gz file read from stdin and outputs to stdout");
    eprintln!("\t-t: employ two threads");
    eprintln!("Example: {} < input.gz > output", program);
}

fn main() -> io::Result<()> {
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

    let reader = io::stdin();
    let mut writer = io::BufWriter::with_capacity(32768, get_stdout()?);

    if multithread {
        let mut decompressor = MultithreadDecompressor::new(reader);
        io::copy(&mut decompressor, &mut writer)?;
    } else {
        let mut decompressor = SingleThreadDecompressor::new(reader);
        io::copy(&mut decompressor, &mut writer)?;
    }

    Ok(())
}

/// Stdout by default is implemented by LineWriter, adding unnecessary overhead for binary data.
/// This is a quick and dirty solution to obtain raw stdout that works for *nix.
#[cfg(unix)]
fn get_stdout() -> io::Result<impl io::Write> {
    std::fs::File::create("/dev/stdout")
}

#[cfg(windows)]
fn get_stdout() -> io::Result<impl io::Write> {
    Ok(io::stdout())
}
