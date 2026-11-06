// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod pdf_generator;

use chrono::Local;
use pdf_generator::{render_quote_pdf, QuotePayload};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static QUOTE_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Metadatos iniciales sugeridos al abrir o reiniciar un presupuesto.
#[derive(Debug, Serialize)]
pub struct InitialMetadata {
    pub suggested_number: String,
    pub today_date: String,
    pub default_validity: String,
    pub default_currency: String,
}

/// Respuesta devuelta al frontend tras exportar el archivo PDF.
#[derive(Debug, Serialize)]
pub struct PdfExportResult {
    pub saved_path: String,
    pub backup_output_path: Option<String>,
    pub total_formatted: String,
}

/// Devuelve la fecha local actual (`DD/MM/YYYY`) y un código sugerido (`PRES-0001`),
/// todo calculado localmente sin acceso a internet ni telemetría.
#[tauri::command]
fn get_initial_metadata() -> InitialMetadata {
    let seq = QUOTE_COUNTER.load(Ordering::Relaxed);
    let now = Local::now();
    InitialMetadata {
        suggested_number: format!("PRES-{:04}", seq),
        today_date: now.format("%d/%m/%Y").to_string(),
        default_validity: "15 días hábiles".to_string(),
        default_currency: "$".to_string(),
    }
}

/// Valida los datos del presupuesto, genera el documento PDF profesional nativo,
/// abre el cuadro de diálogo nativo de Windows para que el usuario elija dónde guardarlo
/// y guarda además una copia automática en la carpeta `outputs/` del proyecto si existe.
#[tauri::command]
async fn generate_and_save_pdf(payload: QuotePayload) -> Result<PdfExportResult, String> {
    let calculated = payload.validate_and_calculate()?;
    let pdf_bytes = render_quote_pdf(&calculated)?;

    let safe_code: String = calculated
        .payload
        .quote_number
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let default_filename = format!("Presupuesto_{}.pdf", safe_code);

    // Determinar carpeta outputs/ local si estamos dentro del árbol del proyecto
    let mut outputs_dir: Option<PathBuf> = None;
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join("outputs");
        if candidate.is_dir() {
            outputs_dir = Some(candidate);
        } else if let Some(parent) = cwd.parent() {
            let parent_candidate = parent.join("outputs");
            if parent_candidate.is_dir() {
                outputs_dir = Some(parent_candidate);
            }
        }
    }

    // Mostrar diálogo nativo de guardado en un hilo bloqueante seguro
    let default_name_clone = default_filename.clone();
    let initial_dir = outputs_dir.clone();

    let chosen_path = tauri::async_runtime::spawn_blocking(move || {
        let mut dialog = rfd::FileDialog::new()
            .set_title("Guardar Presupuesto en PDF")
            .add_filter("Documento PDF (*.pdf)", &["pdf"])
            .set_file_name(&default_name_clone);

        if let Some(dir) = initial_dir {
            dialog = dialog.set_directory(dir);
        }

        dialog.save_file()
    })
    .await
    .map_err(|e| format!("Error al abrir el cuadro de diálogo: {}", e))?;

    let target_path = match chosen_path {
        Some(p) => p,
        None => return Err("CANCELLED".to_string()),
    };

    fs::write(&target_path, &pdf_bytes)
        .map_err(|e| format!("No se pudo escribir el archivo PDF en el destino elegido: {}", e))?;

    // Si existe carpeta outputs/ y el destino elegido es distinto, guardar copia de respaldo en outputs/
    let mut backup_output_path = None;
    if let Some(out_dir) = outputs_dir {
        let out_file = out_dir.join(&default_filename);
        if out_file != target_path {
            if fs::write(&out_file, &pdf_bytes).is_ok() {
                backup_output_path = Some(out_file.to_string_lossy().to_string());
            }
        }
    }

    // Incrementar el contador local para el siguiente presupuesto en la sesión
    QUOTE_COUNTER.fetch_add(1, Ordering::Relaxed);

    let total_formatted =
        pdf_generator::format_currency(calculated.total, &calculated.payload.currency);

    Ok(PdfExportResult {
        saved_path: target_path.to_string_lossy().to_string(),
        backup_output_path,
        total_formatted,
    })
}

fn main() {
    // Si se invoca con `--generate-sample-pdf <ruta>`, genera un PDF de muestra en `outputs/` y termina.
    // Esto permite verificar la generación PDF y poblar `outputs/` desde línea de comandos o tests.
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--generate-sample-pdf" {
        let out_path = PathBuf::from(&args[2]);
        let sample = QuotePayload {
            quote_number: "PRES-0001".into(),
            issue_date: Local::now().format("%d/%m/%Y").to_string(),
            validity: "15 días hábiles".into(),
            currency: "$".into(),
            professional: pdf_generator::PartyInfo {
                first_name: "Victoria".into(),
                last_name: "Sanjurjo".into(),
                phone: "+54 11 5555-0100".into(),
                email: "victoria.sanjurjo@profesional.com".into(),
            },
            client: pdf_generator::PartyInfo {
                first_name: "Alejandro".into(),
                last_name: "Mendoza".into(),
                phone: "+54 11 4789-3321".into(),
                email: "amendoza@empresa-cliente.com".into(),
            },
            items: vec![
                pdf_generator::QuoteItem {
                    description: "Diseño de arquitectura y prototipado de interfaz UI/UX".into(),
                    quantity: 1.0,
                    unit_price: 185000.0,
                },
                pdf_generator::QuoteItem {
                    description: "Desarrollo de aplicación de escritorio nativa (Rust + Tauri v2)".into(),
                    quantity: 1.0,
                    unit_price: 420000.0,
                },
                pdf_generator::QuoteItem {
                    description: "Empaquetado MSIX firmado y documentación técnica integral".into(),
                    quantity: 1.0,
                    unit_price: 95000.0,
                },
            ],
        };
        let calc = sample
            .validate_and_calculate()
            .expect("Datos de muestra inválidos");
        let bytes = render_quote_pdf(&calc).expect("Fallo al renderizar PDF de muestra");
        if let Some(parent) = out_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&out_path, bytes).expect("No se pudo guardar el PDF de muestra");
        println!("PDF de muestra generado exitosamente en: {}", out_path.display());
        return;
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_initial_metadata,
            generate_and_save_pdf
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación Tally");
}

