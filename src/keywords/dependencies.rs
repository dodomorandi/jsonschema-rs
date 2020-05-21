use super::{CompilationResult, Validate, Validators};
use crate::{
    compilation::{compile_validators, CompilationContext, JSONSchema},
    error::{no_error, CompilationError, ErrorIterator},
    keywords::{format_key_value_validators, required::RequiredValidator},
};
use serde_json::{Map, Value};

pub struct DependenciesValidator {
    dependencies: Vec<(String, Validators)>,
}

impl DependenciesValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        if let Value::Object(map) = schema {
            let mut dependencies = Vec::with_capacity(map.len());
            for (key, subschema) in map {
                let s = match subschema {
                    Value::Array(_) => vec![RequiredValidator::compile(subschema)?],
                    _ => compile_validators(subschema, context)?,
                };
                dependencies.push((key.clone(), s))
            }
            return Ok(Box::new(DependenciesValidator { dependencies }));
        }
        Err(CompilationError::SchemaError)
    }
}

impl Validate for DependenciesValidator {
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(item) = instance {
            let errors: Vec<_> = self
                .dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .flat_map(move |(_, validators)| {
                    validators
                        .iter()
                        .flat_map(move |validator| validator.validate(schema, instance))
                })
                .collect();
            // TODO. custom error message for "required" case
            return Box::new(errors.into_iter());
        }
        no_error()
    }

    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(item) = instance {
            return self
                .dependencies
                .iter()
                .filter(|(property, _)| item.contains_key(property))
                .all(move |(_, validators)| {
                    validators
                        .iter()
                        .all(move |validator| validator.is_valid(schema, instance))
                });
        }
        true
    }

    fn name(&self) -> String {
        format!(
            "dependencies: {{{}}}",
            format_key_value_validators(&self.dependencies)
        )
    }
}

#[inline]
pub fn compile(
    _: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    Some(DependenciesValidator::compile(schema, context))
}
