use std::{env, ffi::OsStr, fs::File, io, io::Write, path::Path, process::Command};

fn main() -> io::Result<()> {
    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    write_infos(&dst)
}

fn write_infos(dst: &Path) -> io::Result<()> {
    let mut built_file = File::create(&dst)?;
    built_file.write_all(
        "//\n// AUTO-GENERATED FILE, DO NOT MODIFY, IT WILL BE REGENERATED ON NEXT COMPILATION\n\n"
            .as_ref(),
    )?;

    write_compiler_version(env::var("RUSTC").unwrap().as_ref(), &mut built_file)
}

fn write_compiler_version(rustc: &OsStr, w: &mut File) -> io::Result<()> {
    let rustc_version = get_version_from_cmd(&rustc)?;
    write_str_variable(w, "RUSTC_VERSION", rustc_version)?;

    Ok(())
}

fn get_version_from_cmd(exec: &OsStr) -> io::Result<String> {
    let output = Command::new(exec).arg("-V").output()?;
    let s = String::from_utf8(output.stdout).unwrap();
    Ok(s.trim().into())
}

fn write_str_variable(w: &mut File, name: &str, value: String) -> io::Result<()> {
    let value = format!("r\"{}\"", value);
    write_variable(w, name, "&str", value)
}

fn write_variable(w: &mut File, name: &str, type_: &str, value: String) -> io::Result<()> {
    writeln!(
        w,
        "#[allow(dead_code)]\npub const {}: {} = {};\n",
        name, type_, value
    )
}
