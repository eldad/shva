fn main() {
    // Since we embed the migrations, force cargo to recompile when any change in the directory.
    println!("cargo:rerun-if-changed=migrations")
}
