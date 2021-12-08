use dirs::home_dir;
use dotenv;
use oauth2::reqwest::http_client;
use oauth2::url::Url;
use oauth2::{basic::BasicClient, TokenResponse};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    RevocationUrl, Scope, TokenUrl,
};
use std::fs::{read_to_string, write};
use std::io::{BufRead, BufReader, Error, Write};
use std::net::TcpListener;

fn get_token_from_file() -> Result<String, Error> {
    let token_filepath =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer" + "/token";
    read_to_string(&token_filepath)
}

pub fn get_token() -> String {
    // TODO add refresh flow; uncomment these lines for generating new tokens when needed
    //oauth();
    //get_token_from_file().unwrap()
    match get_token_from_file() {
        Ok(token) => token,
        _ => {
            // TODO error handling, logic for checking token is active, refresh, etc
            oauth();
            get_token_from_file().unwrap()
        }
    }
}

fn oauth() {
    dotenv::dotenv().ok();
    let google_client_id = dotenv::var("GOOGLE_CLIENT_ID").unwrap();
    println!("google_client_id: {}", google_client_id);
    let google_client_secret = dotenv::var("GOOGLE_CLIENT_SECRET").unwrap();
    println!("google_client_secret: {}", google_client_secret);

    let google_client_id = ClientId::new(google_client_id);

    let google_client_secret = ClientSecret::new(google_client_secret);

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8080".to_string()).expect("Invalid redirect URL"),
    )
    .set_revocation_uri(
        RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
            .expect("Invalid revocation endpoint URL"),
    );

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, _) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/drive.file".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/plus.me".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    println!(
        "Open this URL in your browser:\n{}\n",
        authorize_url.to_string()
    );

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code;
            {
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap();

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());
            }

            let message = "Go back to your terminal :)";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );

            stream.write_all(response.as_bytes()).unwrap();

            let token_response = client
                .exchange_code(code)
                .set_pkce_verifier(pkce_code_verifier)
                .request(http_client);

            let token_response = token_response.unwrap();
            println!("token_response: {:?}", token_response);

            let token_filepath = home_dir().unwrap().as_path().display().to_string()
                + "/.productivity-timer"
                + "/token";

            let access_token: String = format!("{:?}", token_response.access_token().secret());
            // TODO figure out a better way to store this
            write(token_filepath, access_token).expect("Problem writing to token file");

            // TODO refresh logic

            break;
        }
    }
}
