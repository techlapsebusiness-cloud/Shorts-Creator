#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::collections::VecDeque;
use std::time::Duration;

use crate::error::AiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

#[derive(Clone)]
#[cfg(test)]
pub struct Seen {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

pub trait Transport {
    fn request(
        &self,
        method: Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&str>,
    ) -> Result<String, AiError>;
}

#[cfg(test)]
pub struct Scripted {
    bodies: RefCell<VecDeque<String>>,
    seen: RefCell<Vec<Seen>>,
}

#[cfg(test)]
impl Scripted {
    pub fn new(bodies: Vec<String>) -> Self {
        Self {
            bodies: RefCell::new(bodies.into()),
            seen: RefCell::new(Vec::new()),
        }
    }

    pub fn seen(&self) -> Vec<Seen> {
        self.seen.borrow().clone()
    }
}

#[cfg(test)]
impl Transport for Scripted {
    fn request(
        &self,
        method: Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&str>,
    ) -> Result<String, AiError> {
        self.seen.borrow_mut().push(Seen {
            method,
            url: url.to_string(),
            headers: headers
                .iter()
                .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
                .collect(),
            body: body.map(str::to_string),
        });
        self.bodies.borrow_mut().pop_front().ok_or(AiError::Empty)
    }
}

pub struct ReqwestTransport {
    client: reqwest::blocking::Client,
}

impl ReqwestTransport {
    pub fn new() -> Result<Self, AiError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| AiError::Http(0, error.to_string()))?;
        Ok(Self { client })
    }
}

impl Transport for ReqwestTransport {
    fn request(
        &self,
        method: Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&str>,
    ) -> Result<String, AiError> {
        let verb = match method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
        };
        let mut request = self.client.request(verb, url);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        if let Some(body) = body {
            request = request
                .header("content-type", "application/json")
                .body(body.to_string());
        }
        let response = request
            .send()
            .map_err(|error| AiError::Http(0, error.to_string()))?;
        let status = response.status();
        let text = response
            .text()
            .map_err(|error| AiError::Http(status.as_u16(), error.to_string()))?;
        if !status.is_success() {
            return Err(AiError::Http(status.as_u16(), text));
        }
        Ok(text)
    }
}
