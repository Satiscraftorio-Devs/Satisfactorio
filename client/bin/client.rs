use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from(network::DEFAULT_SERVER_ADDRESS))]
    address: String,
    #[arg(short, long, default_value_t = String::from(""))]
    username: String,
}

fn main() {
    let args = Args::parse();
    client::run_client(&args.address);
}
