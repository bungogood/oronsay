use clap::Parser;
use crossbeam::channel;
use memmap2::MmapOptions;
use num_format::{Locale, ToFormattedString};
use oronsay::{ChunkStats, Reader, SolverBasic, Worker, Writer};
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{io, thread};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file
    #[clap(short, long)]
    infile: PathBuf,

    /// Output file
    #[clap(short, long)]
    outfile: Option<PathBuf>,

    /// Number of worker threads
    #[clap(short = 't', long = "threads")]
    num_threads: Option<usize>,

    /// Chunk size in kB
    #[clap(short, long, default_value_t = 16)]
    chunk_size: usize,

    /// No hash
    #[clap(short, long)]
    no_hash: bool,

    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
}

fn get_num_threads() -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn display_stats(
    stats: &ChunkStats,
    hash: Option<String>,
    elapsed: Duration,
    num_threads: usize,
) -> io::Result<()> {
    let real_rate = stats.puzzles as f64 / elapsed.as_secs_f64();
    let real_avg_time = elapsed / stats.puzzles as u32;

    let solver_rate = stats.puzzles as f64 / stats.elapsed.as_secs_f64();
    let solver_avg_time = stats.elapsed / stats.puzzles as u32;

    let guess_rate = stats.guesses as f32 / stats.puzzles as f32;
    let no_guess_percent = (stats.no_guesses as f32 / stats.puzzles as f32) * 100.0;

    println!(
        "   # Puzzles: {}, No Guesses: {:.2}%, Avg Guesses: {:.2}",
        stats.puzzles.to_formatted_string(&Locale::en),
        no_guess_percent,
        guess_rate
    );
    println!(
        "   Real Time: {:.2?}, Rate: {}/s, Avg: {:.2?}, # Chunks: {}",
        elapsed,
        (real_rate as u32).to_formatted_string(&Locale::en),
        real_avg_time,
        stats.chunks.to_formatted_string(&Locale::en)
    );
    println!(
        " Solver Time: {:.2?}, Rate: {}/s, Avg: {:.2?}, # Threads: {}",
        stats.elapsed,
        (solver_rate as u32).to_formatted_string(&Locale::en),
        solver_avg_time,
        num_threads
    );

    match hash {
        Some(ref h) => println!("SHA-256 Hash: {}", h),
        None => println!("SHA-256 Hash: Not computed"),
    };

    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let num_workers = args.num_threads.unwrap_or(get_num_threads());
    let chunk_size = args.chunk_size * 1024;

    let input_file = File::open(&args.infile)?;
    let mmap = unsafe { MmapOptions::new().map(&input_file)? };
    let mmap = Arc::new(mmap);

    let (chunk_tx, chunk_rx) = channel::unbounded();
    let (output_tx, output_rx) = channel::unbounded();

    let start = std::time::Instant::now();

    let solver = SolverBasic::new(1, true);

    let (line_length, reader_handle) =
        Reader::spawn(Arc::clone(&mmap), chunk_tx, output_tx.clone(), chunk_size);
    let worker_handles =
        Worker::spawn_multiple(solver, line_length, chunk_rx, output_tx, num_workers);
    let writer_handle = Writer::spawn(output_rx, args.outfile, args.no_hash, args.verbose);

    reader_handle.join().expect("Reader panicked");
    for handle in worker_handles {
        handle.join().expect("Worker panicked");
    }
    let (hash, stats) = writer_handle.join().expect("Writer panicked");

    display_stats(&stats, hash, start.elapsed(), num_workers)
}
