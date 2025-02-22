use csv::ReaderBuilder;
use rand::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone)]
pub struct Confession {
    pub selftext: String,
    pub title: String,
}

const CSV_PATH: &str = "data/inputs/confessions.csv";
const NUM_SAMPLES: usize = 100;
const APPROX_RECORDS_PER_SAMPLE: usize = 100_000;
const RECORDS_TO_TAKE: usize = 10;

fn is_valid_text(text: &str) -> bool {
    !text.is_empty() && !text.contains("[removed]") && !text.contains("[deleted]")
}

fn extract_confession_from_record(record: &csv::StringRecord) -> Option<Confession> {
    let selftext = record.get(9).unwrap_or("");
    let title = record.get(10).unwrap_or("");

    if is_valid_text(selftext) && is_valid_text(title) {
        let selftext = selftext
            .replace(r"\n", " ")
            .replace("\n", " ")
            .replace("\\", "")
            .trim()
            .to_string();
        let title = title
            .replace(r"\n", " ")
            .replace("\n", " ")
            .replace("\\", "")
            .trim()
            .to_string();

        Some(Confession {
            selftext: selftext.to_string(),
            title: title.to_string(),
        })
    } else {
        None
    }
}

pub fn read_random_valid_confession() -> Result<Confession, Box<dyn Error>> {
    let mut rng = rand::rng();

    for _ in 0..NUM_SAMPLES {
        let records_to_skip = rng.random_range(0..APPROX_RECORDS_PER_SAMPLE);

        let file = File::open(CSV_PATH)?;
        let buf_reader = BufReader::new(file);
        let mut rdr_sample = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(buf_reader);

        for _ in 0..records_to_skip {
            if rdr_sample.records().next().is_none() {
                return Err("No valid confession found (reached EOF while skipping)".into());
            }
        }

        for result in rdr_sample.records().take(RECORDS_TO_TAKE) {
            match result {
                Ok(record) => {
                    if let Some(confession) = extract_confession_from_record(&record) {
                        return Ok(confession);
                    }
                }
                Err(e) => {
                    eprintln!("CSV parsing error: {}", e);
                    continue;
                }
            }
        }
    }

    Err("No valid confession found after sampling".into())
}
