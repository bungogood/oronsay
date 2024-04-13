use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read},
    path::PathBuf,
    thread,
};

use clap::Parser;
use crossbeam_channel;
use num_format::{Locale, ToFormattedString};
use tempfile::NamedTempFile;

use orsay::{reader, worker, writer};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    infile: Option<PathBuf>,

    #[clap(short, long)]
    outfile: Option<PathBuf>,

    #[clap(short = 't', long = "threads")]
    num_threads: Option<usize>,

    #[clap(short, long)]
    verbose: bool,
}

fn get_num_threads() -> usize {
    thread::available_parallelism()
        .map(|num| num.get())
        .unwrap_or(1)
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let start = std::time::Instant::now();

    let num_workers = args.num_threads.unwrap_or(get_num_threads());

    let (read_tx, read_rx) = crossbeam_channel::unbounded();
    let (write_tx, write_rx) = crossbeam_channel::unbounded();

    let input: Box<dyn Read + Send> = match args.infile {
        Some(ref path) => Box::new(File::open(path)?),
        None => Box::new(io::stdin()),
    };
    let (outfile, temp_file) = match args.outfile {
        Some(ref path) => (File::create(path)?, None),
        None => {
            let temp_file = NamedTempFile::new()?;
            (temp_file.reopen()?, Some(temp_file))
        }
    };

    let buf_reader = BufReader::new(input);
    let buf_writer = BufWriter::new(outfile);

    let reader_thread = reader::start_reader(buf_reader, read_tx, write_tx.clone());
    let worker_threads = worker::start_workers(num_workers, read_rx, write_tx);
    let writer_thread = writer::start_writer(buf_writer, write_rx);

    let mut num_puzzles = 0;
    reader_thread.join().expect("Error joining reader thread");
    for worker in worker_threads {
        num_puzzles += worker.join().expect("Error joining worker thread");
    }
    writer_thread.join().expect("Error joining writer thread");

    if args.verbose {
        let time_taken = start.elapsed();
        let outpath = match temp_file.as_ref() {
            Some(temp_file) => temp_file.path().to_path_buf(),
            None => args.outfile.unwrap(),
        };
        let out_bytes = std::fs::read(outpath)?;

        let sudokus_rate = num_puzzles as f64 / time_taken.as_secs_f64();
        let avg_time = time_taken / num_puzzles as u32;
        // let guess_rate = guesses as f32 / num_puzzles as f32;

        let sha256sum = crypto_hash::hex_digest(crypto_hash::Algorithm::SHA256, &out_bytes);
        // let md5sum = crypto_hash::hex_digest(crypto_hash::Algorithm::MD5, &out_bytes);

        println!("Number of puzzles: {}", num_puzzles);
        println!(
            "Time Taken: {:.2?}, Speed: {}/s, Avg: {:.2?}",
            time_taken,
            (sudokus_rate as u32).to_formatted_string(&Locale::en),
            avg_time
        );
        println!("SHA-256 Hash: {}", sha256sum);
        // println!("MD5 Hash:     {}", md5sum);
    }

    Ok(())
}
