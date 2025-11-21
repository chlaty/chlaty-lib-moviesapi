use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::panic;
use std::ptr;
use urlencoding::{decode};
use reqwest::{header::{HeaderMap, HeaderValue, HOST, REFERER, ORIGIN, USER_AGENT, HeaderName, ACCEPT_ENCODING}};
use url::Url;
use visdom::Vis;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value, from_value};

use crate::utils::aes;

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

#[derive(Debug, Serialize, Deserialize)]
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
    source_id: String,
    s: Option<usize>,
    e: Option<usize>
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

        let keys = get_keys(&episode_info);
        

        let hls_and_track = get_hls_and_track(&episode_info, &keys);

    
        
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
            referer: "https://ww2.moviesapi.to/".to_string(),
            origin: "https://ww2.moviesapi.to".to_string(),
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


#[derive(Debug, Serialize, Deserialize)]
struct Keys{
    scrapify_url: String,
    encryption_key: String,
    player_api_key: String
}

fn get_keys(episode_info: &EpisodeInfo) -> Keys {
        let url:String;

        if episode_info.e.is_some() && episode_info.s.is_some() {
            url = format!("https://ww2.moviesapi.to/tv/{}/{}/{}",
                episode_info.tmdb_id,
                episode_info.s.unwrap()+1,
                episode_info.e.unwrap()+1
            );
        }else{
            url = format!("https://ww2.moviesapi.to/movie/{}",
                episode_info.tmdb_id
            );
        }

        let client = reqwest::blocking::Client::new();
        let mut headers = HeaderMap::new();
        
        headers.insert(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36"
        ));

        headers.insert(HOST, HeaderValue::from_static("ww2.moviesapi.to"));
        headers.insert(REFERER, HeaderValue::from_static("https://ww2.moviesapi.to/"));
        headers.insert(ORIGIN, HeaderValue::from_static("https://ww2.moviesapi.to"));

        /* Fetch Script URL */

        let res = client.get(&url)
            .headers(headers.clone()).send().unwrap();

        if !res.status().is_success(){
            panic!("Error: {}", res.status());
        }

        let html = res.text().unwrap();
        let vis = Vis::load(&html).unwrap();

        let scripts = vis.find("script");

        let mut script_url = String::new();

        
        for script in scripts {
            let script_ele = Vis::dom(&script);
            if let Some(src) = script_ele.attr("src") {
                if let Some(script_type) = script_ele.attr("type") {
                    if script_type.to_string() == "module" {
                        script_url = format!("https://ww2.moviesapi.to{}", src);
                    }
                }
            }
        }

        if script_url.is_empty() {
            panic!("Error: No `script_url` found");
        }

        /* --- */

        /* Fetch Keys From Script URL */

        let res = client.get(&script_url)
            .headers(headers.clone()).send().unwrap();

        if !res.status().is_success(){
            panic!("Error: {}", res.status());
        }
        let script_text = res.text().unwrap();


        
        let scrapify_re = Regex::new(r#"SCRAPIFY_URL:\s*"([^"]+)""#).unwrap();
        let encryption_re = Regex::new(r#"ENCRYPTION_KEY:\s*"([^"]+)""#).unwrap();
        let player_re = Regex::new(r#"PLAYER_API_KEY:\s*"([^"]+)""#).unwrap();

        
        let scrapify_url = scrapify_re
            .captures(&script_text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        let encryption_key = encryption_re
            .captures(&script_text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        let player_api_key = player_re
            .captures(&script_text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        
        let keys = Keys {
            scrapify_url,
            encryption_key,
            player_api_key,
        };
        /* --- */
        

        return keys;
}

#[derive(Debug, Serialize, Deserialize)]
struct RawEAS{
    source: String,
    id: String,
    #[serde(rename = "type")]
    _type: String,
    season: Option<usize>,
    episode: Option<usize>,
    srv: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HlsAndTrack{
    url: String,
    tracks: Vec<TrackInfo>

}


fn get_hls_and_track(episode_info: &EpisodeInfo, keys: &Keys) -> HlsAndTrack {
    let raw_eas: RawEAS;
    
    if episode_info.s.is_some() && episode_info.e.is_some() {
        raw_eas = RawEAS{
            source: episode_info.source_id.clone(),
            id: episode_info.tmdb_id.clone(),
            _type: String::from("tv"),
            season: Some(episode_info.s.unwrap()+1),
            episode: Some(episode_info.e.unwrap()+1),
            srv: String::from("0"),
        };
    }else{
        raw_eas = RawEAS{
            source: episode_info.source_id.clone(),
            id: episode_info.tmdb_id.clone(),
            _type: String::from("movie"),
            season: episode_info.s,
            episode: episode_info.e,
            srv: String::from("0"),
        };
    }
    
    let encrypted_aes = aes::encrypt(&serde_json::to_string(&raw_eas).unwrap(), &keys.encryption_key);



    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();
    
    headers.insert(USER_AGENT, HeaderValue::from_static(
        "Chrome/142.0.0.0 Safari/537.36"
    ));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
    headers.insert(HOST, HeaderValue::from_static("ww2.moviesapi.to"));
    headers.insert(REFERER, HeaderValue::from_static("https://ww2.moviesapi.to/"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://ww2.moviesapi.to"));
    headers.insert(HeaderName::from_static("x-player-key"),
        HeaderValue::from_str(&keys.player_api_key).unwrap()
    );


    let res = client.post("https://ww2.moviesapi.to/api/scrapify/v1/fetch")
        .headers(headers)
        .json(&json!({
            "payload": encrypted_aes
        }))
        .send().unwrap();

    if !res.status().is_success(){
        panic!("Error: {}", res.status());
    }

    let mut raw_data:Value = res.json().unwrap();

    for item in raw_data.get_mut("tracks").unwrap().as_array_mut().unwrap() {
        item.as_object_mut().unwrap().insert("kind".to_string(), Value::String("subtitles".to_string()));
    }

    let mut parse_data: HlsAndTrack = from_value(raw_data).unwrap();

    let parsed = Url::parse(&parse_data.url).unwrap();

    // Extract host + path
    let host = parsed.host_str().unwrap_or("");
    let path = parsed.path();

    // Build the new URL
    parse_data.url = format!("https://ax.1hd.su/{host}{path}");

    return parse_data;

}