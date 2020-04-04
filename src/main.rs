use clap::clap_app;

fn is_valid_pkg_file(s: String) -> Result<(), String> {
    if s.ends_with(".pkg.tar.xz") {
        return Ok(());
    }
    Err("pkg file does not end with typical format `.pkg.tar.xz`".to_string())
}

fn add(pkgs: Vec<&str>, edit: bool) {
    for pkg in pkgs {
        println!("Updating pkg {}", pkg);
    }
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
    let mut app = clap_app!(myapp =>
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
