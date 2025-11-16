

mod tests {
    use std::ffi::{c_char, CString, CStr};
    use serde_json::{json, to_string};
    use crate::free_ptr::free_ptr;

    
    // #[test]
    // fn test_2() {
    //     use crate::search::search;
    //     unsafe {
    //         let args = CString::new(to_string(&json!({
    //             "search": String::from("loki")
    //         })).unwrap()).expect("CString::new failed");
            
    //         let search_ptr = search(args.as_ptr());
    //         if search_ptr.is_null() {
    //             println!("[Search] Error null ptr.");
    //             return;
    //         }
    //         let result = CStr::from_ptr(search_ptr).to_str().unwrap().to_owned();

    //         println!("{:?}", result);

    //         free_ptr(search_ptr as *mut c_char);
    //     }
    // }

    // use crate::get_episode_list::get_episode_list;
    // #[test]
    // fn test_3() {
    //     unsafe {
    //         let args = CString::new(to_string(&json!({
    //             "id": "%2Ftv%2F1074318%2Floki",

    //         })).unwrap()).expect("CString::new failed");
    //         let get_episode_ptr = get_episode_list(args.as_ptr());
    //         let result = CStr::from_ptr(get_episode_ptr).to_str().unwrap().to_owned();
    //         println!("{}", &result);
    //         free_ptr(get_episode_ptr as *mut c_char);
    //     }
    // }


    

    // #[test]
    // fn test_4() {
    //     use crate::get_episode_server::get_episode_server;
    //     unsafe {
    //         let args = CString::new(to_string(&json!({
    //             "episode_id": "%7B%22e%22%3A0%2C%22id_type%22%3A%22tv%22%2C%22imdb_id%22%3A%22tt9140554%22%2C%22s%22%3A0%2C%22tmdb_id%22%3A%2284958%22%7D",
    //         })).unwrap()).expect("CString::new failed");
    //         let get_episode_server_ptr = get_episode_server(args.as_ptr());
    //         let result = CStr::from_ptr(get_episode_server_ptr).to_str().unwrap().to_owned();
    //         println!("{}", &result);
    //         free_ptr(get_episode_server_ptr as *mut c_char);
    //     }
    // }

    

    #[test]
    fn test_get_server() {
        use crate::get_server::get_server;
        unsafe {
            let args = CString::new(to_string(&json!({
                "index": 0,
                "id": String::from("%7B%22e%22%3A0%2C%22id_type%22%3A%22tv%22%2C%22imdb_id%22%3A%22tt9140554%22%2C%22s%22%3A0%2C%22tmdb_id%22%3A%2284958%22%7D"),
            })).unwrap()).expect("CString::new failed");
            
            let get_server_ptr = get_server(args.as_ptr());
            let result = CStr::from_ptr(get_server_ptr).to_str().unwrap().to_owned();
            println!("{}", &result);
            free_ptr(get_server_ptr as *mut c_char);
        }
    }
}