mod event_data;
mod gatya_data;

use std::{io::Write, time::Instant};

async fn get_event_data(cc: &str, force: bool) -> String {
    let file_path: String = format!("data/gatya_{}.tsv", cc);
    if std::path::Path::new(&file_path).exists() && !force {
        let data: String = std::fs::read_to_string(file_path).unwrap();
        return data;
    }
    let data = event_data::get_event_data(cc).await;
    std::fs::write(file_path, data.clone()).unwrap();
    data
}

fn get_int_from_user(prompt: &str, default: Option<i32>) -> i32 {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();

    let mut input: String = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let input: i32 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => match default {
            Some(d) => return d,
            None => {
                println!("Invalid input. Try again.");
                return get_int_from_user(prompt, default);
            }
        },
    };
    input
}

fn ask_if_want_to_update_data() -> bool {
    let input: i32 = get_int_from_user("Update Game Data? (1 for yes, 2 for no): ", None);
    match input {
        1 => true,
        2 => false,
        _ => {
            println!("Invalid input. Try again.");
            ask_if_want_to_update_data()
        }
    }
}

async fn select_event(cc: &str) -> (gatya_data::GatyaEvent, bool) {
    std::fs::create_dir_all("data").unwrap();

    let force: bool = ask_if_want_to_update_data();

    println!("Getting event data...");

    let data: String = get_event_data(cc, force).await;
    let gatya_events: Vec<gatya_data::GatyaEvent> = gatya_data::parse_gatya_events(data);
    let valid_events: Vec<&gatya_data::GatyaEvent> = gatya_events
        .iter()
        .filter(|gatya_event| !gatya_event.banner_txt.is_empty())
        .collect();
    for (i, gatya_event) in valid_events.iter().enumerate() {
        let start_time: String = gatya_event.start.clone();
        let end_time: String = gatya_event.end.clone();

        let start_time_parsed: String = format!(
            "{}-{}-{}",
            &start_time[0..4],
            &start_time[4..6],
            &start_time[6..8]
        );
        let end_time_parsed: String = format!(
            "{}-{}-{}",
            &end_time[0..4],
            &end_time[4..6],
            &end_time[6..8]
        );
        println!(
            "{}. {} - {}: {}",
            i + 1,
            start_time_parsed,
            end_time_parsed,
            gatya_event.banner_txt
        );
    }
    let mut input: i32;
    loop {
        input = get_int_from_user("Select event: ", None);
        if input < 1 || input > valid_events.len() as i32 {
            println!("Invalid input. Try again.");
            continue;
        }
        break;
    }

    let gatya_event: &gatya_data::GatyaEvent = valid_events[(input - 1) as usize];
    println!("Selected event: {}", gatya_event.banner_txt);

    (gatya_event.clone(), force)
}

fn select_cc() -> String {
    println!("1. English");
    println!("2. Japanese");
    println!("3. Korean");
    println!("4. Taiwanese");
    // validate input
    let input: i32 = get_int_from_user("Select country code: ", None);
    match input {
        1 => "en".to_string(),
        2 => "jp".to_string(),
        3 => "kr".to_string(),
        4 => "tw".to_string(),
        _ => {
            println!("Invalid input. Try again.");
            select_cc()
        }
    }
}

fn select_cats() -> Vec<i32> {
    let mut cats_ids: Vec<i32> = Vec::new();
    let mut counter: u32 = 0;
    loop {
        let cat_id: i32 = get_int_from_user(
            &format!(
                "ID for cat {} (-1 to stop, {} for blank): ",
                counter + 1,
                BLANK_SLOT_USER,
            ),
            None,
        );
        if cat_id == -1 {
            break;
        }
        cats_ids.push(cat_id as i32);
        counter += 1;
    }

    if cats_ids.is_empty() {
        println!("No cats entered. Try again.");
        return select_cats();
    }

    cats_ids
}

fn select_rarities() -> Vec<i32> {
    let mut rarities: Vec<i32> = Vec::new();
    let mut counter: u32 = 0;
    println!("Rarities:");
    println!("1. Rare");
    println!("2. Super Rare");
    println!("3. Uber Rare");
    println!("4. Legend Rare");
    loop {
        let rarity: i32 = get_int_from_user(
            &format!(
                "Rarity for cat {} (-1 to stop, {} for blank): ",
                counter + 1,
                BLANK_SLOT_USER,
            ),
            None,
        );
        if rarity == -1 {
            break;
        }
        if rarity == BLANK_SLOT_USER {
            rarities.push(BLANK_SLOT as i32);
            counter += 1;
            continue;
        }
        rarities.push(rarity - 1_i32);
        counter += 1;
    }

    if rarities.is_empty() {
        println!("No rarities entered. Try again.");
        return select_rarities();
    }

    rarities
}

fn get_cat_slots(gatya_slot_data: Vec<Vec<i32>>, total_rares: u32) -> Vec<(u32, u32)> {
    let cats_ids: Vec<i32> = select_cats();

    //let cats_ids: &[i32] = &[308, 50, 145, 37, 38, 35, 51, 308, 51, 150];
    let cats: Vec<(u32, u32)> =
        gatya_data::get_cat_list_from_ids(gatya_slot_data, cats_ids.to_vec());

    let collisions: bool = is_collisions(cats.clone(), total_rares);

    if collisions {
        println!("WARNING: There might be a duplicate rare cat! The seed might not be found.")
    }
    cats
}

#[tokio::main]
async fn main() {
    let cc: &str = &select_cc();
    println!();
    let (gatya_event, force) = select_event(cc).await;
    let unitbuy_cat_data: Vec<Vec<i32>> = gatya_data::get_unitbuy_cat_data(cc, force).await;

    let gatya_cat_data: Vec<Vec<i32>> = gatya_data::get_gatya_cat_data(cc, force).await;

    let gatya_id: i32 = gatya_event.gatya_id.parse::<i32>().unwrap();

    let gatya_slot_data: Vec<Vec<i32>> =
        gatya_data::get_gatya_slot_data(gatya_id, gatya_cat_data, unitbuy_cat_data);

    let total_rares: u32 = gatya_slot_data[0].len() as u32;
    let total_super_rares: u32 = gatya_slot_data[1].len() as u32;
    let total_uber_rares: u32 = gatya_slot_data[2].len() as u32;
    let total_legend_rares: u32 = gatya_slot_data[3].len() as u32;

    let legend_chance: u32 = 10000 - gatya_event.legend_rare_chance.parse::<u32>().unwrap();
    let uber_chance: u32 = legend_chance - gatya_event.uber_rare_chance.parse::<u32>().unwrap();
    let super_rare_chance: u32 =
        uber_chance - gatya_event.super_rare_chance.parse::<u32>().unwrap();

    println!();

    let seek_or_find: i32 = get_int_from_user(
        "1. Find seed by cats\n2. Seek seed by rarities\nEnter choice: ",
        None,
    );
    let mut cats: Vec<(u32, u32)> = Vec::new();
    if seek_or_find == 1 {
        cats = get_cat_slots(gatya_slot_data.clone(), total_rares);
    } else {
        let rarities: Vec<i32> = select_rarities();
        for rarity in rarities.iter() {
            cats.push((*rarity as u32, IGNORE_SLOT));
        }
    }
    let slice_cats: &[(u32, u32)] = cats.as_slice();

    let thread_count: i32 = get_int_from_user("Enter total threads to use (default 8):", Some(8));

    println!("\nFinding seed...");
    let start: Instant = Instant::now();
    let seeds: Vec<u32> = find_seed(
        slice_cats,
        total_rares,
        total_super_rares,
        total_uber_rares,
        total_legend_rares,
        legend_chance,
        uber_chance,
        super_rare_chance,
        thread_count.try_into().unwrap(),
    );
    let duration: std::time::Duration = start.elapsed();

    println!();

    if seeds.is_empty() {
        println!("Seed not found. Try again.");
    } else if seeds.len() == 1 {
        println!("Seed: {}", seeds[0]);
    } else {
        println!("Multiple seeds found. You need to enter more cats!");
        println!("\nSeeds: ");
        let max_seeds: usize = if seeds.len() > 10 { 10 } else { seeds.len() };
        for seed in seeds[0..max_seeds].iter() {
            println!("{}", seed);
        }
        if max_seeds < seeds.len() {
            println!("... and {} more", seeds.len() - max_seeds);
        }
    }
    println!("\nTime taken to find seed: {:?}", duration);
}

fn is_collisions(cats: Vec<(u32, u32)>, total_rares: u32) -> bool {
    for i in 0..cats.len() - 1 {
        let current_rarity: u32 = cats[i].0;
        let current_slot_code: u32 = cats[i].1;

        if current_rarity != 0 {
            continue;
        }

        let next_rarity: u32 = cats[i + 1].0;
        let next_slot_code: u32 = cats[i + 1].1;

        if (current_rarity == next_rarity)
            && (next_slot_code == (current_slot_code + 1) % total_rares)
        {
            return true;
        }
    }
    false
}

fn find_seed(
    cats: &[(u32, u32)],
    total_rares: u32,
    total_super_rares: u32,
    total_uber_rares: u32,
    total_legend_rares: u32,
    legend_chance: u32,
    uber_chance: u32,
    super_rare_chance: u32,
    total_threads: u32,
) -> Vec<u32> {
    let mut threads: Vec<std::thread::JoinHandle<Vec<u32>>> = Vec::new();
    let mut start_point: u32 = 1;
    let step: u32 = 0xFFFFFFFF / total_threads;

    let mut end_point: u32 = step;
    for i in 0..total_threads {
        let cats: Vec<(u32, u32)> = cats.to_vec();
        threads.push(std::thread::spawn(move || {
            find_seed_range(
                &cats,
                total_rares,
                total_super_rares,
                total_uber_rares,
                total_legend_rares,
                legend_chance,
                uber_chance,
                super_rare_chance,
                start_point,
                end_point,
            )
        }));
        if i == total_threads - 1 {
            break;
        }
        start_point = end_point + 1;
        end_point += step;
    }
    let mut seeds: Vec<u32> = Vec::new();
    for thread in threads {
        let mut thread_seeds: Vec<u32> = thread.join().unwrap();
        seeds.append(&mut thread_seeds);
    }
    seeds
}
const MODULUS: u32 = 10000;
const BLANK_SLOT: u32 = 20;
const IGNORE_SLOT: u32 = 21;
const BLANK_SLOT_USER: i32 = -2;

fn find_seed_range(
    cats: &[(u32, u32)],
    total_rares: u32,
    total_super_rares: u32,
    total_uber_rares: u32,
    total_legend_rares: u32,
    legend_chance: u32,
    uber_chance: u32,
    super_rare_chance: u32,
    start_point: u32,
    end_point: u32,
) -> Vec<u32> {
    let last_cat: usize = cats.len() - 1;

    let mut slot: u32;
    let mut size: u32;

    let mut seed: u32;
    let mut prob: u32;

    // 0 - Rare
    // 1 - Super Rare
    // 2 - Uber Rare
    // 3 - Legend Rare
    let mut seeds: Vec<u32> = Vec::new();

    for i in start_point..end_point {
        seed = i;
        for (j, cat) in cats.iter().enumerate() {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 15;
            prob = seed % MODULUS;

            if cat.0 != BLANK_SLOT {
                if prob < super_rare_chance {
                    if cat.0 != 0 {
                        break;
                    }
                    size = total_rares;
                } else if prob < uber_chance {
                    if cat.0 != 1 {
                        break;
                    }
                    size = total_super_rares;
                } else if prob < legend_chance {
                    if cat.0 != 2 {
                        break;
                    }
                    size = total_uber_rares;
                } else {
                    if cat.0 != 3 {
                        break;
                    }
                    size = total_legend_rares
                }
            } else {
                size = 1;
            }

            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 15;
            slot = seed % size;
            if slot != cat.1 && cat.0 != BLANK_SLOT && cat.1 != IGNORE_SLOT {
                break;
            }
            if j == last_cat {
                seeds.push(i);
            }
        }
    }
    seeds
}
