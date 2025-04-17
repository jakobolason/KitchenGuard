
pub struct WebServer {
    elder_uids: Vec<i32>,
    access_token: String,
}

impl WebServer {
    // given a valid token, html with information from state server
    // should be provided
    pub fn get_info(token: String) {

    }
// NOTE: This could be the constructor for the WebServer 'class'
    // ask StateServer for password token with given credentials
    pub fn user_check(username: String, password: String) -> String {

        // alter access_token and elder_uids from response
        "yes".to_string()
    }
}