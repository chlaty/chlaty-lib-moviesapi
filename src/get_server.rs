use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::panic;
use std::ptr;
use urlencoding::{decode};
use reqwest::{header::{HeaderMap, HeaderValue, HOST, REFERER, ORIGIN, USER_AGENT}};
use url::Url;

use serde::{Deserialize, Serialize};
use serde_json::{from_str};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Timeline {
    pub start: usize,
    pub end: usize
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceInfo {
    pub file: String,

    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackInfo {
    pub file: String,
    pub label: Option<String>,
    pub kind: String,
    pub default: Option<bool>
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data { 
    pub intro: Option<Timeline>,
    pub outro: Option<Timeline>,
    pub sources: Vec<SourceInfo>,
    pub tracks: Vec<TrackInfo>,
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub referer: String,
    pub origin: String,
    pub playlist_base_url: String,
    pub segment_base_url: String
}

#[derive(Serialize, Deserialize)]
struct ServerInfo {
    status: bool,
    message: String,
    data: Option<Data>,
    config: Option<Config>,
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    id: String,
    index: usize
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetServerResult{
    pub data: Data,
    pub config: Config
}

#[derive(Debug, Serialize, Deserialize)]
struct EpisodeInfo{
    id_type: String,
    imdb_id: String,
    tmdb_id: String,
    s: Option<usize>,
    e: Option<usize>
}

#[derive(Debug, Serialize, Deserialize)]
struct HlsAndTrack{
    url: String,
    tracks: Vec<TrackInfo>

}


#[unsafe(no_mangle)]
pub extern "C" fn get_server(
    arguments_ptr: *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {
        let mut return_result = ServerInfo {
            status: false,
            message: String::from(""),
            data: None,
            config: None,
        };

        // Check argument before processing
        if arguments_ptr.is_null() {
            panic!("Expected 1 argument.");
        }

        let args: Arguments = unsafe { 
            from_str(&CStr::from_ptr(arguments_ptr as *mut c_char).to_string_lossy().into_owned()).unwrap()
        };
        
        // ================================================

        let raw_episode_id = args.id;

        let episode_info: EpisodeInfo = from_str(&decode(&raw_episode_id).unwrap()).unwrap();

        

        let url:String;

        if episode_info.e.is_some() && episode_info.s.is_some() {
            url = format!("https://w1.moviesapi.to/ajax-tv.php?tmdb={}&season={}&episode={}",
                episode_info.tmdb_id,
                episode_info.s.unwrap()+1,
                episode_info.e.unwrap()+1
            );
        }else{
            url = format!("https://w1.moviesapi.to/ajax.php?tmdb={}",
                episode_info.tmdb_id
            );
        }

        let client = reqwest::blocking::Client::new();
        let mut headers = HeaderMap::new();
        
        headers.insert(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36"
        ));

        headers.insert(HOST, HeaderValue::from_static("w1.moviesapi.to"));
        headers.insert(REFERER, HeaderValue::from_static("https://w1.moviesapi.to/"));
        headers.insert(ORIGIN, HeaderValue::from_static("https://w1.moviesapi.to"));

        let res = client.get(&url)
            .headers(headers).send().unwrap();

        if !res.status().is_success(){
            panic!("Error: {}", res.status());
        }

        let hls_and_track: HlsAndTrack = res.json().unwrap();
        

        let parsed_hls_url = Url::parse(&hls_and_track.url).unwrap();
        let config_host = parsed_hls_url.host().unwrap().to_string();

        let data = Data {
            intro: None,
            outro: None,
            sources: vec![SourceInfo {
                file: hls_and_track.url,
                _type: "hls".to_string()
            }],
            tracks: hls_and_track.tracks
        };

        let config = Config {
            host: config_host.clone(),
            referer: "https://w1.moviesapi.to/".to_string(),
            origin: "https://w1.moviesapi.to".to_string(),
            playlist_base_url: format!("https://{}", &config_host),
            segment_base_url: format!("https://{}", &config_host)
        };

        return_result.status = true;
        return_result.message = "Success".to_string();
        return_result.data = Some(data);
        return_result.config = Some(config);
        
        
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


