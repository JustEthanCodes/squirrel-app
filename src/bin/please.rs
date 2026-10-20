use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("help")
        || args.next().as_deref() != Some("me")
        || args.next().is_some()
    {
        eprintln!("Usage: please help me");
        return ExitCode::from(2);
    }

    println!("Telling bradley to back off...");
    let status = match Command::new("taskkill")
        .args(["/IM", "bradley.exe", "/F"])
        .status()
    {
        Ok(status) => status,
        Err(error) => {
            eprintln!("Sorry. He regected. His reason is {error}");
            return ExitCode::FAILURE;
        }
    };

    if status.success() {
        println!("Ok, he is gone.");
        ExitCode::SUCCESS
    } else {
        eprintln!("Could not stop bradley. (taskkill exited with {status}).");
        ExitCode::FAILURE
    }
}
