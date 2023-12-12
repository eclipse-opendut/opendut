use opendut_util::logging;

const BANNER: &str = r"
                         _____     _______
                        |  __ \   |__   __|
   ___  _ __   ___ _ __ | |  | |_   _| |
  / _ \| '_ \ / _ \ '_ \| |  | | | | | |
 | (_) | |_) |  __/ | | | |__| | |_| | |
  \___/| .__/ \___|_| |_|_____/ \__,_|_|
       | |   _____          _____  _
       |_|  / ____|   /\   |  __ \| |
           | |       /  \  | |__) | |
           | |      / /\ \ |  _  /| |
           | |____ / ____ \| | \ \| |____
            \_____/_/    \_\_|  \_\______|

              - He Fixes the Cable -";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", opendut_carl::app_info::formatted_with_banner(BANNER));

    logging::initialize()?;

    opendut_carl::create(opendut_util::settings::Config::default()).await
}
