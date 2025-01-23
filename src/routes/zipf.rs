use crate::utils::file_utils::{get_extension, read_document_pdf, read_document_txt};
use crate::utils::zipfs_utils::zipf_law_process;
use crate::utils::zipfs_utils::ZipfResponse;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use uuid::Uuid;

const UPLOAD_DIR: &str = "/tmp";

#[derive(FromForm)]
pub struct Upload<'f> {
    file: TempFile<'f>,
}

#[post("/zipf-plot", format = "multipart/form-data", data = "<form>")]
pub async fn zipf_plot(mut form: Form<Upload<'_>>) -> Result<Json<ZipfResponse>, (Status, String)> {
    // Verificar que el archivo exista
    let filename = form.file.name().unwrap_or("null");

    if filename == "null" {
        return Err((Status::BadRequest, "El archivo no existe".to_string()));
    }

    let extension = get_extension(&form.file).unwrap();

    // Crear un identificador único para el archivo
    let file_id: String = Uuid::new_v4().to_string();
    let file_path = format!("{}/{}.{}", UPLOAD_DIR, file_id, extension);

    // Copiar el archivo al directorio destino
    form.file.copy_to(&file_path).await.map_err(|e| {
        (
            Status::InternalServerError,
            format!("Error al guardar archivo: {}", e),
        )
    })?;

    // Agregar el script de la ley de zipf
    let content;
    if extension == "pdf" {
        content = read_document_pdf(&file_path).map_err(|e| {
            (
                Status::InternalServerError,
                format!("Error al leer el archivo pdf: {}", e),
            )
        })?;
    } else {
        content = read_document_txt(&file_path).map_err(|e| {
            (
                Status::InternalServerError,
                format!("Error al leer el archivo txt: {}", e),
            )
        })?;
    }

    let response = zipf_law_process(content, &file_path).await.unwrap();
    Ok(Json(response))
}
