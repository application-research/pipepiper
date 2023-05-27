use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiperMode {
    Server,
    Client,
}

argwerk::define! {
    #[usage = "ppr [-h] [recv | send <ip> <port>]"]
    pub struct Args {
        pub help: bool = false,
        #[required = "run mode"]
        pub mode: PiperMode,
        #[required = "run mode"]
        pub addr: SocketAddr,
        pub config: Option<String>,
    }
    /// Print the help text.
    ["-h" | "--help"] => {
        help = true;
    }
    /// Run piper in client mode.
    ["recv", ip, port] => {
        mode = Some(PiperMode::Server);
        addr = Some(format!("{ip}:{port}").parse::<SocketAddr>()?);
    }
    /// Run piper in server mode.
    ["send", ip, port] => {
        mode = Some(PiperMode::Client);
        addr = Some(format!("{ip}:{port}").parse::<SocketAddr>()?);
    }
    /// Configuration file.
    ["--config", file] => {
        config = Some(file);
    }
}

pub fn parse_args() -> Args {
    match Args::args() {
        Ok(args) => {
            if args.help {
                std::process::exit(1);
            }
            args
        }
        Err(why) => {
            eprintln!("{}", Args::help());
            eprintln!("{why:#}");
            std::process::exit(1);
        }
    }
}
