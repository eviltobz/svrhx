use directories::{ProjectDirs};
// This works with ConEmu & the VS Code build output.
// It DOESN'T work with consolez, default powershell, or the VS Code built in terminal
// So methinks I'll need to do a windows specific version, with changing the terminal fg colour around.
use colored::*;
use structopt::StructOpt;
use std::path::PathBuf;
use std::path::Path;
//use confy::*;
use serde::{Serialize, Deserialize};
//use serde_derive::{Serialize, Deserialize};

#[derive(StructOpt)]
#[derive(Debug)]
#[structopt(about = "SerVeRHaXx")]
enum Command { 
    // None,
    Add { 
        #[structopt(parse(from_os_str))]
        files: Vec<PathBuf>
    },
    Init { 
        path: PathBuf
    },
    Watch
}

#[derive(StructOpt)]
#[derive(Debug)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Option<Command>
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    project_root: Option<PathBuf>,
    files: Vec<PathBuf>,
    destination: Option<PathBuf>
}
impl :: std::default::Default for Config {
    fn default() -> Self { Self { files: Vec::new() , destination: None, project_root: None } }
}

fn main() {
    let _ = control::set_virtual_terminal(true);
    println!("{}", "Meh".green());
    let opt = Opt::from_args();
    println!("opt: {:?}", opt);
    let config = get_config();
    println!("Loaded Config into main: {:?}", config);
    display_config(&config);

    match opt.cmd {
        Some(Command::Add{files}) => println!("{} {:?}", "Adding".green(), files),
        Some(Command::Init{path}) => println!("{} {} ", "Initialise and point to ".green(), path.to_string_lossy().cyan()),
        Some(Command::Watch) => println!("{} ", "Watching for file changes".green()),

        None => println!("{}", "No command supplied".purple())
    }



    save_config(&config);
}

// Confy has its own ideas about where stuff should be stored, using directories to build
// the path with company name & whatnot makes it go pop. So we'll just use its defaults 
// if it seems to be sufficient
fn get_config_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "vvec", "") {
        println!("  {:?}", proj_dirs);
        // return  Some(proj_dirs.config_dir().join("svrhx"));
        return  Some(proj_dirs.config_dir().to_path_buf());
    }
    return None;
}


const CONFIG_FILE_NAME: &str = "svrhx";
fn get_config() -> Config {
        let loaded: std::result::Result<Config, confy::ConfyError> = confy::load(CONFIG_FILE_NAME);
        let cfg = loaded.ok();
        // println!("Loaded Config: {:?}", cfg);
        // confy::store(CONFIG_FILE_NAME, &cfg);
        return cfg.unwrap();
}

fn save_config(cfg: &Config) {
    confy::store(CONFIG_FILE_NAME, &cfg);
}

fn display_config(cfg: &Config) {
    println!("Current Config:");
    if let Some(root) = &cfg.project_root {
        println!("  Project Root Location: {}", root.to_string_lossy().yellow());
    } else {
        println!("  Project Root Location: {}", "Not Configured".red());
    }
    if let Some(dest) = &cfg.destination {
        println!("  Server Destination: {}", dest.to_string_lossy().yellow());
    } else {
        println!("  Server Destination: {}", "Not Configured".red());
    }
    if cfg.files.len() == 0 {
        println!("  Tracked Files: {}", "Not Configured".red());
    } else {
        println!("  Tracked Files:");
        for file in cfg.files.iter(){
            println!("    {}", file.to_string_lossy().yellow());
        }
    }

}