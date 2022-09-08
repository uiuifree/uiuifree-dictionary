use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct DictionaryLocation {
    pub location_type: Option<String>,
    pub prefecture_id: Option<i32>,
    pub prefecture_name: Option<String>,
    pub major_city_id: Option<i32>,
    pub major_city_name: Option<String>,
    pub city_id: Option<i32>,
    pub city_name: Option<String>,
    pub street_address: Option<String>,
    pub station_id: Option<i32>,
    pub station_name: Option<String>,
}


impl PartialEq for DictionaryLocation {
    fn eq(&self, other: &Self) -> bool {
        self.location_type == other.location_type
            && self.prefecture_id == other.prefecture_id
            && self.city_id == other.city_id
            && self.station_id == other.station_id
            && self.street_address == other.street_address
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct DictionaryValue<T> {
    pub word: String,
    pub value: T,
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct MecabCost {
    cost: i32,
}

impl MecabCost {
    pub fn new(cost: i32) -> MecabCost {
        MecabCost { cost }
    }
    pub fn is_fix(&self) -> bool {
        -18000 >= self.cost
    }
    pub fn is_prefecture(&self) -> bool {
        -11000 >= self.cost && self.cost > -18000
    }
    pub fn is_require_prefecture(&self) -> bool {
        -11000 < self.cost
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct MecabLocation {
    pub location: DictionaryLocation,
    pub cost: MecabCost,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct DictionaryPlace {
    // 正確値
    fixed_locations: Vec<DictionaryLocation>,
    // 予測値
    predictive_locations: Vec<MecabLocation>,
    // 予測値
    low_predictive_locations: Vec<MecabLocation>,
}

impl DictionaryPlace {
    pub fn new() -> DictionaryPlace {
        DictionaryPlace {
            fixed_locations: vec![],
            predictive_locations: vec![],
            low_predictive_locations: vec![],
        }
    }
    pub fn append_predictive_location(&mut self, location: MecabLocation) {
        self.predictive_locations.push(location);
    }
    pub fn append_low_predictive_location(&mut self, location: MecabLocation) {
        self.low_predictive_locations.push(location);
    }
    pub fn fuzzy_location(&mut self) ->Vec<(i32,DictionaryLocation)>{
        let mut response = vec![];
        for location in &self.predictive_locations {
            response.push((location.cost.cost, location.location.clone()));
        }
        if response.is_empty(){

        for location in &self.low_predictive_locations {
            response.push((location.cost.cost, location.location.clone()));
        }
        }
        response
    }

    pub fn fix_location(&mut self) -> Vec<DictionaryLocation> {
        // 決まってるロケーション
        let mut fix_prefectures = vec![];
        let mut fix_cities = vec![];
        let mut fix_streets = vec![];
        let mut fix_stations = vec![];
        // めかぶデータの分類
        let mut fix_locations = vec![];
        let mut prefecture_locations = vec![];
        let mut require_prefecture_locations = vec![];
        for mecab_location in &self.predictive_locations {
            if mecab_location.cost.is_fix() {
                fix_locations.push(mecab_location.location.clone())
            } else if mecab_location.cost.is_prefecture() {
                prefecture_locations.push(mecab_location.location.clone())
            } else {
                require_prefecture_locations.push(mecab_location.location.clone())
            }
        }

        // 固定で決まってるのは「街名」と「市区町村」と「駅」
        for location in &fix_locations {
            let category = location.location_type.clone().unwrap_or_default();
            if category == "街名" {
                fix_prefectures.push(location.clone());
                fix_cities.push(location.clone());
                fix_streets.push(location.clone());
            } else if category == "市区町村" {
                fix_prefectures.push(location.clone());
                fix_cities.push(location.clone());
            } else if category == "駅" {
                fix_prefectures.push(location.clone());
                fix_stations.push(location.clone());
            }
        }
        // 都道府県確定
        for location in &prefecture_locations {
            let category = location.location_type.clone().unwrap_or_default();
            if category == "都道府県" {
                fix_prefectures.push(location.clone());
            }
        }
        let mut prefecture_ids = vec![];
        for prefecture in &fix_prefectures {
            let id = prefecture.prefecture_id.unwrap_or_default();
            if id == 0 {
                continue;
            }
            prefecture_ids.push(prefecture.prefecture_id.unwrap_or_default());
        }
        // println!("require {:?}", require_prefecture_locations);
        for location in &require_prefecture_locations {
            let prefecture_id = location.prefecture_id.unwrap_or_default();
            if !prefecture_ids.contains(&prefecture_id) {
                continue;
            }
            // println!("req--{:?}", location);
            let category = location.location_type.clone().unwrap_or_default();
            if category == "街名" {
                fix_prefectures.push(location.clone());
                fix_cities.push(location.clone());
                fix_streets.push(location.clone());
            } else if category == "市区町村" {
                fix_prefectures.push(location.clone());
                fix_cities.push(location.clone());
            } else if category == "駅" {
                fix_prefectures.push(location.clone());
                fix_stations.push(location.clone());
            }
        }
        let mut values = vec![];
        for value in fix_prefectures {
            if values.contains(&value) {
                continue;
            }
            values.push(value);
        }
        for value in fix_cities {
            if values.contains(&value) {
                continue;
            }
            values.push(value);
        }
        for value in fix_streets {
            if values.contains(&value) {
                continue;
            }
            values.push(value);
        }
        for value in fix_stations {
            if values.contains(&value) {
                continue;
            }
            values.push(value);
        }
        self.fixed_locations = values.clone();
        return values;
    }
}