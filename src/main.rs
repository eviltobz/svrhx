//use directories::{ProjectDirs};
//use confy::*;
use std::iter::Extend;
use colored::*;
use structopt::StructOpt;
use std::path::PathBuf;
use std::path::Path;
use serde::{Serialize, Deserialize};
// use notify::*; //{Watcher, watcher, RecursiveMode};
// use std::sync::mpsc::channel;
// use std::time::Duration;
// use hotwatch::*;
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};

#[derive(StructOpt)]
#[derive(Debug)]
enum Command { 
    #[structopt(about = "Add to the list of tracked files to copy")]
    Add { 
        #[structopt(parse(from_os_str), about = "Files to track")]
        files: Vec<PathBuf>
    },
    #[structopt(about = "Initialise a session based on the current directory")]
    Init { 
        #[structopt(about = "Remote location to copy files to")]
        path: PathBuf
    },
    #[structopt(about = "Watch for changing files and copy them")]
    Watch
}

#[derive(StructOpt)]
#[derive(Debug)]
/// SerVeRHaXx
/// 
/// Automatically copies files to a remote location when they change
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
struct ValidatedConfig {
    project_root: PathBuf,
    files: Vec<PathBuf>,
    destination: PathBuf
}

fn main() {
    let _ = control::set_virtual_terminal(true);
    let opt = Opt::from_args();
    // println!("command line args: {:?}", opt);
    let mut config = get_config();
    // println!("Loaded Config into main: {:?}", config);
    // display_config(&config);

    match opt.cmd {
        Some(Command::Add{files}) => add(&mut config, files),
        Some(Command::Init{path}) => init(&mut config, path),
        Some(Command::Watch) => try_watch(config),

        None => display_config(&config) // println!("{}", "No command supplied".purple())
    }
}

fn validate_config(config: Config) -> Option<ValidatedConfig>{
    let current = std::env::current_dir().unwrap();
    let expected = config.project_root.unwrap();
    if expected != current {
        println!("{}", "Error: SVRHX is running in the wrong location.".red());
        println!("{} {} {} {}", "Expected:".red(), expected.to_string_lossy().yellow(), "Actual:".red(), &current.to_string_lossy().yellow());
        println!("to be honest, i could just cd there myself...");
        return None;
    }

    let destination = config.destination.unwrap();

    if config.files.len() == 0 {
        println!("{}", "Error: No files configured to track.".red());
        return None;
    }

    return Some(ValidatedConfig{project_root: current, destination: destination, files: config.files});

}

fn try_watch(raw_config: Config) {
    let config = validate_config(raw_config);
    
    match config {
        Some(valid) => watch(valid).unwrap(),
        None => {}
    }
}

fn watch(config: ValidatedConfig) -> Result<(), hotwatch::Error> {
    println!("{} {} {}", "Watching".green(), config.project_root.to_string_lossy().yellow(), "for changes to:".green());
    display_tracked_files(&config.files);

    let mut watcher = Hotwatch::new()?; //.expect("hotwatch failed to initialise");


    // let (sender, receiver) = channel();
    // let mut watcher = watcher(sender, Duration::from_secs(2)).unwrap();
    for file in config.files {
        let x = file.clone();
        let mut dest_path = config.destination.clone();
        let dest_root = config.destination.clone();
        dest_path.push(&x);

        watcher.watch(file, move |event: Event| {

            match event {
                Event::Write(path) => {
                    println!("Write event for: {}, registered at {}", path.to_string_lossy(), x.to_string_lossy());
                    println!("So I wanna copy from {} to {}", path.to_string_lossy(), dest_path.to_string_lossy());
                    // let bits = x.components();
                    // for bit in bits {
                    //     println!(" - {:?}", bit);
                    // }
                    // let os_bits = x.iter().collect::<Vec<std::ffi::OsStr>>();
                    let os_bits = x.iter();
                    let mut partial = dest_root.clone();

                    for bit in os_bits {
                        partial.push(bit);
                        println!(" bit:{:?}, partial:{}", bit, partial.to_string_lossy());
                    }

                    // println!("Ancestors:");
                    // for old_folk in x.ancestors() {
                    //     println!(" old_folk:{:?}", old_folk);
                    // }

                    println!("Ancestors.Skip:");
                    let to_check = x.ancestors().skip(1).collect::<Vec<_>>(); //.rev();
                    println!("Got {} ancestor steps.", to_check.len());
                    for old_folk in to_check {
                        println!(" old_folk:{:?}", old_folk);
                    }
                    ensure_path(&dest_root, &x.ancestors().skip(1).next().unwrap());

                    let copy_result = std::fs::copy(&path, &dest_path); //.expect("Could not copy");
                    println!("Copy result for '{}' to '{}': {:?}", &path.to_string_lossy(), &dest_path.to_string_lossy(), copy_result);

                    println!("(might have) Copied :{} at {}", x.to_string_lossy().yellow(), "(get a datetime stamp)");
                    panic!("?");
                },
                _ => println!("Unhandled event: {:?}", event)
            }
            Flow::Continue
        }).expect("failed to watch file");
    }
    // watcher.watch(&config.files[0], RecursiveMode::NonRecursive).unwrap();
    // loop {
    //     print!(".");
    //     match receiver.recv() {
    //         Ok(event) => handle(event, &config.destination),
    //         Err(e) => println!("Err:{:?}", e),
    //     }
    // }

    watcher.run();
    println!("Exiting...");
    Ok(())
}

fn ensure_path(root: &PathBuf, extra: &Path) -> bool {
    let full = root.clone().join(extra);
    println!("ensuring {}", full.to_string_lossy());
    if full.exists() {
        println!("{}", "w00t.".green());
        return true;
    }
    std::fs::create_dir_all(full).expect("could not create path");
    true
    /*
    let next = extra.ancestors().skip(1).next();
    match next {
        Some(path) => {
            if ensure_path(&root, &path) {
                // mkdir
                true
            } else {
                false
            }
        },
        _ => false
    }
    */
}

// fn handle(event: DebouncedEvent, dest: &PathBuf) {
//     println!("Event:{:?}", event);
//     //let dest2 = dest.join()
//     match event {
//         DebouncedEvent::Write(path) => {
//             println!("{} {}", "I need to copy the file:".green(), path.to_string_lossy().yellow());
//             println!("{} {}", "To:".green(), path.to_string_lossy().yellow());
//         },
//         _ => println!("{} {:?}", "Not currently handling event type:".red(), event)
//     }
// }

fn add(config: &mut Config, files: Vec<PathBuf>) {
    // Error if adding when in a different folder to the project root!
    println!("{} {:?}", "Adding".green(), files);
    config.files.extend(files);

    save_config(&config);
}

fn init(config: &mut Config, dest: PathBuf) {
    // Using unwrap is bad. At some point I should change this to be more
    // idiomatic rust code...
    let current = std::env::current_dir().unwrap();
    println!("{} {} {} {}", 
        "Initialise in".green(), 
        current.to_string_lossy().cyan(), 
        "and point to".green(), 
        dest.to_string_lossy().cyan());

    config.destination = Some(dest);
    config.project_root = Some(current);
    config.files = Vec::new();

    save_config(&config);
}

// Confy has its own ideas about where stuff should be stored, using directories to build
// the path with company name & whatnot makes it go pop. So we'll just use its defaults 
// if it seems to be sufficient
/*
fn get_config_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "vvec", "") {
        println!("  {:?}", proj_dirs);
        // return  Some(proj_dirs.config_dir().join("svrhx"));
        return  Some(proj_dirs.config_dir().to_path_buf());
    }
    return None;
}
*/


const CONFIG_FILE_NAME: &str = "svrhx";
fn get_config() -> Config {
        let loaded: std::result::Result<Config, confy::ConfyError> = confy::load(CONFIG_FILE_NAME);
        let cfg = loaded.ok();
        return cfg.unwrap();
}

fn save_config(cfg: &Config) {
    let _ = confy::store(CONFIG_FILE_NAME, &cfg);

    println!("{}", "Updated config!".purple());
    display_config(&cfg);
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
        display_tracked_files(&cfg.files);
    }
}

fn display_tracked_files(files: &Vec<PathBuf>) {
    for file in files.iter(){
        println!("    {}", file.to_string_lossy().yellow());
    }
}


