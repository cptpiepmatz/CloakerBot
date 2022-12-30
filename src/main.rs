use crate::preparations::file_extract::extract_files;

mod assets;
mod embedded_files;
mod preparations;

fn main() {
    extract_files().unwrap();
}
