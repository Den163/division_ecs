use std::env;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
}