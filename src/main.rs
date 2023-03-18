use clap::{Parser, command, ArgGroup};
use std::path::PathBuf;
use std::process::{Command,Output,Stdio,Child};
use std::time::Instant;
use std::{io, fs};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author = "SliceOfArdath", version, about = "A benchmark tool.", long_about = None)]
#[command(group(ArgGroup::new("mode").required(true).args(["run", "file"])))]
struct Args {
    /// Commands to execute. If there is more than one command, they will be piped into one another. Spaces separate commands (outside of quotations marks).
    #[arg(required=true,value_name="COMMAND")]
    run: Option<Vec<String>>,
    /// File path. The given file should contain a command, or list of commands to chain. Each command or list of commands should only take one line. Commands on the same line are separated by a pipe "|".
    #[arg(short)]
    file: Option<PathBuf>,
    /// Iteration count. The number of times the command is ran.
    #[arg(short,default_value_t=10,value_name="N")]
    iter: u8,
    /// Toggle timing. If present, the execution time will be returned.
    #[arg(short,long="testosterone")]
    time: bool,
    /// Expected result. If blank, the result is returned.
    #[arg(short,long="estrogen")]
    expect: Option<String>,
    /// Disable time statistics. The raw times will instead be displayed.
    #[arg(long)]
    no_stats: bool,
    /// Warmup rounds count.
    #[arg(short, default_value_t=3,value_name="N")]
    warmup: u8
}

fn build(command: Vec<&str>) -> Command {
    let mut output = Command::new(command.get(0).expect("No command attached!"));

    for i in 1..command.len() {
        output.arg(command[i]);
    }
    return output;
}

//Call the first command in a call chain
fn begin(first: Vec<&str>) -> Child {
    return build(first).stdout(Stdio::piped()).spawn().expect("Failed command");
}
/// Links the first command's ouput to the second's input, then starts the second command.
fn link(first: Child, second: Vec<&str>) -> Child {
    //first.stdout(Stdio::piped());
    return build(second).stdin(first.stdout.unwrap()).stdout(Stdio::piped()).spawn().expect("Failed command");
}
///Finishes a call stack
fn finish(last: Child) -> Result<Output, io::Error> {
    return last.wait_with_output();
}

fn run_notime(iter: u8, warmup: u8, commands: Vec<Vec<&str>>, expected: Option<String>) {
    for _ in 0..warmup {
        let mut r = begin(commands.get(0).expect("You must have at least one command.").to_vec());
        for i in 1..commands.len() {
            r = link(r, commands.get(i).expect("Access Error").to_vec());
        }
    }
    for _ in 0..iter {
        let mut r = begin(commands.get(0).expect("You must have at least one command.").to_vec());
        for i in 1..commands.len() {
            r = link(r, commands.get(i).expect("Access Error").to_vec());
        }
        match expected {
            None => println!("Result: {:?}", String::from_utf8_lossy(&finish(r).unwrap().stdout)),
            Some(ref x) => assert_eq!(x, &String::from_utf8_lossy(&finish(r).unwrap().stdout)),
        }
    }
}
fn run_time(iter: u8, warmup: u8, nostats: bool, commands: Vec<Vec<&str>>, expected: Option<String>) {
    for _ in 0..warmup {
        let mut r = begin(commands.get(0).expect("You must have at least one command.").to_vec());
        for i in 1..commands.len() {
            r = link(r, commands.get(i).expect("Access Error").to_vec());
        }
        finish(r).unwrap();
    }
    let mut stats: Vec<f64> = Vec::new();
    for _ in 0..iter {
        let start = Instant::now();
        let mut r = begin(commands.get(0).expect("You must have at least one command.").to_vec());
        for i in 1..commands.len() {
            r = link(r, commands.get(i).expect("Access Error").to_vec());
        }
        match expected {
            None => println!("Result: {:?}", String::from_utf8_lossy(&finish(r).unwrap().stdout)),
            Some(ref x) => assert_eq!(x, &String::from_utf8_lossy(&finish(r).unwrap().stdout)),
        }
        let elapsed = start.elapsed();
        if nostats {
            println!("Raw Time: {}", elapsed.as_secs_f64());
        } else {
            stats.push(elapsed.as_secs_f64());
        }
    }
    if !nostats {
        println!("Time: {} (±{})", statistical::mean(&stats), statistical::variance(&stats, None))
    }
}

fn execute(args: (u8, u8, bool, bool, Option<String>), run_arg: Vec<String>) {
    let command: Vec<Vec<&str>> = run_arg.iter().map(|s| s.split("\"").collect()).collect();
    //todo! improve split
    println!("Commands: {:?}", command);
    if args.2 {
        run_time(args.0, args.1, args.3, command, args.4);
    } else {
        run_notime(args.0, args.1, command, args.4);
    }  
}

fn main() {
    let args = Args::parse();
    match args.run {
        Some(r) => {
            let run = r;
            execute((args.iter, args.warmup, args.time, args.no_stats, args.expect), run.to_vec());
        }
        None => {
            let contents = fs::read_to_string(&args.file.unwrap()).expect("Failed reading the file.");
            let command_list: Vec<&str> = contents.split("\n").collect();
            for command in command_list {
                let to_run = command.split("\"|\"").map(|s| String::from(s)).collect();
                execute((args.iter, args.warmup, args.time, args.no_stats, args.expect.clone()), to_run);
            }
        }
    }
}