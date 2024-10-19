use crate::{BLANK_SLOT, BLANK_SLOT_USER};

fn parse_csv(data: String, delimiter: char) -> Vec<Vec<String>> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .has_headers(false)
        .flexible(true)
        .from_reader(data.as_bytes());
    let mut records: Vec<Vec<String>> = Vec::new();
    for result in reader.records() {
        let record = result.unwrap();
        let mut row: Vec<String> = Vec::new();
        for field in record.iter() {
            row.push(field.to_string());
        }
        records.push(row);
    }
    records
}

#[derive(Clone)]
pub struct GatyaEvent {
    pub index: u32,
    pub start: String,
    pub end: String,
    pub gatya_id: String,
    pub rare_chance: String,
    pub super_rare_chance: String,
    pub uber_rare_chance: String,
    pub legend_rare_chance: String,
    pub banner_txt: String,
}

fn parse_gatya_event(index: i32, line: Vec<String>) -> GatyaEvent {
    let length: i32 = line[9].parse::<i32>().unwrap();
    let gatya_id_pos: i32 = 10 + (15 * (length - 1));
    let rare_chance_pos: i32 = 16 + (15 * (length - 1));
    let super_rare_chance_pos: i32 = 18 + (15 * (length - 1));
    let uber_rare_chance_pos: i32 = 20 + (15 * (length - 1));
    let legend_rare_chance_pos: i32 = 22 + (15 * (length - 1));
    let banner_text_pos: i32 = 24 + (15 * (length - 1));

    GatyaEvent {
        index: index as u32,
        start: line[0].to_string(),
        end: line[2].to_string(),
        gatya_id: line[gatya_id_pos as usize].to_string(),
        rare_chance: line[rare_chance_pos as usize].to_string(),
        super_rare_chance: line[super_rare_chance_pos as usize].to_string(),
        uber_rare_chance: line[uber_rare_chance_pos as usize].to_string(),
        legend_rare_chance: line[legend_rare_chance_pos as usize].to_string(),
        banner_txt: line[banner_text_pos as usize].to_string(),
    }
}

pub fn parse_gatya_events(data: String) -> Vec<GatyaEvent> {
    let records: Vec<Vec<String>> = parse_csv(data, '\t');
    let mut gatya_events: Vec<GatyaEvent> = Vec::new();
    for (index, record) in records.iter().enumerate() {
        if record.len() < 25 {
            continue;
        }
        gatya_events.push(parse_gatya_event(index as i32, record.to_vec()));
    }
    gatya_events
}

pub fn get_gatya_event(data: &[GatyaEvent], gatya_id: i32) -> GatyaEvent {
    for gatya_event in data.iter() {
        if gatya_event.gatya_id.parse::<i32>().unwrap() == gatya_id {
            return gatya_event.clone();
        }
    }
    panic!("Gatya event not found");
}

pub fn get_gatya_event_from_index(data: &[GatyaEvent], index: u32) -> GatyaEvent {
    for gatya_event in data.iter() {
        if gatya_event.index == index {
            return gatya_event.clone();
        }
    }
    panic!("Gatya event not found");
}

async fn get_latest_game_data_version(cc: &str) -> String {
    let url: String =
        "https://raw.githubusercontent.com/fieryhenry/BCData/master/latest.txt".to_string();
    let client: reqwest::Client = reqwest::Client::new();
    let res: reqwest::Response = client.get(&url).send().await.unwrap();
    let body: String = res.text().await.unwrap();
    let lines: Vec<&str> = body.split('\n').collect();
    match cc {
        "en" => lines[0].to_string(),
        "jp" => lines[1].to_string(),
        "kr" => lines[2].to_string(),
        "tw" => lines[3].to_string(),
        _ => panic!("Invalid country code"),
    }
}

pub async fn get_gatya_cat_data(cc: &str, force: bool) -> Vec<Vec<i32>> {
    let file_path: String = format!("data/gatya_{}.csv", cc);
    let body: String;
    if std::path::Path::new(&file_path).exists() && !force {
        body = std::fs::read_to_string(file_path).unwrap();
    } else {
        let latest_game_data_version: String = get_latest_game_data_version(cc).await;
        let url: String = format!(
            "https://raw.githubusercontent.com/fieryhenry/BCData/master/{}/DataLocal/GatyaDataSetR1.csv",
            latest_game_data_version
        );
        let client: reqwest::Client = reqwest::Client::new();
        let res: reqwest::Response = client.get(&url).send().await.unwrap();
        body = res.text().await.unwrap();

        std::fs::write(file_path, body.clone()).unwrap();
    }
    let records: Vec<Vec<String>> = parse_csv(body, ',');
    let mut gatya_cat_data: Vec<Vec<i32>> = Vec::new();
    for record in records.iter() {
        let mut row: Vec<i32> = Vec::new();
        for field in record.iter() {
            let result = field.parse::<i32>();
            if result.is_err() {
                continue;
            }
            let value: i32 = result.unwrap();
            if value == -1 {
                break;
            }
            row.push(value);
        }
        gatya_cat_data.push(row);
    }
    gatya_cat_data
}

pub async fn get_unitbuy_cat_data(cc: &str, force: bool) -> Vec<Vec<i32>> {
    let file_path: String = format!("data/unitbuy_{}.csv", cc);
    let body: String;
    if std::path::Path::new(&file_path).exists() && !force {
        body = std::fs::read_to_string(file_path).unwrap();
    } else {
        let latest_game_data_version: String = get_latest_game_data_version(cc).await;
        let url: String = format!(
            "https://raw.githubusercontent.com/fieryhenry/BCData/master/{}/DataLocal/unitbuy.csv",
            latest_game_data_version
        );
        let client: reqwest::Client = reqwest::Client::new();
        let res: reqwest::Response = client.get(&url).send().await.unwrap();
        body = res.text().await.unwrap();

        std::fs::write(file_path, body.clone()).unwrap();
    }

    let records: Vec<Vec<String>> = parse_csv(body, ',');
    let mut unitbuy_cat_data: Vec<Vec<i32>> = Vec::new();
    for record in records.iter() {
        let mut row: Vec<i32> = Vec::new();
        for field in record.iter() {
            let resut = field.parse::<i32>();
            if resut.is_err() {
                continue;
            }
            let value = resut.unwrap();
            row.push(value);
        }
        unitbuy_cat_data.push(row);
    }
    unitbuy_cat_data
}

pub fn get_gatya_slot_data(
    gatya_id: i32,
    gatya_cat_data: Vec<Vec<i32>>,
    unit_buy_cat_data: Vec<Vec<i32>>,
) -> Vec<Vec<i32>> {
    let gatya_cat_data: Vec<i32> = gatya_cat_data[gatya_id as usize].to_vec();
    let mut gatya_slot_data: Vec<Vec<i32>> = Vec::new();
    for _ in 0..4 {
        gatya_slot_data.push(Vec::new());
    }

    for cat_id in gatya_cat_data {
        let rarity: i32 = unit_buy_cat_data[cat_id as usize][13];
        if rarity == 0 || rarity == 1 {
            continue;
        }
        if rarity == 2 {
            gatya_slot_data[0].push(cat_id);
        } else if rarity == 3 {
            gatya_slot_data[1].push(cat_id);
        } else if rarity == 4 {
            gatya_slot_data[2].push(cat_id);
        } else if rarity == 5 {
            gatya_slot_data[3].push(cat_id);
        }
    }

    gatya_slot_data
}

fn get_slot_from_id(gatya_slot_data: Vec<Vec<i32>>, cat_id: i32) -> (i32, i32) {
    for (rarity, rarity_data) in gatya_slot_data.iter().enumerate() {
        for (slot_id, slot_data) in rarity_data.iter().enumerate() {
            if *slot_data == cat_id {
                return (rarity as i32, slot_id as i32);
            }
        }
    }
    panic!("Cat ID not found in gatya slot data");
}

pub fn get_cat_list_from_ids(gatya_slot_data: Vec<Vec<i32>>, cat_ids: Vec<i32>) -> Vec<(u32, u32)> {
    let mut cat_list: Vec<(u32, u32)> = Vec::new();
    for cat_id in cat_ids.iter() {
        if *cat_id == BLANK_SLOT_USER {
            cat_list.push((BLANK_SLOT, 0))
        }
        let (rarity, slot_id) = get_slot_from_id(gatya_slot_data.clone(), *cat_id);
        cat_list.push((rarity as u32, slot_id as u32));
    }
    cat_list
}
