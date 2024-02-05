use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use std::io::prelude::*;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize)]
struct Position {
    lat: f64,
    lng: f64,
    alt: f64,
}

struct KomootResponse {
    name: String,
    coordinates: Vec<Position>,
}

fn request_komoot(url: &str) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".parse().unwrap());
    headers.insert("sec-ch-ua", "\" Not A;Brand\";v=\"99\", \"Chromium\";v=\"100\", \"Google Chrome\";v=\"100\"".parse().unwrap());
    headers.insert("sec-ch-ua-platform", "\"macOS\"".parse().unwrap());
    headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    headers.insert("accept-language", "es-ES,es;q=0.9,en;q=0.8,ca;q=0.7".parse().unwrap());
    headers.insert("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15".parse().unwrap());

    let request = client.request(reqwest::Method::GET, url)
        .headers(headers);

    let response = request.send()?;
    println!("Recieved response: {:?}", response.status());
    let body = response.text()?;

    Ok(body)
}

fn parse_komoot(body: String) -> KomootResponse {

    let re = regex::Regex::new(r"kmtBoot\.setProps\((.*)\)").unwrap();
    let captures = re.captures(&body).unwrap();
    let json_text = captures.get(1).unwrap().as_str().replace("\\\"", "\"").replace("\\\"", "\"");
    let json_text = json_text.trim_start_matches("\"").trim_end_matches("\"");

    let json: serde_json::Value = serde_json::from_str(&json_text).unwrap();

    // coordinates
    let coordinates= json["page"]["_embedded"]["tour"]["_embedded"]["coordinates"]["items"].as_array().unwrap();
    let coordinates: Vec<Position> = coordinates.iter().map(|c| {
        let lat = c["lat"].as_f64().unwrap();
        let lng = c["lng"].as_f64().unwrap();
        let alt = c["alt"].as_f64().unwrap();
        Position { lat, lng, alt }
    }).collect();

    // name
    let name = json["page"]["_embedded"]["tour"]["name"].as_str().unwrap().to_string();

    KomootResponse {
        name,
        coordinates,
    }
}

fn build_gpx(komoot: &KomootResponse) -> String {
    let mut gpx = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    gpx.push_str("<gpx creator=\"Felix's Komoot\" version=\"1.1\" xsi:schemaLocation=\"http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/11.xsd\" xmlns:ns3=\"http://www.garmin.com/xmlschemas/TrackPointExtension/v1\" xmlns=\"http://www.topografix.com/GPX/1/1\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:ns2=\"http://www.garmin.com/xmlschemas/GpxExtensions/v3\">\n");
    gpx.push_str("<trk>\n");
    gpx.push_str(format!("<name>{}</name>\n", komoot.name).as_str());
    gpx.push_str("<trkseg>\n");

    for c in &komoot.coordinates {
        gpx.push_str(&format!("<trkpt lat=\"{}\" lon=\"{}\">\n", c.lat, c.lng));
        gpx.push_str(&format!("<ele>{}</ele>\n", c.alt));
        gpx.push_str("</trkpt>\n");
    }

    gpx.push_str("</trkseg>\n");
    gpx.push_str("</trk>\n");
    gpx.push_str("</gpx>\n");

    gpx
}

fn save_to_file(gpx: String, komoot: KomootResponse) {
    let mut file = File::create(komoot.name + ".gpx").unwrap();
    file.write_all(gpx.as_bytes()).unwrap();
}

fn komoot_to_file(url: &str) -> Result<(), reqwest::Error> {
    let raw = request_komoot(url)?;

    let data = parse_komoot(raw);
    let gpx = build_gpx(&data);
    save_to_file(gpx, data);

    Ok(())
}


fn main() {
    let komoot_url = std::env::args().nth(1).expect("no komoot_url given");
    println!("Downloading GPX from: {:?}", komoot_url);

    let res = komoot_to_file(komoot_url.as_str());
    match res {
        Ok(_) => println!("Downloaded GPX"),
        Err(e) => println!("Error: {:?}", e),
    }
}
