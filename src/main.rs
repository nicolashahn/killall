use nix::{
    sys::signal::{kill, SIGKILL},
    unistd::{getpid, Pid},
};
use std::{collections::BTreeSet, env, error::Error, fs, io::Read};

fn main() -> Result<(), Box<dyn Error>> {
    let mut filenames = env::args().collect::<Vec<_>>();
    filenames.remove(0);
    if filenames.is_empty() {
        return Err("you must pass at least one filename".into());
    }

    let proc = "/proc/";
    let mut pgroups = BTreeSet::new();

    for full_path in fs::read_dir(proc)?.map(|res| res.map(|e| e.path())) {
        let fp_str = full_path?.into_os_string().into_string().unwrap();

        let rel_path = fp_str.split("/").collect::<Vec<_>>()[2];
        if rel_path.chars().all(char::is_numeric) {
            let get_pid_file_contents = |filename: &str| -> Result<String, Box<dyn Error>> {
                let mut path = fp_str.clone();
                path.push_str(filename);
                let mut contents = String::new();
                fs::File::open(path)?.read_to_string(&mut contents)?;

                Ok(contents)
            };

            let stat = get_pid_file_contents("/stat")?;
            let cmdline = get_pid_file_contents("/cmdline")?;

            let neg_pgroup = Pid::from_raw(-stat.split(" ").collect::<Vec<_>>()[4].parse::<i32>()?);

            for filename in &filenames {
                if cmdline.contains(filename) {
                    let pid = Pid::from_raw(rel_path.to_string().parse::<i32>()?);
                    if getpid() != pid {
                        pgroups.insert(neg_pgroup);
                    }
                }
            }
        }
    }

    for pgroup in pgroups {
        println!("Killing pgroup: {:?}", pgroup.as_raw());
        kill(pgroup, Some(SIGKILL)).unwrap();
    }

    Ok(())
}
