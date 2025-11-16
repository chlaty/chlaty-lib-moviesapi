

pub const SOURCE_HOST: &str = "simkl.com";
pub const SOURCE_REFERER: &str = "https://simkl.com/";
pub const SOURCE_ORIGIN: &str = "https://simkl.com";

pub const SERVER_ORIGIN: &str = "https://simkl.com";
pub const SERVER_REFERER: &str = "https://simkl.com/";
pub const SERVER_HOST: &str = "simkl.com";

pub mod search;
pub mod get_episode_list;
pub mod get_episode_server;
pub mod get_server;
pub mod free_ptr;

#[cfg(test)]
mod test;