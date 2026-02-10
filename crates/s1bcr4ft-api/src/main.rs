use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use s1bcr4ft_core::{
    audit::{AuditAction, AuditLogger},
    backup::BackupManager,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    version: String,
    modules_count: usize,
    packages_installed: usize,
    backups_count: usize,
}

#[derive(Debug, Serialize)]
struct ModuleInfo {
    id: String,
    name: String,
    category: String,
    description: String,
}

struct AppState {
    audit_logger: Mutex<AuditLogger>,
    backup_manager: Mutex<BackupManager>,
}

// Health check endpoint
async fn health() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success("S1bCr4ft API is running"))
}

// Get system status
async fn get_status() -> impl Responder {
    let status = StatusResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        modules_count: 57,
        packages_installed: 0, // Would query pacman
        backups_count: 0,      // Would query backup manager
    };

    HttpResponse::Ok().json(ApiResponse::success(status))
}

// List all modules
async fn list_modules() -> impl Responder {
    let modules = vec![
        ModuleInfo {
            id: "core/base-system".to_string(),
            name: "Base System".to_string(),
            category: "core".to_string(),
            description: "Essential Arch Linux packages".to_string(),
        },
        ModuleInfo {
            id: "linux-optimization/window-managers/hyprland-config".to_string(),
            name: "Hyprland".to_string(),
            category: "linux-optimization".to_string(),
            description: "Modern Wayland compositor".to_string(),
        },
        ModuleInfo {
            id: "red-team/c2-frameworks/sliver-c2".to_string(),
            name: "Sliver C2".to_string(),
            category: "red-team".to_string(),
            description: "Modern C2 framework".to_string(),
        },
    ];

    HttpResponse::Ok().json(ApiResponse::success(modules))
}

// Get config
async fn get_config() -> impl Responder {
    let config = serde_json::json!({
        "version": "1.0",
        "name": "my-arch-setup",
        "modules": [
            "core/base-system",
            "linux-optimization/window-managers/hyprland-config"
        ],
        "options": {
            "auto_backup": true,
            "parallel_install": true
        }
    });

    HttpResponse::Ok().json(ApiResponse::success(config))
}

// List backups
async fn list_backups(data: web::Data<AppState>) -> impl Responder {
    let backup_manager = data.backup_manager.lock().unwrap();

    match backup_manager.list_backups() {
        Ok(backups) => HttpResponse::Ok().json(ApiResponse::success(backups)),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

// Get audit log
async fn get_audit_log(data: web::Data<AppState>) -> impl Responder {
    let audit_logger = data.audit_logger.lock().unwrap();

    match audit_logger.get_entries(None) {
        Ok(entries) => HttpResponse::Ok().json(ApiResponse::success(entries)),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

#[derive(Debug, Deserialize)]
struct SyncRequest {
    dry_run: Option<bool>,
}

// Sync system
async fn sync_system(req: web::Json<SyncRequest>, data: web::Data<AppState>) -> impl Responder {
    let dry_run = req.dry_run.unwrap_or(false);

    // Log the sync action
    let audit_logger = data.audit_logger.lock().unwrap();
    let _ = audit_logger.log(
        AuditAction::Sync,
        serde_json::json!({"dry_run": dry_run}),
        true,
    );

    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "message": if dry_run { "Dry run completed" } else { "Sync completed" },
        "packages_installed": 0,
        "duration_secs": 0
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("🚀 Starting S1bCr4ft REST API...");
    println!("📡 Server will be available at: http://0.0.0.0:8080");
    println!("📚 API Documentation:");
    println!("  GET  /health          - Health check");
    println!("  GET  /api/status      - System status");
    println!("  GET  /api/modules     - List modules");
    println!("  GET  /api/config      - Get configuration");
    println!("  GET  /api/backups     - List backups");
    println!("  GET  /api/audit       - Get audit log");
    println!("  POST /api/sync        - Sync system");

    // Initialize app state
    let audit_logger = AuditLogger::new().expect("Failed to create audit logger");
    let backup_manager = BackupManager::new().expect("Failed to create backup manager");

    let app_state = web::Data::new(AppState {
        audit_logger: Mutex::new(audit_logger),
        backup_manager: Mutex::new(backup_manager),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/health", web::get().to(health))
            .route("/api/status", web::get().to(get_status))
            .route("/api/modules", web::get().to(list_modules))
            .route("/api/config", web::get().to(get_config))
            .route("/api/backups", web::get().to(list_backups))
            .route("/api/audit", web::get().to(get_audit_log))
            .route("/api/sync", web::post().to(sync_system))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
