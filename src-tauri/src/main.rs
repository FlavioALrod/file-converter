// Impede que uma janela de console extra se abra no Windows em builds de
// release, sem afetar Linux/macOS. Padrao gerado pelo proprio Tauri.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    file_converter_lib::run();
}
