use rocket::fs::TempFile;
use std::{fmt::Error, fs::File, io::Read};

// Funcion para verificar que es una letra que pertenece a nuestro rango ASCII de interes (32-126)
pub fn is_ascii_valid(word: &str, ascii_interest: &Vec<u8>) -> Result<bool, Error> {
    let bytes_word = word.as_bytes();
    for byte in bytes_word {
        if !ascii_interest.contains(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn get_extension(file: &TempFile<'_>) -> Result<String, Error> {
    let content_type = file.content_type();
    let extension = content_type.unwrap().extension().unwrap().to_string();
    Ok(extension)
}

// una función que permita leer el documento pdf
pub fn read_document_pdf(path: &str) -> Result<String, Error> {
    let bytes = std::fs::read(path).unwrap();
    let content = pdf_extract::extract_text_from_mem(&bytes).unwrap();
    Ok(content
        .to_lowercase()
        .replace(&[',', '.', '(', ')', '[', ']'][..], ""))
}

// una función que permita leer el documento
pub fn read_document_txt(path: &str) -> Result<String, Error> {
    let mut f = File::open(path).unwrap();
    let mut content = String::new();
    f.read_to_string(&mut content).ok();
    Ok(content
        .to_lowercase()
        .replace(&[',', '.', '(', ')', '[', ']'][..], ""))
}
