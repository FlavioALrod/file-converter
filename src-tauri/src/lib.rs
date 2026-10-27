mod commands;
mod conversion;
mod error;
mod fs_utils;
mod manager;

use tauri::Manager;

use conversion::registry::ConversionRegistry;
use manager::{db, scheduler};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(ConversionRegistry::default())
        .setup(|app| {
            // O banco e o agendador so podem ser inicializados aqui dentro
            // do setup, porque precisam do AppHandle (para achar o
            // diretorio de dados do app) -- nao dava para fazer isso no
            // nivel do Builder, como o ConversionRegistry acima, que nao
            // depende de nada alem de si mesmo.
            let handle = app.handle().clone();

            let database = db::init(&handle).expect(
                "nao foi possivel inicializar o banco de dados do gerenciador de arquivos",
            );
            app.manage(database);

            // O agendador comeca a rodar aqui, em segundo plano, e fica
            // ativo durante toda a vida do app -- nao so quando a Aba 2
            // esta visivel.
            let scheduler_handle = scheduler::spawn(handle);
            app.manage(scheduler_handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::convert_file::convert_file,
            commands::cancel_conversion::cancel_conversion,
            commands::files::list_folder,
            commands::files::list_all_folders,
            commands::files::create_folder,
            commands::files::import_files,
            commands::files::rename_folder,
            commands::files::rename_file,
            commands::files::move_folder,
            commands::files::move_file,
            commands::files::delete_folder,
            commands::files::delete_file,
            commands::files::set_folder_favorite,
            commands::files::set_file_favorite,
            commands::files::duplicate_folder,
            commands::files::duplicate_file,
            commands::jobs::create_job,
            commands::jobs::list_jobs,
            commands::jobs::cancel_job,
            commands::jobs::retry_job,
            commands::jobs::delete_job,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar a aplicacao Tauri");
}
