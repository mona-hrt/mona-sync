use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
pub struct SyncQuery {
    pub last_sync: i64,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub struct SupplyItem {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub total_dose: Option<String>,
    pub used_dose: Option<String>,
    pub concentration: Option<String>,
    pub molecule_json: Option<String>,
    pub administration_route_name: Option<String>,
    pub ester_name: Option<String>,
    pub amount: Option<i64>,
    pub updated_at: i64,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub struct MedicationIntake {
    pub id: String,
    pub scheduled_date_time: String,
    pub taken_date_time: Option<String>,
    pub taken_time_zone: Option<String>,
    pub dose: String,
    pub schedule_id: Option<String>,
    pub side: Option<String>,
    pub molecule_json: String,
    pub administration_route_name: String,
    pub ester_name: Option<String>,
    pub supply_item_id: Option<String>,
    pub notes: Option<String>,
    pub updated_at: i64,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub struct MedicationSchedule {
    pub id: String,
    pub name: String,
    pub dose: String,
    pub interval_days: i64,
    pub start_date: String,
    pub molecule_json: String,
    pub administration_route_name: String,
    pub ester_name: Option<String>,
    pub notification_times: String,
    pub updated_at: i64,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub struct BloodTest {
    pub id: String,
    pub date_time: String,
    pub time_zone: String,
    pub estradiol_levels: Option<String>,
    pub testosterone_levels: Option<String>,
    pub estradiol_unit: Option<String>,
    pub testosterone_unit: Option<String>,
    pub updated_at: i64,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub is_deleted: bool,
}

pub fn deserialize_bool_from_anything<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrInt {
        Bool(bool),
        Int(i64),
    }

    match BoolOrInt::deserialize(deserializer)? {
        BoolOrInt::Bool(b) => Ok(b),
        BoolOrInt::Int(i) => Ok(i != 0),
    }
}
