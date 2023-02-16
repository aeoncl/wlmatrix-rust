use std::path::{PathBuf, Path};

use actix_web::{get, post, web, HttpRequest, HttpResponse, HttpResponseBuilder};

use log::info;

use matrix_sdk::ruma::api::client::error::{ErrorBody, ErrorKind};
use matrix_sdk::{Client};

use regex::Regex;

use std::str::from_utf8;

use urlencoding::decode;

use http::StatusCode;
use lazy_static::lazy_static;
use matrix_sdk::ruma::{UserId, OwnedUserId};
use yaserde::de::from_str;
use yaserde::ser::to_string;

use crate::generated::ppcrl_webservice::factories::RST2ResponseFactory;
use crate::generated::ppcrl_webservice::*;
use crate::models::uuid::UUID;

use crate::utils::identifiers::{msn_addr_to_matrix_id, get_matrix_device_id};


use super::error::WebError;

lazy_static! {
    pub static ref DEFAULT_CACHE_KEY: String = String::from("12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA");
    pub static ref SHA1_REGEX: Regex = Regex::new(r"ru=([^&]*)&").unwrap();

}

lazy_static_include_bytes! {
    MSGR_CONFIG_XML => "assets/web/MsgrConfig.xml"
}

#[post("/")]
pub async fn firewall_test(request: HttpRequest) -> Result<HttpResponse, WebError> {
    return Ok(HttpResponseBuilder::new(StatusCode::OK).finish());
}

#[post("/RST2.srf")]
pub async fn rst2(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let test = std::str::from_utf8(&body).unwrap();

    let request_parsed: RST2RequestMessageSoapEnvelope = from_str(test).unwrap();
    let username_token = request_parsed.header.security.username_token.unwrap();

    let matrix_id = msn_addr_to_matrix_id(&username_token.username);
    let matrix_id_str = matrix_id.as_str();
    
    let matrix_user : OwnedUserId = UserId::parse(matrix_id_str).unwrap();

    let client = Client::builder().disable_ssl_verification().server_name(matrix_user.server_name()).build().await?;
    
    match client.login_username(matrix_id_str, username_token.password.as_str()).device_id(get_matrix_device_id().as_str()).initial_device_display_name("WLMatrix").await {
        Ok(result) => {
            let response = RST2ResponseFactory::get_rst2_success_response(
                result.access_token,
                username_token.username,
                UUID::from_string(&matrix_id),
            );
        
            
        
            let response_serialized = to_string(&response)?;
            info!("RST2 Response: {}", &response_serialized);
            return Ok(HttpResponseBuilder::new(StatusCode::OK)
                .append_header(("Content-Type", "application/soap+xml"))
                .body(response_serialized));
        },
        Err(error) => {
            log::error!("Unable to login to homeserver: {}", &error);
            if let matrix_sdk::Error::Http(err) = error {
               if let Some(test) = err.as_client_api_error(){
               if let ErrorBody::Standard { kind, message } = &test.body {
                    if &ErrorKind::Forbidden == kind {
                        return Err(WebError { message: Some(RST2ResponseFactory::get_auth_error_response()), status_code: StatusCode::INTERNAL_SERVER_ERROR });
                    }
               }
            }
        }
    }
    
    }
    return Err(WebError { message: None, status_code: StatusCode::INTERNAL_SERVER_ERROR });
}

#[get("/Config/MsgrConfig.asmx")]
pub async fn get_msgr_config() -> HttpResponse {
    let data: &'static [u8] = *MSGR_CONFIG_XML;
    return HttpResponseBuilder::new(StatusCode::OK)
        .append_header(("Content-Type", "application/soap+xml"))
        .body(data);
}

#[post("/ppsecure/sha1auth.srf")]
pub async fn sha1auth(body: web::Bytes) -> Result<HttpResponse, WebError> {
    let body = decode(from_utf8(&body)?)?.into_owned();
    let captures = SHA1_REGEX.captures(&body).unwrap();
    let redirect_url = decode(&captures[1])?.into_owned();
    info!("Redirect to {}", &redirect_url);
    return Ok(HttpResponseBuilder::new(StatusCode::FOUND).append_header(("Location", redirect_url.as_str())).finish());
}
