use crate::file::File;
use serde_json::Value;
use std::fs;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct VersionedTemplate {
    values: Value,
    version: u32,
    overlays: Vec<Overlay>,
    schema: Value,
    created: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct Overlay {
    name: String,
    values: Value,
}

impl Overlay {
    pub fn new(name: &str, values: Value) -> Self {
        Overlay {
            name: name.to_owned(),
            values,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct Implementation {
    name: String,
    version: String,
    overlays: Vec<String>,
    overlay: Value,
    created: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct TemplateState {
    name: String,
    current: VersionedTemplate,
    previous_versions: Vec<VersionedTemplate>,
    created: String,
    updated: String,
    implementations: Vec<Implementation>,
}

impl TemplateState {
    fn new(name: &str, values: Value, overlays: Vec<Overlay>, schema: Value) -> Self {
        let now = chrono::Utc::now().to_string();
        let current = VersionedTemplate {
            version: 0,
            values,
            overlays,
            schema,
            created: now.to_owned(),
        };
        TemplateState {
            name: name.to_owned(),
            current,
            previous_versions: Vec::new(),
            created: now.to_owned(),
            updated: now.to_owned(),
            implementations: Vec::new(),
        }
    }

    fn values(&self) -> &Value {
        &self.current.values
    }
    fn overlays(&self) -> &[Overlay] {
        &self.current.overlays
    }
    fn schema(&self) -> &Value {
        &self.current.schema
    }
    fn template_has_changed(&self, template: &Value) -> bool {
        self.current.values.as_str() != template.as_str()
    }
    fn schema_has_changed(&self, schema: &Value) -> bool {
        self.current.schema.as_str() != schema.as_str()
    }
    fn overlays_have_changed(&self, overlays: &[Overlay]) -> bool {
        if self.current.overlays.len() != overlays.len() {
            return false;
        }
        let mut i = 0;
        while i < self.current.overlays.len() {
            if self.current.overlays[i].values.as_str() != overlays[i].values.as_str() {
                return false;
            }
            i += 1;
        }
        true
    }
    fn has_changed(&self, template: &TemplateState) -> bool {
        self.template_has_changed(template.values())
            && self.schema_has_changed(template.schema())
            && self.overlays_have_changed(template.overlays())
    }

    fn update_template(&mut self, template: TemplateState) {
        let current_version = self.current.version;
        self.previous_versions.push(self.current.clone());
        self.current = VersionedTemplate {
            values: template.current.values,
            overlays: template.current.overlays,
            version: current_version + 1,
            schema: template.current.schema,
            created: chrono::Utc::now().to_string(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct CometState {
    name: String,
    templates: Vec<TemplateState>,
    created: String,
    updated: String,
}

impl CometState {
    fn new(name: &str) -> Self {
        CometState {
            name: name.to_owned(),
            templates: Vec::new(),
            created: chrono::Utc::now().to_string(),
            updated: chrono::Utc::now().to_string(),
        }
    }
    fn get_template_mut(&mut self, name: &str) -> Option<&mut TemplateState> {
        let mut i = 0;
        while i < self.templates.len() {
            if self.templates[i].name == name {
                return Some(&mut self.templates[i]);
            }
            i += 1;
        }
        None
    }

    fn update_templates(&mut self, templates: Vec<TemplateState>) {
        let mut has_changed = false;
        for template in templates {
            if let Some(tmp) = self.get_template_mut(&template.name) {
                if tmp.has_changed(&template) {
                    tmp.update_template(template);
                    has_changed = true;
                }
            } else {
                self.templates.push(template);
                has_changed = true;
            }
        }
        if has_changed {
            self.updated = chrono::Utc::now().to_string();
        }
    }
}

pub fn sync_state_file(name: &str, path: &str) {
    let mut state_file = path.to_owned();
    state_file.push_str("gitcomet.gtcstate");
    let mut state: CometState;
    if let Ok(state_str) = fs::read_to_string(state_file.clone()) {
        state = serde_json::from_str(&state_str).expect("should parse ok");
    } else {
        state = CometState::new(name);
    }
    let mut templates_folder = path.to_owned();
    templates_folder.push_str("templates/");
    let paths = fs::read_dir(templates_folder).unwrap();
    let mut templ_states: Vec<TemplateState> = Vec::new();
    for path in paths {
        match path {
            Ok(path) => {
                let name = path.file_name();
                let name_str = name.to_str().expect("should work");
                let mut base_path = path.path().clone();
                base_path.push("base.yaml");
                let base = File::from_yaml(base_path);

                let mut schema_path = path.path().clone();
                schema_path.push("schema.yaml");
                let schema = File::from_yaml(schema_path);
                let mut overlays_folder = path.path().clone();
                overlays_folder.push("overlays/");
                let overlays = fs::read_dir(overlays_folder).unwrap();
                let mut ovrlys: Vec<Overlay> = Vec::new();
                for overlay in overlays {
                    match overlay {
                        Ok(overlay) => {
                            let name = overlay.file_name();
                            let name_str = name.to_str().expect("should work");
                            let contents = File::from_yaml(overlay.path());
                            let ovrly = Overlay::new(name_str, contents.data().clone());
                            ovrlys.push(ovrly);
                        }
                        _ => todo!(),
                    }
                }
                let tmpl = TemplateState::new(
                    name_str,
                    base.data().clone(),
                    ovrlys,
                    schema.data().clone(),
                );
                templ_states.push(tmpl);
            }
            _ => todo!(),
        }
    }
    state.update_templates(templ_states);
    let final_json = serde_json::to_string_pretty(&state).expect("expected to work");
    fs::write(&state_file, &final_json).expect("should work");
}
