use std::collections::HashMap;

use delete::delete_file_async;
use rocket::Error;
use serde::Serialize;

use crate::utils::sort_utils::merge_sort;

use super::file_utils::is_ascii_valid;

// Una función que permita obtener la tabla (o vectores) 'ranking' y 'frecuencia' según el documento leido
pub fn get_zipf_law_results(keys_vector: &mut Vec<String>, values_vector: &mut Vec<u32>) {
    merge_sort(
        values_vector,
        keys_vector,
        0,
        (values_vector.len() - 1) as u32,
    );
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ZipfResponse {
    vector_keys: Vec<String>,
    vector_values: Vec<f64>,
    vector_ranking: Vec<f64>,
}

pub async fn zipf_law_process(content: String, file_path: &str) -> Result<ZipfResponse, Error> {
    // Configuración ASCII para que solo se adecue a las palabras usadas en idioma ingles
    let mut ascii_interest: Vec<u8> = (97..121).collect();
    ascii_interest.push(39);

    let mut words: HashMap<&str, u32> = HashMap::new();
    for word in content.split_whitespace() {
        if is_ascii_valid(word, &ascii_interest).unwrap() {
            let count = words.entry(word).or_insert(0);
            *count += 1;
        }
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

    Ok(ZipfResponse {
        vector_keys: keys,
        vector_values: log_values,
        vector_ranking: log_ranking,
    })
}
