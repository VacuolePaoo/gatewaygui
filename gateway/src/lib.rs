use plugins::file_manager;
use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;

pub mod plugins;
pub mod gateway;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .manage(file_manager::FileManagerState {
            selected_files: std::sync::Mutex::new(Vec::new()),
        });

    // CrabNebula DevTools prevents other logging plugins from working
    // https://docs.crabnebula.dev/devtools/troubleshoot/log-plugins/
    #[cfg(debug_assertions)]
    {
        let devtools = tauri_plugin_devtools::init();
        builder = builder.plugin(devtools);
    }

    #[cfg(not(debug_assertions))]
    {
        builder = builder.plugin(plugins::logging::tauri_plugin_logging());
    }
    builder
        .plugin(tauri_plugin_decorum::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            file_manager::set_selected_files,
            file_manager::get_selected_files,
            file_manager::clear_selected_files,
            // Gateway API
            gateway::tauri_api::start_gateway,
            gateway::tauri_api::stop_gateway,
            gateway::tauri_api::restart_gateway,
            gateway::tauri_api::get_gateway_status,
            
            // Configuration API
            gateway::tauri_api::get_gateway_config,
            gateway::tauri_api::update_gateway_config,
            gateway::tauri_api::validate_config,
            gateway::tauri_api::reset_to_default_config,
            
            // Directory API
            gateway::tauri_api::mount_directory,
            gateway::tauri_api::unmount_directory,
            gateway::tauri_api::get_mount_points,
            gateway::tauri_api::list_directory,
            gateway::tauri_api::create_search_token,
            gateway::tauri_api::validate_search_token,
            gateway::tauri_api::authorize_file,
            gateway::tauri_api::get_metadata_by_token,
            gateway::tauri_api::confirm_data_transfer,
            gateway::tauri_api::create_data_transfer_request,
            gateway::tauri_api::get_pending_transfer_requests,
            gateway::tauri_api::get_transfer_request_details,
            gateway::tauri_api::create_file_transfer,
            gateway::tauri_api::get_transfer_status,
            gateway::tauri_api::cancel_transfer,
            gateway::tauri_api::get_all_transfers,
            gateway::tauri_api::cleanup_completed_transfers,
            gateway::tauri_api::get_network_stats,
            
            // Network API
            gateway::tauri_api::get_network_status,
            gateway::tauri_api::start_p2p_discovery,
            gateway::tauri_api::stop_p2p_discovery,
            gateway::tauri_api::get_discovered_nodes,
            gateway::tauri_api::connect_to_node,
            gateway::tauri_api::disconnect_from_node,
            
            // Performance API
            gateway::tauri_api::get_performance_report,
            gateway::tauri_api::get_compression_stats,
            gateway::tauri_api::get_cache_stats,
            gateway::tauri_api::start_performance_benchmark,
            gateway::tauri_api::get_benchmark_result,
            
            // Status API
            gateway::tauri_api::get_system_info,
            gateway::tauri_api::get_service_logs,
            gateway::tauri_api::health_check,
            
            // Security API
            gateway::tauri_api::get_security_config,
            gateway::tauri_api::update_security_config,
            gateway::tauri_api::generate_tls_certificate,
            gateway::tauri_api::add_access_rule,
            gateway::tauri_api::remove_access_rule,
            gateway::tauri_api::get_access_rules,
            gateway::tauri_api::validate_client_access,
            gateway::tauri_api::get_active_sessions,
            gateway::tauri_api::disconnect_session,
        ])
        .setup(|app| {
            // Initialize event emitter
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = gateway::tauri_api::initialize_event_emitter(app_handle).await {
                    eprintln!("Failed to initialize event emitter: {}", e);
                }
            });

            // Create a custom titlebar for main window
            // On Windows this hides decoration and creates custom window controls
            // On macOS it needs hiddenTitle: true and titleBarStyle: overlay
            let main_window = app.get_webview_window("main").unwrap();
            main_window.create_overlay_titlebar().unwrap();

            // Some macOS-specific helpers
            #[cfg(target_os = "macos")]
            {
                // Set a custom inset to the traffic lights
                main_window.set_traffic_lights_inset(12.0, 16.0).unwrap();

                // Make window transparent without privateApi
                main_window.make_transparent().unwrap();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}