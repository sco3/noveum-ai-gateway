use aws_sigv4::http_request::{SigningSettings, SignableBody, SignableRequest};
use aws_credential_types::Credentials;
use aws_sigv4::sign::v4;
use axum::http::{HeaderMap, Request};
use std::time::SystemTime;
use crate::error::AppError;
use tracing::debug;
use chrono::{DateTime, Utc}; 

pub async fn sign_aws_request(
    method: &str,
    url: &str,
    headers: &HeaderMap,
    body: &[u8],
    access_key: &str,
    secret_key: &str,
    region: &str,
    service: &str,
) -> Result<HeaderMap, AppError> {
    debug!("Signing request with method: {}, url: {}, region: {}, service: {}", 
           method, url, region, service);

    let credentials = Credentials::new(
        access_key,
        secret_key,
        None,
        None,
        "signing-credentials",
    );

    let host = format!("{}.{}.amazonaws.com", service, region);
    debug!("Using host: {}", host);
    
    let mut headers_to_sign = vec![
        ("host", host.as_str()),
        ("content-type", "application/json"),
        ("x-amz-target", "bedrock-runtime.InvokeModel"),
    ];

    let now = SystemTime::now();
    let datetime = DateTime::<Utc>::from(now);
    let formatted_date = datetime.format("%Y%m%dT%H%M%SZ").to_string();
    headers_to_sign.push(("x-amz-date", &formatted_date));
    
    debug!("Headers to sign: {:?}", headers_to_sign);

    let signing_settings = SigningSettings::default();
    let identity = credentials.into();
    
    let signing_params = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name(service)
        .time(now)
        .settings(signing_settings)
        .build()
        .map_err(|e| AppError::AwsParamsError(e.to_string()))?
        .into();

    let signable_request = SignableRequest::new(
        method,
        url,
        headers_to_sign.into_iter(),
        SignableBody::Bytes(body),
    ).map_err(|e| AppError::AwsSigningError(e))?;

    let (signing_instructions, signature) = aws_sigv4::http_request::sign(signable_request, &signing_params)
        .map_err(|e| AppError::AwsSigningError(e))?
        .into_parts();

    debug!("Generated signature: {}", signature);

    let mut final_headers = HeaderMap::new();
    let mut temp_request = Request::builder()
        .method(method)
        .uri(url)
        .header("host", &host)
        .header("content-type", "application/json")
        .header("x-amz-target", "bedrock-runtime.InvokeModel")
        .body(())
        .unwrap();
        
    signing_instructions.apply_to_request_http1x(&mut temp_request);
    
    for (key, value) in temp_request.headers() {
        final_headers.insert(key.clone(), value.clone());
    }

    debug!("Final signed headers: {:?}", final_headers);
    Ok(final_headers)
} 