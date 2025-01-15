use clap::Parser;
#[cfg(feature = "client")]
use client::Client;
#[cfg(feature = "client")]
use error::*;

mod error;
mod reminder;
mod time;
#[cfg(feature = "client")]
mod client;
#[cfg(feature = "server")]
mod server;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Clone, Debug, PartialEq, clap::Subcommand)]
enum Subcommand {
    /// Launch a new server instance and listen for incoming connections
    #[cfg(feature = "server")]
    Serve {},

    /// Create a new reminder
    #[cfg(feature = "client")]
    Create {
        /// Date and time for your reminder
        #[clap(value_parser = time::parse)]
        time: chrono::DateTime<chrono::Local>,

        /// Message to add to the reminder
        message: String,
    },

    /// List all reminders
    #[cfg(feature = "client")]
    List {},

    /// Fetch all pending reminders and mark them as acknowledged
    #[cfg(feature = "client")]
    Fetch {},

    /// Listen for reminders and mark them as acknowledged
    #[cfg(feature = "client")]
    Listen {
        /// Waiting interval between fetches
        #[clap(short, long, default_value_t = 60)]
        interval: u64,

        /// Command to call on new reminders (fed through stdin)
        #[clap(short, long)]
        command: Option<String>,
    },
}

fn main() {
    let args = Args::parse();

    use Subcommand::*;
    let result = match args.subcommand {
        #[cfg(feature = "server")]
        Serve {  } => server::serve(),

        #[cfg(feature = "client")]
        Create { time, message } => Client::new().create(time, message),

        #[cfg(feature = "client")]
        List {} => Client::new().list(),

        #[cfg(feature = "client")]
        Listen { interval, command } => Client::new().listen(interval, command),

        #[cfg(feature = "client")]
        Fetch {} => Client::new().fetch(),
    };

    error::resolve(result);
}
