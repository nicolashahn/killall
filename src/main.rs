use clap::{App, Arg};
use nix::{
    sys::signal::{kill, SIGKILL},
    unistd::{getpid, Pid},
};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
};

fn main() -> io::Result<()> {
    let matches = App::new("killall")
        .arg(
            Arg::with_name("filenames")
                .help("killall processes in the same group as the filename for each given")
                .multiple(true)
                .required(true),
        )
        .get_matches();

    let filenames = matches.values_of("filenames").unwrap().collect::<Vec<_>>();
    let proc = "/proc/";
    let mut pgroups = BTreeSet::new();

    for full_path in fs::read_dir(proc)?
        .map(|res| res.map(|e| e.path()))
        .collect::<io::Result<Vec<_>>>()?
    {
        full_path
            .into_os_string()
            .into_string()
            .map(|fp| {
                let rel_path = fp.split("/").collect::<Vec<_>>()[2];
                if rel_path.chars().all(char::is_numeric) {
                    let get_file_contents = |filename: &str| {
                        let mut path = fp.to_owned();
                        path.push_str(filename);
                        let mut file = fs::File::open(path).unwrap();
                        let mut contents = String::new();
                        file.read_to_string(&mut contents).unwrap();
                        contents
                    };

                    let stat = get_file_contents("/stat");
                    let cmdline = get_file_contents("/cmdline");

                    let neg_pgroup = Pid::from_raw(
                        -stat.split(" ").collect::<Vec<_>>()[4]
                            .parse::<i32>()
                            .unwrap(),
                    );

                    for filename in &filenames {
                        if cmdline.contains(filename) {
                            let pid = Pid::from_raw(rel_path.to_string().parse::<i32>().unwrap());
                            if getpid() != pid {
                                pgroups.insert(neg_pgroup);
                            }
                        }
                    }
                }
            })
            .unwrap();
    }

    for pgroup in pgroups {
        println!("Killing pgroup: {:?}", pgroup.as_raw());
        kill(pgroup, Some(SIGKILL)).unwrap();
    }

    Ok(())
}
