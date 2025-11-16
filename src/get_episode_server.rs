use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char};
use std::panic;
use std::ptr;

use serde::{Deserialize, Serialize};
use serde_json::{from_str};





#[derive(Debug, Serialize, Deserialize)]
struct EpisodeInfo{
    id_type: String,
    imdb_id: String,
    tmdb_id: String,
    s: Option<usize>,
    e: Option<usize>
}


#[derive(Debug, Serialize, Deserialize)]
struct EpisodeServerData{
    index: usize,
    id: String,
    title: String,
    verify_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReturnResult {
    status: bool,
    message: String,
    data: HashMap<String, Vec<EpisodeServerData>>,
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    episode_id: String
}


#[unsafe(no_mangle)]
pub extern "C" fn get_episode_server(
    arguments_ptr : *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {
        let mut return_result = ReturnResult {
            status: false,
            message: String::from(""),
            data: HashMap::new(),
            
        };

        // Check argument before processing
        if arguments_ptr.is_null() {
            panic!("Expected 1 argument.");
        }

        let args: Arguments = unsafe { 
            from_str(&CStr::from_ptr(arguments_ptr as *mut c_char).to_string_lossy().into_owned()).unwrap()
        };
        
        // ================================================

        

        let raw_episode_id = args.episode_id;

        let server_data = vec![
            EpisodeServerData{
                id: raw_episode_id,
                index: 0,
                title: "MoviesAPI".to_string(),
                verify_url: None
            }
        ];
        
        return_result.data.insert(String::from("SERVER"), server_data);

        return_result.status = true;
        return_result.message = String::from("Success");
        
        
        return serde_json::to_string(&return_result).unwrap();
    });
    
    match result {
        Ok(data) => {
            let result = CString::new(data).unwrap();
            return result.into_raw();
        },
        _ => ptr::null(),
    }
}
