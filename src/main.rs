use crate::preparations::file_extract::extract_files;
use crate::preparations::java::find_java_executable;

mod assets;
mod embedded_files;
mod preparations;

fn main() {
    let java_exec = find_java_executable().unwrap();
    extract_files().unwrap();
}
