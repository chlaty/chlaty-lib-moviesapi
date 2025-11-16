use std::ffi::{CString, CStr};
use std::os::raw::{c_char};
use std::panic;
use std::ptr;


use serde::{Deserialize, Serialize};
use serde_json::{ from_str, Value};
use reqwest::header::{HeaderMap, HOST, ORIGIN, REFERER, HeaderValue, USER_AGENT};
use urlencoding::{encode};
use html_escape::decode_html_entities;

use crate::{SOURCE_HOST, SOURCE_ORIGIN, SOURCE_REFERER};





#[derive(Debug, Serialize, Deserialize)]
struct Data{
    id: String,
    title: String,
    cover: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ReturnResult {
    status: bool,
    message: String,
    data: Vec<Data>,
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    search: String
}


#[unsafe(no_mangle)]
pub extern "C" fn search(
    arguments_ptr : *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {
        let mut return_result = ReturnResult {
            status: false,
            message: String::from(""),
            data: Vec::new(),
        };

        // Check argument before processing
        if arguments_ptr.is_null() {
            panic!("Expected 1 argument.");
        }

        let args: Arguments = unsafe { 
            from_str(&CStr::from_ptr(arguments_ptr as *mut c_char).to_string_lossy().into_owned()).unwrap()
        };
        
        // ================================================


        let search_string = args.search;

        let client = reqwest::blocking::Client::new();
        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_static(
            "Chrome/126.0.0.0"
        ));

        headers.insert(HOST, HeaderValue::from_str(SOURCE_HOST).unwrap());
        headers.insert(REFERER, HeaderValue::from_str(SOURCE_REFERER).unwrap());
        headers.insert(ORIGIN, HeaderValue::from_str(SOURCE_ORIGIN).unwrap());

        let mut new_data: Vec<Data> = Vec::new();


        for search_type in ["movies", "tv"] {
            let url = format!("https://{}/ajax/full/search.php?s={}&type={}&sort=0", 
                SOURCE_HOST, 
                if search_string.trim().is_empty() { "+".to_string() } else { encode(&search_string).to_string() },
                search_type,
            );

            let res = client.post(&url)
                .headers(headers.clone())
                .send()
                .unwrap();


            if !res.status().is_success(){
                panic!("Error: {}", res.status());
            }

            /* Do the work here */
            let data:Value = res.json().unwrap();
            
            for (_, value) in data.as_object().unwrap().into_iter() {
                
                let title: String = value.get("titles")
                    .and_then(|v| v.get("m"))
                    .unwrap_or(&Value::String("".to_string()))
                    .as_str().unwrap().to_string();

                let cover: String = format!("https://simkl.in/posters/{}_m.webp",
                    value.get("poster").unwrap().as_str().unwrap().to_string()
                );

                let raw_id: String = value.get("url").unwrap().as_str().unwrap().to_string();

                let id = encode(&raw_id).to_string();

                new_data.push(Data {
                    id: id,
                    title: decode_html_entities(&title).to_string(),
                    cover: cover
                });
            }
            
            /* --- */
        }

        return_result.status = true;
        return_result.message = String::from("Success");
        return_result.data = new_data;
        
        return serde_json::to_string(&return_result).unwrap();
        
    });

    match result {
        Ok(data) => {
            let result = CString::new(data).unwrap();
            return result.into_raw();
        },
        Err(e) => {
            eprintln!("[Search] Error: {:?}", e);
            return ptr::null();
        },
    }
}