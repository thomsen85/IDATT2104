use std::{collections::HashMap, str::FromStr};

use itertools::Itertools;
use nom::{IResult, bytes::complete::{tag, take_until}, multi::separated_list1, character::complete::multispace1};

#[derive(Debug)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
}

impl FromStr for Method {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "CONNECT" => Ok(Method::CONNECT),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            _ => Err("Failed to parse method")
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl From<String> for Request {
    fn from(value: String) -> Self {
        _parse_method(&value).unwrap().1
    }
}

fn _parse_method(input: &str) -> IResult<&str, Request> {
    let (input, method) = nom::bytes::complete::take_until(" ")(input)?;
    let method = Method::from_str(method).unwrap();
    let (input, _) = tag(" ")(input)?;

    let (input, path) = nom::bytes::complete::take_until("\r\n")(input)?;
    let (_, path) = nom::bytes::complete::take_until(" ")(path)?;
    
    let (input, headers) = take_until("\r\n\r\n")(input)?;
    let (_, headers) = separated_list1(multispace1, nom::bytes::complete::take_until("\r\n"))(headers)?;

    let headers = headers.into_iter().map(|header| {
        if header.find(':').is_some() {
            let (key, value) = header.split(": ").next_tuple().unwrap();
            (key.to_string(), value.to_string())
        } else {
            (header.to_string(), "".to_string())
        }
    }).collect();

    if input.len() > 4 {
        let (input, _) = tag("\r\n\r\n")(input)?;
        return Ok((input, Request { method, path: path.to_string(), headers, body: input.to_string() }))
    }

    Ok((input, Request { method, path: path.to_string(), headers, body: input.to_string() }))
}

#[derive(Debug, Clone, Copy)]
pub enum StatusCode {
    OK = 200,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", *self as u16)?;
        match self {
            StatusCode::OK => write!(f, "OK"),
            StatusCode::BadRequest => write!(f, "Bad Request"),
            StatusCode::NotFound => write!(f, "Not Found"),
            StatusCode::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub http_version: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl Response {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_string(&self) -> String {
        let mut response = format!("{} {}\r\n", self.http_version, self.status);
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str("\r\n\r\n");
        response.push_str(&self.body);
        response
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.as_string().as_bytes().to_vec()
    }
}


impl Default for Response {
    fn default() -> Self {
        Response {
            http_version: "HTTP/1.1".to_string(),
            status: StatusCode::OK as u16,
            headers: HashMap::new(),
            body: "".to_string()
        }
    }
}