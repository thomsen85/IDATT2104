use std::{collections::HashMap, str::FromStr};

use itertools::Itertools;
use nom::{IResult, bytes::complete::tag, multi::separated_list1, character::complete::multispace1};

#[derive(Debug)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTION,
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
            "OPTION" => Ok(Method::OPTION),
            "TRACE" => Ok(Method::TRACE),
            _ => Err("Failed to parse method")
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>
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

    let (input, path) = nom::bytes::complete::take_until("\n")(input)?;
    let (_, path) = nom::bytes::complete::take_until(" ")(path)?;
    let (input, headers) = separated_list1(multispace1, nom::bytes::complete::take_until("\n"))(input)?;
    //let (input, headers) = separated_list1(tag("\n"), nom::bytes::complete::take_till(|c| c == ':' || c == '\n'))(input)?;
    let headers = headers.into_iter().map(|header| {
        if header.find(':').is_some() {
            let (key, value) = header.split(": ").next_tuple().unwrap();
            (key.to_string(), value.to_string())
        } else {
            (header.to_string(), "".to_string())
        }
    }).collect();

    Ok((input, Request { method, path: path.to_string(), headers }))
}

