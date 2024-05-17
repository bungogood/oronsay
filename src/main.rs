use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read},
    path::PathBuf,
    thread,
    time::Duration,
};

use clap::Parser;
use crossbeam_channel;
use num_format::{Locale, ToFormattedString};
use tempfile::NamedTempFile;

use orsay::{reader, solver::Stats, worker, writer};

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

    let (line_length, reader_thread) = reader::start_reader(buf_reader, read_tx, write_tx.clone())?;
    let worker_threads = worker::start_workers(num_workers, line_length, read_rx, write_tx);
    let writer_thread = writer::start_writer(buf_writer, write_rx);

    let mut stats = Stats::new();
    reader_thread.join().expect("Error joining reader thread");
    for worker in worker_threads {
        stats.add(&worker.join().expect("Error joining worker thread"));
    }
    writer_thread.join().expect("Error joining writer thread");

    if args.verbose {
        let outpath = match temp_file.as_ref() {
            Some(temp_file) => temp_file.path().to_path_buf(),
            None => args.outfile.unwrap(),
        };
        display_stats(&stats, start.elapsed(), &outpath)?;
    }

    Ok(())
}

fn display_stats(stats: &Stats, duration: Duration, outpath: &PathBuf) -> io::Result<()> {
    let sudokus_rate = stats.puzzles as f64 / duration.as_secs_f64();
    let avg_time = duration / stats.puzzles as u32;
    let guess_rate = stats.guesses as f32 / stats.puzzles as f32;
    let no_guess_percent = (stats.no_guesses as f32 / stats.puzzles as f32) * 100.0;

    println!(
        "Number of puzzles: {}",
        stats.puzzles.to_formatted_string(&Locale::en)
    );
    println!(
        "Time Taken: {:.2?}, Solve Rate: {}/s, Avg: {:.2?}",
        duration,
        (sudokus_rate as u32).to_formatted_string(&Locale::en),
        avg_time
    );
    println!(
        "No Guesses: {:.2}%, Avg Guesses: {:.2}",
        no_guess_percent, guess_rate
    );

    let out_bytes = fs::read(outpath)?;
    let sha256sum = crypto_hash::hex_digest(crypto_hash::Algorithm::SHA256, &out_bytes);
    // let md5sum = crypto_hash::hex_digest(crypto_hash::Algorithm::MD5, &out_bytes);

    println!("SHA-256 Hash: {}", sha256sum);
    // println!("MD5 Hash:     {}", md5sum);

    Ok(())
}
