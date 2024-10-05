use chrono::DateTime;
use chrono::Utc;
use gethostname::gethostname;
use serde::Serialize;
use serde::Serializer;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error;

pub const DEFAULT_VERSION: i8 = 1;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}

pub struct Config {
    version: i8,
    app_name: String,
    file_path: String,
}
impl Config {
    pub fn new(self, app_name: &str, file_path: &str, version: Option<i8>) -> Self {
        Config {
            app_name: app_name.to_string(),
            version: version.unwrap_or(DEFAULT_VERSION),
            file_path: file_path.to_string(),
        }
    }
}
#[derive(Serialize)]
struct Record {
    #[serde(rename = "@timestamp", serialize_with = "zulu_serializer")]
    timestamp: DateTime<Utc>,
    #[serde(rename = "@version")]
    version: i8,
    host: String,
    level: Level,
    message: String,
    #[serde(rename = "@fields", skip_serializing_if = "Option::is_none")]
    extra_fields: Option<serde_json::Value>,
    #[serde(rename = "type")]
    type_field: String,
}

impl Record {
    fn new(
        version: i8,
        level: Level,
        message: &str,
        extra_fields_as_json: Option<&str>,
        type_field: &str,
    ) -> Result<Self, String> {
        let extra_fields_value = if !extra_fields_as_json.as_slice().is_empty() {
            Some(
                serde_json::from_str::<serde_json::Value>(extra_fields_as_json.expect("error"))
                    .expect("error.."),
            )
        } else {
            None
        };
        Ok(Record {
            timestamp: chrono::Utc::now(),
            version,
            host: gethostname().to_str().unwrap().to_string(),
            level,
            type_field: type_field.to_string(),
            message: message.to_string(),
            extra_fields: extra_fields_value,
        })
    }
}

struct Logger {
    config: Config,
}

impl Logger {
    fn log_extra(&self, level: Level, message: &str, extra_fields: &str) {
        let record = Record::new(
            self.config.version,
            level,
            message,
            Some(extra_fields),
            self.config.app_name.as_str(),
        );
        write_to_file(
            self.config.file_path.as_str(),
            serde_json::to_string(&record).unwrap().as_bytes(),
        );
    }
    fn log(&self, level: Level, message: &str) {
        let record = Record::new(
            self.config.version,
            level,
            message,
            None,
            self.config.app_name.as_str(),
        );
        write_to_file(
            self.config.file_path.as_str(),
            serde_json::to_string(&record).unwrap().as_bytes(),
        );
    }

    pub fn info_extra(&self, message: &str, extra_fields: &str) {
        Self::log_extra(&self, Level::Info, message, extra_fields)
    }
    pub fn info(&self, message: &str) {
        Self::log(&self, Level::Info, message)
    }
}

fn write_to_file(file_name: &str, data: &[u8]) -> Result<(), Error> {
    let mut file = match OpenOptions::new().write(true).create(true).open(file_name) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error while opening file ({})", e);
            return Err(e);
        }
    };

    match file.write_all(data) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error while writing to file ({})", e);
            return Err(e);
        }
    }
}

fn zulu_serializer<S>(datetime: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted_date = datetime.to_rfc3339();
    serializer.serialize_str(&formatted_date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]

    fn it_logs() {
        let logger = Logger {
            config: Config {
                version: 1,
                app_name: "my_app".to_string(),
                file_path: "/home/medunes/logstash.log".to_string(),
            },
        };
        logger.info("test message");
        logger.info_extra("test123", "{\"field\":\"yesss\"}");
        logger.info_extra("message", &json!({"user_id": 1}).to_string());
    }
}
