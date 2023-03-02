use clap::{Parser, command};
use std::process::{Command,Output,Stdio,Child};
use std::time::Instant;
use std::{time, io};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author = "SliceOfArdath", version, about = "A benchmark tool.", long_about = None)]
//#[command(group(ArgGroup::new("mode").required(true).args(["run", "file"])))]
struct Args {
    /// Commands to execute. If there is more than one command, they will be piped into one another.
    #[arg(required=true,value_name="COMMAND")]
    run: Vec<String>,
    // File path. The given file should contain a command, or list of commands to chain. Each command should only take one line.
    //#[arg(short)]
    //file: Option<String>,
    /// Iteration count. The number of times the command is ran.
    #[arg(short,default_value_t=10,value_name="N")]
    iter: u8,
    /// Toggle timing. If present, the execution time will be returned.
    #[arg(short,long="testosterone")]
    time: bool,
    /// Expected result. If blank, the result is returned.
    #[arg(short,long="estrogen")]
    expect: Option<String>,
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

fn run_notime(iter: u8, commands: Vec<Vec<&str>>) {
    for _ in 0..iter {
        let mut r = begin(commands.get(0).expect("You must have at least one command.").to_vec());
        for i in 1..commands.len() {
            r = link(r, commands.get(i).expect("Access Error").to_vec());
        }
        println!("Result: {:?}", finish(r));
    }
}
fn run_time(iter: u8, commands: Vec<Vec<&str>>) {
    for _ in 0..iter {
        let start = Instant::now();
        let mut r = begin(commands.get(0).expect("You must have at least one command.").to_vec());
        for i in 1..commands.len() {
            r = link(r, commands.get(i).expect("Access Error").to_vec());
        }
        println!("Result: {:?}", finish(r));
        let elapsed = start.elapsed();
        println!("{}", elapsed.as_secs_f64());
    }
}

fn main() {
    let args = Args::parse();
    let run = args.run;
    let command: Vec<Vec<&str>> = run.iter().map(|s| s.split(" ").collect()).collect();
    //todo! improve split
    println!("Commands: {:?}", command);
    if args.time {
        run_time(args.iter, command);
    } else {
        run_notime(args.iter, command);
    }
}