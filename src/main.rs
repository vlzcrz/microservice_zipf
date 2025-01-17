use delete::delete_file_async;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Serialize;

use rocket::{form::Form, fs::TempFile};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};
use std::{
    collections::HashMap,
    fs::File,
    io::{Error, Read},
};
use uuid::Uuid;

#[macro_use]
extern crate rocket;

const UPLOAD_DIR: &str = "/tmp";

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[derive(FromForm)]
struct Upload<'f> {
    file: TempFile<'f>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ZipfResponse {
    vector_keys: Vec<String>,
    vector_values: Vec<f64>,
    vector_ranking: Vec<f64>,
}

#[post("/upload", format = "multipart/form-data", data = "<form>")]
async fn upload(mut form: Form<Upload<'_>>) -> Result<Json<ZipfResponse>, (Status, String)> {
    // Verificar que el archivo exista
    let filename = form.file.name().unwrap_or("null");

    if filename == "null" {
        return Err((Status::BadRequest, "El archivo no existe".to_string()));
    }

    if !is_text_file(&form.file) {
        return Err((
            Status::BadRequest,
            "El archivo no es un archivo de texto válido.".to_string(),
        ));
    }
    // Asegurarse de que el directorio de uploads exista
    if !std::path::Path::new(UPLOAD_DIR).exists() {
        std::fs::create_dir_all(UPLOAD_DIR).map_err(|e| {
            (
                Status::InternalServerError,
                format!("Error al crear el directorio de uploads: {}", e),
            )
        })?;
    }

    // Crear un identificador único para el archivo
    let file_id: String = Uuid::new_v4().to_string();
    let file_path = format!("{}/{}.txt", UPLOAD_DIR, file_id);

    println!("Guardando archivo en: {}", file_path);

    // Persistir el archivo al directorio destino
    form.file.copy_to(&file_path).await.map_err(|e| {
        (
            Status::InternalServerError,
            format!("Error al guardar archivo: {}", e),
        )
    })?;

    println!("Archivo guardado con éxito en: {}", file_path);

    // Agregar el script de la ley de zipf
    let content = read_document(&file_path).map_err(|e| {
        (
            Status::InternalServerError,
            format!("Error al leer el archivo: {}", e),
        )
    })?;

    let mut words: HashMap<&str, u32> = HashMap::new();
    for word in content.split_whitespace() {
        let count = words.entry(word).or_insert(0);
        *count += 1;
    }

    let mut keys: Vec<String> = Vec::new();
    let mut values: Vec<u32> = Vec::new();

    for (key, value) in &words {
        keys.push(key.to_string());
        values.push(*value);
    }

    get_zipf_law_results(&mut keys, &mut values);
    delete_file_async(&file_path).await.unwrap();

    keys.reverse();
    values.reverse();
    let capacity = values.len() as u32;
    let ranking: Vec<u32> = (1..=capacity).collect();

    // Aplicamos log base 10 para graficar asimilando una recta con pendiente negativa
    let log_values: Vec<f64> = values.iter().map(|&val| (val as f64).log10()).collect();
    let log_ranking: Vec<f64> = ranking.iter().map(|&val| (val as f64).log10()).collect();

    Ok(Json(ZipfResponse {
        vector_keys: keys,
        vector_values: log_values,
        vector_ranking: log_ranking,
    }))
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(CORS).mount("/", routes![upload])
}

// Una función que obtiene la extension de un archivo
//fn get_extension_from_filename(filename: &str) -> Option<&str> {
//    println!("{}", filename);
//    Path::new(filename).extension().and_then(OsStr::to_str)
//}

// Función para verificar si el archivo es de tipo texto
fn is_text_file(file: &TempFile<'_>) -> bool {
    // Verificar el tipo MIME (propiedades del archivo)
    if let Some(content_type) = file.content_type() {
        if content_type.is_text() {
            return true;
        } else {
            return false;
        }
    } else {
        return false;
    }
}

// una función que permita leer el documento
fn read_document(path: &str) -> Result<String, Error> {
    let mut f = File::open(path)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    Ok(content.to_lowercase())
}

// Una función que permita obtener la tabla (o vectores) 'ranking' y 'frecuencia' según el documento leido
fn get_zipf_law_results(keys_vector: &mut Vec<String>, values_vector: &mut Vec<u32>) {
    merge_sort(
        values_vector,
        keys_vector,
        0,
        (values_vector.len() - 1) as u32,
    );
}

// Función de sorting para vectores aplicando mergesort
fn merge_sort(
    vector_pointer_values: &mut Vec<u32>,
    vector_pointer_keys: &mut Vec<String>,
    left: u32,
    right: u32,
) {
    // Condición de salida
    if left >= right {
        return;
    }

    // Delimitador "mid" para separar el vector en 2
    let mid = left + (right - left) / 2;

    merge_sort(vector_pointer_values, vector_pointer_keys, left, mid);
    merge_sort(vector_pointer_values, vector_pointer_keys, mid + 1, right);

    // función merge
    merge(vector_pointer_values, vector_pointer_keys, left, mid, right);
}

fn merge(
    vector_pointer_values: &mut Vec<u32>,
    vector_pointer_keys: &mut Vec<String>,
    left: u32,
    mid: u32,
    right: u32,
) {
    // Declaramos la cantidad de valores que existen para el lado derecho y izquierdo del vector
    let q_left: u32 = mid - left + 1;
    let q_right: u32 = right - mid;

    // Vector de referencias mutables

    // Creación vector auxiliar para mantener las referencias de los datos a manipular
    let mut aux_vec_left: Vec<u32> = Vec::with_capacity(q_left as usize);
    let mut aux_vec_right: Vec<u32> = Vec::with_capacity(q_right as usize);

    let mut aux_vec_keys_left: Vec<String> = Vec::with_capacity(q_left as usize);
    let mut aux_vec_keys_right: Vec<String> = Vec::with_capacity(q_right as usize);

    // Copiamos la data al vector auxiliar
    //      Data auxiliar para indexar
    let mut i: u32 = 0;
    let mut j: u32 = 0;

    //      Copia de valores a los vectores auxiliares
    while i < q_left {
        aux_vec_left.push(vector_pointer_values[(left + i) as usize]);
        aux_vec_keys_left.push(vector_pointer_keys[(left + i) as usize].clone());
        i += 1;
    }

    while j < q_right {
        aux_vec_right.push(vector_pointer_values[(mid + j + 1) as usize]);
        aux_vec_keys_right.push(vector_pointer_keys[(mid + j + 1) as usize].clone());
        j += 1;
    }

    // Reiniciamos los valores indexados
    i = 0;
    j = 0;
    let mut k: u32 = left;

    // Modificaremos el vector principal para ordenarlo usando los auxiliares que tienen los valores ya guardados
    while (i < q_left) && (j < q_right) {
        if aux_vec_left[i as usize] <= aux_vec_right[j as usize] {
            vector_pointer_values[k as usize] = aux_vec_left[i as usize];
            vector_pointer_keys[k as usize] = aux_vec_keys_left[i as usize].clone();

            i += 1;
        } else {
            vector_pointer_values[k as usize] = aux_vec_right[j as usize];
            vector_pointer_keys[k as usize] = aux_vec_keys_right[j as usize].clone();
            j += 1;
        }
        k += 1;
    }

    // En caso de que se hayan acabado los valores del vector derecho
    while i < q_left {
        vector_pointer_values[k as usize] = aux_vec_left[i as usize];
        vector_pointer_keys[k as usize] = aux_vec_keys_left[i as usize].clone();
        i += 1;
        k += 1;
    }
    // En caso de que se hayan acabado los valores del vector izquierdo
    while j < q_right {
        vector_pointer_values[k as usize] = aux_vec_right[j as usize];
        vector_pointer_keys[k as usize] = aux_vec_keys_right[j as usize].clone();
        j += 1;
        k += 1;
    }
    return;
}
