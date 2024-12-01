use jsonschema::ValidationError;
use serde_json::Map;
use serde_json::{Number, Value};
use std::fs;
use std::path::Path;
use yaml_rust2::yaml::Hash;
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};

/// Wrapper for yaml_rust2::Yaml type. This is to be able to implement external traits.
pub struct YamlType(pub Yaml);
/// Wrapper for serde_json::Value type. This is to be able to implement external traits.
pub struct JsonType(pub Value);

impl From<YamlType> for JsonType {
    /// Converts a YamlType into a JsonType
    fn from(value: YamlType) -> Self {
        match value.0 {
            Yaml::Null => JsonType(Value::Null),
            Yaml::String(str) => JsonType(Value::String(str)),
            Yaml::Boolean(b) => JsonType(Value::Bool(b)),
            Yaml::Integer(i) => JsonType(Value::Number(i.into())),
            Yaml::Real(r) => {
                let real: f64 = r.parse().expect("could not parse float");
                let num = Number::from_f64(real).expect("could not convert from f64 to number");
                JsonType(Value::Number(num))
            }
            Yaml::Array(arr) => {
                let mut new_arr: Vec<Value> = Vec::new();
                for v in arr.into_iter() {
                    let json_type = JsonType::from(YamlType(v));
                    new_arr.push(json_type.0);
                }
                JsonType(Value::Array(new_arr))
            }
            Yaml::Hash(h) => {
                let mut obj = Map::new();
                for (key, val) in h.into_iter() {
                    if let Some(key_as_string) = key.as_str() {
                        let json_type = JsonType::from(YamlType(val));
                        obj.insert(key_as_string.to_owned(), json_type.0);
                    }
                }
                JsonType(Value::Object(obj))
            }
            _ => JsonType(Value::Null),
        }
    }
}

impl From<JsonType> for YamlType {
    /// Converts a JsonType into a YamlType
    fn from(value: JsonType) -> Self {
        match value.0 {
            Value::Null => YamlType(Yaml::Null),
            Value::String(str) => YamlType(Yaml::String(str)),
            Value::Bool(b) => YamlType(Yaml::Boolean(b)),
            Value::Number(i) => {
                if i.is_i64() || i.is_u64() {
                    let int = i.as_i64().expect("could not convert to i64");
                    YamlType(Yaml::Integer(int))
                } else {
                    YamlType(Yaml::Real(i.to_string()))
                }
            }
            Value::Array(arr) => {
                let mut new_arr: Vec<Yaml> = Vec::new();
                for v in arr.into_iter() {
                    let yaml_type = YamlType::from(JsonType(v));
                    new_arr.push(yaml_type.0);
                }
                YamlType(Yaml::Array(new_arr))
            }
            Value::Object(h) => {
                let mut obj = Hash::new();
                for (key, val) in h.into_iter() {
                    let yaml_type = YamlType::from(JsonType(val));
                    obj.insert(Yaml::from_str(&key), yaml_type.0);
                }
                YamlType(Yaml::Hash(obj))
            }
        }
    }
}

struct StackItem {
    pub path: Vec<String>,
    pub value: Value,
}

/// Represents the contents of a file from a supported data format (yaml, json or toml)
pub struct File {
    /// Contents of the file. For max compatibility with serde, the contents is always stored as
    /// serde_json::Value
    data: Value,
}



impl File {
    pub fn new(data: Value) -> Self {
        File { data }
    }
    pub fn data(&self) -> &Value {
        &self.data
    }
    pub fn from_json(path: &str) -> Self {
        let content = fs::read_to_string(path).expect("Should have been able to read the file");
        let json: Value = serde_json::from_str(&content).unwrap();
        File::new(json)
    }
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Self {
        let content = fs::read_to_string(path).expect("Should have been able to read the file");
        let yamls = YamlLoader::load_from_str(&content).unwrap();
        let yaml = YamlType(yamls[0].clone());
        let json: JsonType = yaml.into();
        File::new(json.0)
    }
    pub fn to_yaml_string(&self) -> String {
        let yaml_type: YamlType = JsonType(self.data.clone()).into();
        let mut out_str = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut out_str);
            emitter.dump(&yaml_type.0).unwrap(); // dump the YAML object to a String
        }
        out_str
    }
    pub fn to_json(&self) -> String {
        self.data.to_string()
    }

    pub fn write(&self, path: &str) {
        fs::write(path, self.to_json()).expect("Unable to write file");
    }
    pub fn write_yaml(&self, path: &str) {
        fs::write(path, self.to_yaml_string()).expect("Unable to write file");
    }

    pub fn insert(&mut self, path: Vec<String>, value: Value) {
        let mut json_obj = &mut self.data;
        if path.len() == 1 {
            if let Value::Object(node_to_update) = json_obj {
                node_to_update.insert(path[0].to_owned(), value);
            }
        } else {
            let mut i = 0;
            while i < path.len() - 1 {
                let key = &path[i];
                if let Value::Object(node) = json_obj {
                    if path[i + 1].starts_with("$") && !node.contains_key(key) {
                        node.insert(key.to_owned(), Value::Array(Vec::new()));
                    } else if path[i + 1].starts_with("$") && !node[key].is_array() {
                        node.remove(key);
                        node.insert(key.clone(), Value::Array(Vec::new()));
                    } else if !node.contains_key(key) {
                        node.insert(key.clone(), Value::Object(Map::new()));
                    } else if !node[key].is_object() && !path[i + 1].starts_with("$") {
                        node.remove(key);
                        node.insert(key.clone(), Value::Object(Map::new()));
                    }
                    json_obj = &mut node[key];
                }
                i += 1;
            }
            if path[i].starts_with("$") {
                if let Value::Array(arr) = json_obj {
                    arr.push(value.clone());
                }
            }
            if let Value::Object(node_to_update) = json_obj {
                node_to_update.insert(path[i].to_owned(), value);
            }
        }
    }

    pub fn validate(&self, schema: &Value) -> Result<(), ValidationError> {
        jsonschema::validate(schema, &self.data)
    }
    pub fn merge(&mut self, overlay: File) {
        let stack_item = StackItem {
            path: Vec::new(),
            value: overlay.data,
        };
        let mut stack: Vec<StackItem> = Vec::new();
        stack.push(stack_item);
        let mut i = 0;

        while i < stack.len() {
            let stack_item = &stack[i];
            let mut add_to_stack: Vec<StackItem> = Vec::new();
            if let Value::Object(obj) = &stack_item.value {
                for (key, value) in obj.into_iter() {
                    let mut p = stack_item.path.clone();
                    p.push(key.to_owned());

                    add_to_stack.push(StackItem {
                        path: p,
                        value: value.clone(),
                    });
                }
                stack.append(&mut add_to_stack);
            } else if let Value::Array(arr) = &stack_item.value {
                for (j, item) in arr.iter().enumerate() {
                    let mut p = stack_item.path.clone();
                    let mut k = "$".to_owned();
                    let index = j.to_string();
                    k.push_str(&index);
                    p.push(k);
                    self.insert(p, item.clone());
                }
                stack.append(&mut add_to_stack);
            } else {
                self.insert(stack[i].path.clone(), stack[i].value.clone());
            }
            i += 1;
        }
    }
}
