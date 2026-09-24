fn main() {
    // La interfaz se embebe en el binario: recompilar cuando cambie.
    println!("cargo:rerun-if-changed=ui");
    tauri_build::build()
}
