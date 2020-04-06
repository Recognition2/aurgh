use clap::clap_app;
use duct::cmd;
use itertools::Itertools;
use std::{thread::sleep, time::Duration};

fn is_valid_pkg_file(s: String) -> Result<(), String> {
    if s.contains(".pkg.tar.") {
        return Ok(());
    }
    Err("pkg file does not end with typical format `.pkg.tar.xz`".to_string())
}

fn aurto_sync() -> Result<(), std::io::Error> {
    cmd!("sudo", "pacsync", "aurto").stdout_null().stderr_null().run()?;
    Ok(())
}

// Todo: Create global lock
fn add(pkgs: Vec<&str>, edit: bool) {
    let aur_pkglist = cmd!("aur", "pkglist")
        .pipe(cmd!("sort"))
        .stdout_capture()
        .read()
        .expect("Command failed!")
        .lines()
        .map(str::to_owned)
        .collect_vec();

    fn aur_check_deps(needle: &str, haystack: &Vec<String>) -> Vec<String> {
        println!("My needle is [{}]", needle);
        let stack = cmd!("aur", "depends", needle)
            .stderr_null()
            .pipe(cmd!("cut", "-f2"))
            .pipe(cmd!("sort"))
            .stdout_capture()
            .read()
            .expect("Command failed!")
            .lines()
            .filter(|my| haystack.iter().any(|aur| aur == my))
            .map(str::to_owned)
            .collect();
        println!("stack is [{:?}]", stack);
        stack
    }

    let pkgs_and_deps: Vec<_> = pkgs
        .into_iter()
        .map(|pkg| aur_check_deps(pkg, &aur_pkglist).into_iter())
        .kmerge()
        .dedup()
        .collect();
    println!("pkgs and deps are {:?}", pkgs_and_deps);

    let all = pkgs_and_deps.join(" ");
    println!("pkgs and deps are {}", all);

    let sync = cmd!(
        "aur",
        "sync",
        "--chroot",
        "--database=aurto",
        "--makepkg-conf=/etc/aurto/makepkg-chroot.conf",
        all
    )
    .start()
    .unwrap();

    sleep(Duration::from_millis(3000));

    let out = sync.wait().unwrap();
    println!("Output is {:?}", out);
}

fn remove(pkgs: Vec<&str>) {
    for pkg in pkgs {
        println!("Updating pkg {}", pkg);
    }
}

fn addpkg(pkgs: Vec<&str>) {
    for pkg in pkgs {
        println!("Updating pkg {}", pkg);
    }
}

fn update(pkgs: Vec<&str>, edit: bool) {
    for pkg in pkgs {
        println!("Updating pkg {}", pkg);
    }
}
fn main() {
    let app = clap_app!(myapp =>
        (version: "0.0")
        (author: "Kevin H. <kevin@kevinhill.nl>")
        (about: "aur-utils wrapper")
        (@arg verbose: ... -v --verbose "Increase verbosity")
        (@subcommand status =>
            (about: "Get status of `aurto` repository")
        )
        (@subcommand add =>
            (about: "Add packages to `aurto` repository")
            (@arg EDIT_PKGBUILD: -e --edit "Edit PKGBUILD  before building")
            (@arg packages: * "Package(s) to add")
        )
        (@subcommand update =>
            (about: "Try to update all packages in the `aurto` repository. Force rebuild of <packages>")
            (@arg EDIT_PKGBUILD: -e --edit "Edit PKGBUILD  before building")
            (@arg packages: "Package(s) to update")
        )
        (@subcommand remove =>
            (about: "Remove packages from `aurto` repository")
            (@arg packages: * "Package(s) to remove")
        )
        (@subcommand addpkg =>
            (about: "Add packages files to `aurto` repository")
            (@arg packages: * {is_valid_pkg_file} "Package(s) to add")
        )
    );
    let cli_args = app.clone().get_matches();

    match cli_args.subcommand() {
        ("add", Some(sub)) => add(
            sub.values_of("packages").unwrap().collect(),
            sub.is_present("EDIT_PKGBUILD"),
        ),
        ("remove", Some(sub)) => remove(sub.values_of("packages").unwrap().collect()),
        ("update", Some(sub)) => update(
            sub.values_of("packages").unwrap().collect(),
            sub.is_present("EDIT_PKGBUILD"),
        ),
        ("addpkg", Some(sub)) => addpkg(sub.values_of("packages").unwrap().collect()),

        _ => app
            .write_help(&mut std::io::stdout())
            .expect("Failed to write to stdout"),
    }
}
