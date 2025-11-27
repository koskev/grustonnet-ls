use std::collections::VecDeque;


/// Go seems to scan the stack an will panic upon encountering a 0x1 pointer.
/// However, Rust does use this value in some cases
/// Just setting the variable is not enough. Therefore we'll set the environment
/// variable and restart the current program
/// If this turns out to be a problem we'll need to switch to an ipc based solution
/// If the variable is already set this function does nothing
pub fn restart_with_fixed_env() {

    if std::env::var("GODEBUG").is_err() {
        // At this point we are single threaded. Therefore this is safe
        unsafe {
            std::env::set_var("GODEBUG", "invalidptr=0,cgocheck=0");
        }

        let exe = std::env::current_exe().expect("Could not get path to the current executable");

        // On Unix we can just use execvp and replace the current process
        #[cfg(unix)]
        {
            let args: VecDeque<String> = std::env::args().collect();
            let err = exec::execvp(&exe, &args);
            eprintln!("Failed to restart with GODEBUG: {}", err);
            std::process::exit(1);
        }
        // Windows does not support essential features and therefore we just spawn a child process
        // and pass over stdin. This results in more memory usage, but that is the life on Windows
        #[cfg(not(unix))]
        {
            let mut args: VecDeque<String> = std::env::args().collect();
            // Pop first argument = executable
            args.pop_front();

            std::process::Command::new(exe)
                .args(args)
                .spawn()
                .expect("Could not spawn child process")
                .wait()
                .unwrap();
            std::process::exit(0);
        }
    }
}
