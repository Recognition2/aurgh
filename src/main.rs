use clap::{clap_app, ArgMatches};
use duct::cmd;
use itertools::Itertools;

const AURTO_CACHE: &str = "/var/cache/pacman/aurto/";

fn is_valid_pkg_file(s: String) -> Result<(), String> {
    if s.contains(".pkg.tar.") {
        return Ok(());
    }
    Err("pkg file does not end with typical format `.pkg.tar".to_string())
}

fn aurto_sync() -> Result<(), std::io::Error> {
    cmd!("sudo", "pacsync", "aurto").stdout_null().stderr_null().run()?;
    Ok(())
}

// Todo: Create global lock
fn add(sub: &ArgMatches) -> Option<()> {
    let aur_pkglist = cmd!("aur", "pkglist")
        .pipe(cmd!("sort"))
        .stdout_capture()
        .read()
        .expect("Command failed!")
        .lines()
        .map(str::to_owned)
        .collect_vec();

    fn aur_check_deps(needle: &str, haystack: &[String]) -> Vec<String> {
        cmd!("aur", "depends", needle)
            .stderr_null()
            .pipe(cmd!("cut", "-f2"))
            .pipe(cmd!("sort"))
            .stdout_capture()
            .read()
            .expect("Command failed!")
            .lines()
            .filter(|my| haystack.iter().any(|aur| aur == my))
            .map(str::to_owned)
            .collect()
    }
    let pkgs = ["hoi"];

    let pkgs_and_deps: Vec<_> = pkgs
        .into_iter()
        .map(|pkg| aur_check_deps(pkg, &aur_pkglist).into_iter())
        .kmerge()
        .dedup()
        .collect();

    let all = pkgs_and_deps.join(" ");
    println!("Installing {}", all);

    let mut args = vec![
        "sync",
        "--chroot",
        "--database=aurto",
        "--makepkg-conf=/etc/aurto/makepkg-chroot.conf",
    ];

    let edit = false;
    if !edit {
        args.push("--no-view");
        args.push("--no-confirm");
    }

    let bind = Some("meuk");
    let s;
    if let Some(val) = bind {
        s = format!("--bind={}", val);
        args.push(&s);
    }

    pkgs_and_deps.iter().for_each(|pkg| args.push(pkg));
    let sync = cmd("aur", &args).start().ok()?;

    // Wait for sync to finish
    let out = sync.wait().ok()?;
    if out.status.success() {
        println!("Added {} to `aurto` db successfully!", all);
    }
    aurto_sync().ok()?;
    Some(())
}

fn remove(sub: &ArgMatches) -> Option<()> {
    let pkgs = vec!["hoi"];
    let mut removed_pkgs = Vec::new();
    for pkg in pkgs {
        if cmd!("repo-remove", "/var/cache/pacman/aurto/aurto.db.tar", pkg)
            .stderr_to_stdout()
            .stdout_capture()
            .read()
            .unwrap()
            .contains("ERROR")
        {
            println!("Package {} not found!", pkg);
        } else {
            let dir = "/var/cache/pacman/aurto";
            for entry in std::fs::read_dir(dir).ok()? {
                let entry = entry.ok()?;
                let path = entry.path();
                let is_file = path.is_file();
                let is_relevant_package = path.file_name()?.to_str()?.starts_with(pkg);
                let contains_pkg = path.file_name()?.to_str()?.contains(".pkg.");
                if is_file && is_relevant_package && contains_pkg {
                    // Try to remove file
                    if std::fs::remove_file(path).is_ok() && removed_pkgs.iter().all(|&r| r != pkg) {
                        removed_pkgs.push(pkg);
                    }
                }
            }
        }
    }
    println!("Removed packages {}", removed_pkgs.join(" "));

    aurto_sync().ok()?;
    Some(())
}

fn addpkg(sub: &ArgMatches) -> Option<()> {
    let pkgs = vec!["hoi"];
    let mut args = vec!["/var/cache/pacman/aurto/aurto.db.tar"];
    for pkg in &pkgs {
        args.push(pkg);
    }

    cmd("repo-add", args).run().ok()?;
    for pkg in pkgs {
        std::fs::copy(pkg, format!("{}{}", AURTO_CACHE, pkg)).ok()?;
    }

    aurto_sync().ok()?;
    Some(())
}

fn update(sub: &ArgMatches) -> Option<()> {
    let pkgs = vec!["meuk"];
    aurto_sync().ok()?;
    let update_all = pkgs.len() == 0;
    if update_all {
        let mut args = vec![
            "sync",
            "--chroot",
            "--database=aurto",
            "--makepkg-conf=/etc/aurto/makepkg-chroot.conf",
            "--upgrades",
        ];
        let edit = false;

        if !edit {
            args.push("--no-view");
            args.push("--no-confirm");
        }

        let bind = Some("m");
        let s;
        if let Some(val) = bind {
            s = format!("--bind={}", val);
            args.push(&s);
        }
        cmd("aur", args).run().ok()?;
    }


    aurto_sync().ok()?;
    cmd!("paccache", "-rk1", "-c", AURTO_CACHE).run().ok()?;
    Some(())
}
fn main() {
    let app = clap_app!(aurgh =>
        (version: "0.1")
        (author: "Kevin H. <kevin@kevinhill.nl>")
        (about: "aur-utils wrapper")
        (@arg verbose: ... -v --verbose "Increase verbosity")
        (@subcommand status =>
            (about: "Get status of `aurto` repository")
        )
        (@subcommand add =>
            (about: "Add packages to `aurto` repository")
            (@arg EDIT_PKGBUILD: -e --edit "Edit PKGBUILD  before building")
            (@arg bind: --bind [DIR] "Bind directory read-only")
            (@arg packages: ... * "Package(s) to add")
            (@group build_options =>
                (@arg force: -f --force "Continue the build process if a package with the same name exists")
                (@arg ignorefile: --("ignore-file") [FILE] "Ignore package upgrades listed in FILE. Each package name should be specified on a separate line.")
                (@arg ignore: --ignore ... [PACKAGE] "Ignore a package upgrade. Multiple packages can be specified by seperating them with a comma, or by repeating the --ignore option.")
                (@arg nograph: --nograph --no-graph "Do not verify the AUR dependency with aur graph (1).")
            )
            (@group container_options =>
                (@arg chroot: -c --chroot "Build packages with makechrootpkg. (aur build -c)")
            )
        )
        (@subcommand update =>
            (about: "Try to update all packages in the `aurto` repository. Force rebuild of <packages>")
            (@arg EDIT_PKGBUILD: -e --edit "Edit PKGBUILD  before building")
            (@arg bind: --bind [dir] "Bind directory read-only")
            (@arg packages: ... "Package(s) to update")
        )
        (@subcommand remove =>
            (alias: "rm")
            (about: "Remove packages from `aurto` repository")
            (@arg packages: ... * "Package(s) to remove")
        )
        (@subcommand addpkg =>
            (about: "Add packages files to `aurto` repository")
            (@arg packages: ... * {is_valid_pkg_file} "Package(s) to add")
        )
    );
    let cli_args = app.clone().get_matches();

    match cli_args.subcommand() {
        ("add", Some(sub)) => add(sub).unwrap(),
        ("remove", Some(sub)) => remove(sub).unwrap(),
        ("update", Some(sub)) => update(sub).unwrap(),
        ("addpkg", Some(sub)) => addpkg(sub).unwrap(),
        _ => app
            .write_help(&mut std::io::stdout())
            .expect("Failed to write to stdout"),
    }
}
