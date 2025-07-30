use std::collections::HashMap;

use axum::http::Method;
use regex::Regex;
use serde::Serialize;

crate::inventory::collect!(&'static EndpointSpecification);

pub struct EndpointSpecification {
    pub operation_id: &'static str,
    pub path: &'static str,
    pub method: Method,
    pub summary: &'static str,
    pub description: Option<&'static str>,
    pub parameters: &'static [ParameterSpecification],
    pub responses: &'static [Response],
}

pub struct ConstParameterSpecification {
    pub name: &'static str,
    pub in_: &'static str, // e.g., "query", "path", "header"
    pub description: Option<&'static str>,
    pub required: bool,
    pub schema: ConstSchemaSpecification,
}

impl Into<ParameterSpecification> for ConstParameterSpecification {
    fn into(self) -> ParameterSpecification {
        ParameterSpecification {
            name: self.name.to_string(),
            in_: self.in_.to_string(),
            description: self.description.map(|d| d.to_string()),
            required: self.required,
            schema: self.schema.into(),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParameterSpecification {
    pub name: String,
    #[serde(rename = "in")]
    pub in_: String, // e.g., "query", "path", "header"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
    pub schema: SchemaSpecification,
}

#[derive(Clone)]
pub struct ConstSchemaSpecification {
    pub title: &'static str,
    pub kind: &'static str, // e.g., "string", "integer", "array"
    pub properties: HashMap<String, ConstSchemaSpecification>, // For objects
    pub enumeration: Option<Vec<String>>, // For enums
    pub format: Option<&'static str>, // e.g., "int32", "date-time"
    pub items: Option<Box<ConstSchemaSpecification>>, // For arrays
}

impl Into<SchemaSpecification> for ConstSchemaSpecification {
    fn into(self) -> SchemaSpecification {
        SchemaSpecification {
            title: self.title.to_string(),
            kind: self.kind.to_string(),
            properties: self
                .properties
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            enumeration: self.enumeration,
            format: self.format.map(|d| d.to_string()),
            items: self.items.map(|item| item.into()),
        }
    }
}

impl Into<Box<SchemaSpecification>> for Box<ConstSchemaSpecification> {
    fn into(self) -> Box<SchemaSpecification> {
        Box::new(SchemaSpecification {
            title: self.title.to_string(),
            kind: self.kind.to_string(),
            properties: self
                .properties
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            enumeration: self.enumeration,
            format: self.format.map(|d| d.to_string()),
            items: self.items.map(|item| item.into()),
        })
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaSpecification {
    #[serde(skip)]
    pub title: String,
    #[serde(rename = "type")]
    pub kind: String, // e.g., "string", "integer", "array"
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, SchemaSpecification>, // For objects
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enumeration: Option<Vec<String>>, // For enums
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>, // e.g., "int32", "date-time"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<SchemaSpecification>>, // For arrays
}

#[derive(Clone)]
pub struct Response {
    pub status_code: &'static str,
    pub description: &'static str,
    pub content: Vec<ConstContentSpecification>,
}

#[derive(Clone)]
pub struct ConstContentSpecification {
    pub media_type: &'static str, // e.g., "application/json"
    pub schema: ConstSchemaSpecification,
}

#[derive(Clone)]
pub struct ContentSpecification {
    pub media_type: String, // e.g., "application/json"
    pub schema: SchemaSpecification,
}

impl Into<ContentSpecification> for ConstContentSpecification {
    fn into(self) -> ContentSpecification {
        ContentSpecification {
            media_type: self.media_type.to_string(),
            schema: self.schema.into(),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefSpecification {
    #[serde(rename = "$ref")]
    pub reference: String, // Reference to a schema definition
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaRefSpecification {
    pub schema: RefSpecification,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseSpecification {
    pub description: String,
    pub content: HashMap<String, SchemaRefSpecification>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InfoSpecification {
    pub title: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerSpecification {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDocsSpecification {
    pub description: String,
    pub url: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OperationSpecification {
    pub summary: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ParameterSpecification>,
    //pub request_body: Option<SchemaSpecification>,
    pub responses: HashMap<String, ResponseSpecification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocsSpecification>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SerializedEndpoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<OperationSpecification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<OperationSpecification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<OperationSpecification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<OperationSpecification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<OperationSpecification>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComponentsSpecification {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub schemas: HashMap<String, SchemaSpecification>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub responses: HashMap<String, ResponseSpecification>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub parameters: HashMap<String, ParameterSpecification>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenApiDocument {
    pub openapi: String,
    pub info: InfoSpecification,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<ServerSpecification>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub paths: HashMap<String, SerializedEndpoint>,
    pub components: ComponentsSpecification,
}

pub fn generate_specification<'a>() -> OpenApiDocument {
    let mut document = OpenApiDocument {
        openapi: "3.0.0".to_string(),
        info: InfoSpecification {
            title: "My API".to_string(),
            version: "1.0.0".to_string(),
            description: Some("This is a sample API".to_string()),
        },
        servers: vec![ServerSpecification {
            url: "http://localhost:3000".to_string(),
            description: Some("Local development server".to_string()),
        }],
        paths: HashMap::new(),
        components: ComponentsSpecification {
            schemas: HashMap::new(),
            responses: HashMap::new(),
            parameters: HashMap::new(),
        },
    };

    for route in inventory::iter::<&EndpointSpecification> {
        // replace instances of {param:type} with {param}
        let path = Regex::new(r"\{([^:]+):[^}]+\}")
            .unwrap()
            .replace_all(route.path, "{$1}")
            .to_string();

        if !document.paths.contains_key(&path) {
            document.paths.insert(
                path.clone(),
                SerializedEndpoint {
                    get: None,
                    post: None,
                    put: None,
                    delete: None,
                    patch: None,
                },
            );
        }

        let endpoint = document.paths.get_mut(&path).unwrap();

        let endpoint = match route.method {
            Method::GET => &mut endpoint.get,
            Method::POST => &mut endpoint.post,
            Method::PUT => &mut endpoint.put,
            Method::DELETE => &mut endpoint.delete,
            Method::PATCH => &mut endpoint.patch,
            _ => panic!("unsupported method"),
        };

        *endpoint = Some(OperationSpecification {
            summary: route.summary.to_string(),
            tags: vec![], // Tags can be added later
            description: route.description.map(|d| d.to_string()),
            operation_id: Some(route.operation_id.to_string()),
            parameters: Vec::from(route.parameters),
            responses: HashMap::new(),
            external_docs: None, // External docs can be added later
        });

        let path_parameters = Regex::new(r"\{([^:}]+):([^}]+)\}")
            .unwrap()
            .captures_iter(route.path)
            .map(|cap| (cap[1].to_string(), cap[2].to_string()))
            .collect::<Vec<_>>();

        for (param_name, param_type) in path_parameters {
            if endpoint
                .as_ref()
                .unwrap()
                .parameters
                .iter()
                .any(|p| p.name == param_name && p.in_ == "path")
            {
                continue; // Skip if parameter already exists
            }

            let parameter_spec = ParameterSpecification {
                name: param_name.clone(),
                in_: "path".to_string(),
                description: Some(format!("Path parameter: {}", param_name)),
                required: true,
                schema: SchemaSpecification {
                    title: param_name.clone(),
                    kind: match param_type.as_str() {
                        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" => {
                            "integer".to_string()
                        }
                        "f32" | "f64" => "number".to_string(),
                        "bool" | "boolean" => "boolean".to_string(),
                        _ => "string".to_string(),
                    },
                    properties: HashMap::new(),
                    enumeration: None,
                    format: None,
                    items: None,
                },
            };

            if !document.components.parameters.contains_key(&param_name) {
                document
                    .components
                    .parameters
                    .insert(param_name.clone(), parameter_spec.clone());
            }

            endpoint.as_mut().unwrap().parameters.push(parameter_spec);
        }

        for response in &Vec::from(route.responses) {
            let mut response_spec = ResponseSpecification {
                description: response.description.to_string(),
                content: HashMap::new(),
            };

            for content in &response.content {
                if !document
                    .components
                    .schemas
                    .contains_key(content.schema.title)
                {
                    document.components.schemas.insert(
                        content.schema.title.to_string(),
                        content.schema.clone().into(),
                    );
                }

                let schema_ref = SchemaRefSpecification {
                    schema: RefSpecification {
                        reference: format!("#/components/schemas/{}", content.schema.title),
                    },
                };

                response_spec
                    .content
                    .insert(content.media_type.to_string(), schema_ref);
            }

            endpoint
                .as_mut()
                .unwrap()
                .responses
                .insert(response.status_code.to_string(), response_spec);
        }
    }

    document
}
