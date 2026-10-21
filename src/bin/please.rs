use std::process::ExitCode;

#[cfg(any(windows, target_os = "linux", target_os = "macos"))]
use std::process::Command;

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

    #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
    {
        #[cfg(windows)]
        let result = Command::new("taskkill")
            .args(["/IM", "bradley.exe", "/F"])
            .status();
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let result = Command::new("pkill").args(["-x", "bradley"]).status();

        let status = match result {
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
            eprintln!("Could not stop bradley (process command exited with {status}).");
            ExitCode::FAILURE
        }
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        eprintln!("The please helper supports Windows, Linux, and macOS only.");
        ExitCode::FAILURE
    }
}
