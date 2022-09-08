mod resource;

use mecab::Tagger;
use serde::de::DeserializeOwned;
use serde_json::Value;
use crate::resource::{MecabCost, MecabLocation, DictionaryPlace};
pub use resource::*;
pub use crate::resource::{DictionaryLocation};

pub struct Dictionary {}

impl Dictionary {
    pub fn parse_from_dic<T>(dic: &str, value: &str) -> Vec<DictionaryValue<T>>
        where T: DeserializeOwned
    {
        word_to_kconfig(dic, value)
    }
    pub fn location(place: &str, fuzzy_station: &str) -> DictionaryPlace {
        let mut mecab_locations = Dictionary::str_to_location(place);
        let mut prefecture_ids = vec![];
        for mecab_location in &mecab_locations {
            let prefecture_id = mecab_location.location.prefecture_id.unwrap_or_default();
            if prefecture_id > 0 {
                prefecture_ids.push(prefecture_id)
            }
        }
        for pref in prefecture_ids {
            let stations = Dictionary::str_to_station(pref, place, false);
            for station in stations {
                mecab_locations.push(station);
            }
            let stations = Dictionary::str_to_station(pref, fuzzy_station, true);
            for station in stations {
                mecab_locations.push(station);
            }
        }


        let mut dic_place = DictionaryPlace::new();
        for v in mecab_locations {
            dic_place.append_predictive_location(v);
        }

        let predicts = Dictionary::str_to_predict_location(place);

        for v in predicts {
            dic_place.append_low_predictive_location(v);
        }


        dic_place
    }

    pub fn str_to_location(value: &str) -> Vec<MecabLocation> {
        let locations = value_to_json(format!("-u {}dic_area.dic", dic_root()).as_str(), value);
        let mut primary = vec![];
        for location in &locations {
            if location.location.location_type == Some("街名".to_string()) {
                primary.push(location.clone());
            }
        }
        if primary.len() > 0 {
            return primary;
        }
        return locations;
    }
    pub fn str_to_predict_location(value: &str) -> Vec<MecabLocation> {
        let locations = value_to_json3(format!("-u {}dic_area_predict.dic", dic_root()).as_str(), value);

        return locations;
    }
    pub fn str_to_station(
        prefecture_id: i32,
        value: &str,
        fuzzy: bool,
    ) -> Vec<MecabLocation> {
        let mut dic = format!(
            "-u {}dic_station_prefecture_{}.dic",
            dic_root(),
            prefecture_id
        );
        if fuzzy {
            dic = format!(
                "-u {}dic_station_fuzzy_prefecture_{}.dic",
                dic_root(),
                prefecture_id
            );
        }
        value_to_json(dic.as_str(), value)
    }
}

fn dic_root() -> String {
    let value = std::env::var("UIUIFREE_DIC_ROOT");
    match value {
        Ok(v) => {
            if v.ends_with("/") {
                return v;
            }
            v + "/"
        }
        Err(_) => { "./dictionary/".to_string() }
    }
}


fn value_to_json(dic: &str, value: &str) -> Vec<MecabLocation> {
    let mut tagger = Tagger::new(dic);
    let mut node = tagger.parse_to_node(value);
    let mut rows = vec![];

    loop {
        match node.next() {
            Some(n) => {
                let json = serde_json::from_str::<DictionaryLocation>(n.feature.as_str());
                if json.is_ok() {
                    rows.push(MecabLocation {
                        location: json.unwrap(),
                        cost: MecabCost::new(n.wcost as i32),
                    })
                }
                node = n;
            }
            None => break,
        }
    }
    return rows;
}


fn value_to_json3(dic: &str, value: &str) -> Vec<MecabLocation> {
    let mut tagger = Tagger::new(dic);
    let mut node = tagger.parse_to_node(value);
    let mut rows = vec![];

    loop {
        match node.next() {
            Some(n) => {
                let json = serde_json::from_str::<Vec<DictionaryLocation>>(n.feature.as_str());
                if json.is_ok() {
                    let json = json.unwrap();
                    for value in json {
                        rows.push(MecabLocation {
                            cost: MecabCost::default(),
                            location: value,
                        });
                    }
                }
                node = n;
            }
            None => break,
        }
    }
    return rows;
}


fn word_to_kconfig<T>(dic: &str, value: &str) -> Vec<DictionaryValue<T>>
    where T: DeserializeOwned
{
    let value = kana::wide2ascii(value);
    let values = value_to_json2(dic, value.as_str());
    let mut response = vec![];
    for value in values {
        if value.value.is_none() {
            continue;
        }
        let word = value.word;
        let value = serde_json::from_value::<T>(value.value.unwrap());
        if value.is_err() {
            continue;
        }
        response.push(DictionaryValue {
            word: word,
            value: value.unwrap(),
        });
    }
    response
}

fn value_to_json2(dic: &str, value: &str) -> Vec<MecabNode> {
    let tagger = Tagger::new(format!("-u {}", dic));

    let result = tagger.parse_str(value);
    let rows: Vec<&str> = result.split("\n").collect();

    let mut node = Vec::new();

    for row in rows {
        let line: Vec<&str> = row.trim().split("\t").collect();
        if line.len() < 2 {
            continue;
        }
        let word = line[0].to_string();
        let data = line[1].to_string();
        let json = serde_json::from_str::<Value>(data.as_str());
        if json.is_ok() {
            node.push(MecabNode {
                word,
                value: Some(json.unwrap()),
            });
        }
    }
    return node;
}

#[derive(Default, Debug)]
struct MecabNode {
    pub word: String,
    pub value: Option<Value>,
}
