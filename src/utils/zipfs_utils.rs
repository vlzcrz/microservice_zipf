use std::collections::HashMap;

use delete::delete_file_async;
use rocket::Error;
use serde::Serialize;

use crate::utils::{linear_regression_utils::linear_regression_x1, sort_utils::merge_sort};

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
    words_trend_n50: Vec<String>,
    values_trend_n50: Vec<u32>,
    total_words: u32,
    total_different_words: u32,
    linear_regression_parameters: Vec<f64>,
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
    let total: u32 = values.iter().sum();
    let ranking: Vec<u32> = (1..=capacity).collect();

    let mut words_first_n50: Vec<String> = Vec::with_capacity(50);
    let mut values_first_n50: Vec<u32> = Vec::with_capacity(50);

    if capacity < 50 {
        for word in keys.iter().take(capacity as usize) {
            words_first_n50.push(word.to_string());
        }
        for value in values.iter().take(capacity as usize) {
            values_first_n50.push(*value);
        }
    } else {
        for words in keys.iter().take(50) {
            words_first_n50.push(words.to_string());
        }
        for value in values.iter().take(50) {
            values_first_n50.push(*value);
        }
    }

    // Aplicamos log base 10 para graficar asimilando una recta con pendiente negativa
    let log_values: Vec<f64> = values.iter().map(|&val| (val as f64).log10()).collect();
    let log_ranking: Vec<f64> = ranking.iter().map(|&val| (val as f64).log10()).collect();

    let linregress_parameters = linear_regression_x1(&log_values, &log_ranking).unwrap();

    println!("{:?}", keys);
    println!("{:?}", log_ranking);
    println!("{:?}", log_values);
    println!("{:?}", words_first_n50);
    println!("{:?}", total);
    println!("{:?}", capacity);
    println!("{:?}", linregress_parameters);

    Ok(ZipfResponse {
        vector_keys: keys,
        vector_values: log_values,
        vector_ranking: log_ranking,
        words_trend_n50: words_first_n50,
        values_trend_n50: values_first_n50,
        total_words: total,
        total_different_words: capacity,
        linear_regression_parameters: linregress_parameters,
    })
}
