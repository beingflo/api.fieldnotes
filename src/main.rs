mod env;

use env::Config;
use log::info;

fn main() {
    pretty_env_logger::init();
    info!("Textli started");

    let config = Config::get();

    println!("{:?}", config);
}
