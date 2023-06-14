//! Shader source utilities

use std::{
    collections::{hash_map::Keys, HashMap, HashSet},
    fmt::{Error, Write},
    sync::Mutex,
};

use handlebars::{no_escape, Handlebars, Helper, HelperDef, RenderError};
use rust_embed::RustEmbed;
use serde::Serialize;

use crate::types::{Material, VertexAttributeId};

/// Layout of vertex input attributes used by render pipelines
pub struct VertexInputStreamLayout {
    attributes: Vec<VertexBufferSpec>,
}

impl VertexInputStreamLayout {
    /// Constructs a new layout given a material
    pub fn new<M: Material>() -> Self {
        let required = M::required_attributes();
        let mut attrs = Vec::with_capacity(required.len());

        for attr in required.into_iter() {
            attrs.push(VertexBufferSpec { attribute: *attr });
        }

        Self { attributes: attrs }
    }
}

struct VertexBufferSpec {
    attribute: VertexAttributeId,
}

/// Preprocessor information for shader source generation.
pub struct ShaderPreprocessor {
    sources: HashMap<String, String>,
}

/// Compilation configuration settings
#[derive(Debug, Default, Serialize)]
pub struct ShaderCompileConfig {}

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/data/shaders"]
struct TempestShaderSources;

impl Default for ShaderPreprocessor {
    fn default() -> Self {
        let mut result = Self {
            sources: Default::default(),
        };
        result.add_embedded_shaders::<TempestShaderSources>("tempest");
        result
    }
}

impl ShaderPreprocessor {
    /// Adds embedded shaders into a sources list for usage by the preprocessor.
    pub fn add_embedded_shaders<T: RustEmbed>(&mut self, path_prefix: &str) {
        for f in T::iter() {
            let contents = String::from_utf8(T::get(&f).unwrap().data.into_owned()).unwrap();
            self.sources.insert(format!("{path_prefix}/{f}"), contents);
        }
    }

    /// Adds a shader with a given name and source code to the preprocessor
    pub fn add_shader(&mut self, name: &str, contents: &str) {
        self.sources.insert(name.to_owned(), contents.to_owned());
    }

    pub fn files(&self) -> Keys<'_, String, String> {
        self.sources.keys()
    }

    pub fn fetch(&self, name: &str) -> Option<&String> {
        self.sources.get(name)
    }

    pub fn bake<T: Serialize>(
        &self,
        base: &str,
        user_config: &T,
        vertex_buffers: Option<&VertexInputStreamLayout>,
    ) -> Result<String, RenderError> {
        #[derive(Serialize)]
        struct BufferConfig<'a, T> {
            vertex_attrib_count: usize,
            #[serde(flatten)]
            user_config: &'a T,
        }

        // prevent dupes
        let mut include_state = Mutex::new(HashSet::<String>::new());
        _ = include_state
            .get_mut()
            .and_then(|state| Ok(state.insert(base.to_string())));

        let mut hb_reg = Handlebars::new();
        hb_reg.set_strict_mode(true);
        hb_reg.set_dev_mode(cfg!(debug_assertions));
        hb_reg.register_escape_fn(no_escape);
        hb_reg.register_helper(
            "include",
            Box::new(ShaderPreprocessIncludeDirective::new(base, &self.sources)),
        );

        if let Some(cfg) = vertex_buffers {
            hb_reg.register_helper(
                "vertex_fetch",
                Box::new(ShaderPreprocessorVertexFetchDirective::new(cfg)),
            )
        }

        let contents = self
            .sources
            .get(base)
            .ok_or_else(|| RenderError::new(format!("Shader base template is not registered.")))?;

        let va_counts: usize = if let Some(vertex_buffers) = vertex_buffers {
            vertex_buffers.attributes.len()
        } else {
            0
        };

        hb_reg.render_template(
            contents,
            &BufferConfig {
                vertex_attrib_count: va_counts,
                user_config,
            },
        )
    }
}

struct ShaderPreprocessIncludeDirective<'a> {
    files: &'a HashMap<String, String>,    // File sources
    include_state: Mutex<HashSet<String>>, // Included files
}

impl<'a> ShaderPreprocessIncludeDirective<'a> {
    fn new(base: &str, files: &'a HashMap<String, String>) -> Self {
        Self {
            files,
            include_state: Mutex::new({
                let mut set = HashSet::new();
                set.insert(base.to_owned());
                set
            }),
        }
    }
}

impl<'a> HelperDef for ShaderPreprocessIncludeDirective<'a> {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &handlebars::Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc handlebars::Context,
        _rc: &mut handlebars::RenderContext<'reg, 'rc>,
        out: &mut dyn handlebars::Output,
    ) -> handlebars::HelperResult {
        let file_name = match h
            .param(0)
            .ok_or_else(|| RenderError::new("Missing argument 0 for shader source include path."))?
            .value()
        {
            handlebars::JsonValue::String(s) => s,
            _ => {
                return Err(RenderError::new(
                    "Shader source include path must be a string.",
                ))
            }
        };

        let mut lock = self.include_state.try_lock();
        if let Ok(ref mut includes) = lock {
            if includes.contains(file_name) {
                return Ok(());
            }

            includes.insert(file_name.clone());
        }
        drop(lock); // early drop lock, since this template is filled out recursively

        let contents = self
            .files
            .get(file_name)
            .ok_or_else(|| RenderError::new("Included file is not registered in the map."))?;

        out.write(&r.render_template(contents, ctx.data())?)?;

        Ok(())
    }
}

struct ShaderPreprocessorVertexFetchDirective<'a> {
    config: &'a VertexInputStreamLayout,
}

impl<'a> ShaderPreprocessorVertexFetchDirective<'a> {
    fn new(config: &'a VertexInputStreamLayout) -> Self {
        Self { config }
    }

    fn generate(
        &self,
        h: &Helper,
        object_buffer: &str,
        batch_buffer: &str,
    ) -> Result<String, Error> {
        let includes = r#"{{include "tempest/vertex_layout.wgsl"}}"#;
        let unpack_fn_tpl = format!("
            fn unpack_vertex_index(vertex_index: u32) -> Indices {{
                let local_object_index = vertex_index >> 24u;
                let vertex_id = vertex_index & 0xFFFFFFu;
                let object_id = {batch_buffer}.ranges[local_object_index].object_id;

                return Indices(vertex_id, object_id);
            }}"
        );

        let mut vertex_input_struct = String::new();
        writeln!(vertex_input_struct, "struct VertexInput {{")?;
        let mut input_fn = String::new();
        writeln!(
            input_fn,
            "fn get_vertex(indices: Indices) -> VertexInput {{"
        )?;
        writeln!(input_fn, "  var vert: VertexInput;")?;
        for attr in &h.params()[2..] {
            let (attr_idx, spec) = self
                .config
                .attributes
                .iter()
                .enumerate()
                .find_map(
                    |(idx, s)| match s.attribute.name() == attr.relative_path().unwrap() {
                        true => Some((idx, s)),
                        false => None,
                    },
                )
                .unwrap();

            writeln!(
                vertex_input_struct,
                "  {}: {},",
                spec.attribute.name(),
                spec.attribute.meta().data_type
            )?;

            writeln!(input_fn, "  let {}_offset = {object_buffer}[indices.object].vertex_attribute_start_offsets[{attr_idx}];", spec.attribute.name())?;
            writeln!(
                input_fn,
                "  vert.{name} = {}({name}_offset, indices.vertex);",
                spec.attribute.meta().extracter,
                name = spec.attribute.name()
            )?;
        }

        writeln!(vertex_input_struct, "}}")?;
        writeln!(input_fn, "  return vert;")?;
        writeln!(input_fn, "}}")?;

        let template = format!("{includes}{unpack_fn_tpl}{vertex_input_struct}{input_fn}");

        Ok(template)
    }
}

impl<'a> HelperDef for ShaderPreprocessorVertexFetchDirective<'a> {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &handlebars::Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc handlebars::Context,
        _rc: &mut handlebars::RenderContext<'reg, 'rc>,
        out: &mut dyn handlebars::Output,
    ) -> handlebars::HelperResult {
        let object_buffer = match h
            .param(0)
            .ok_or_else(|| RenderError::new("Missing argument 0 for vertex fetch."))?
            .relative_path()
        {
            Some(s) => s,
            _ => {
                return Err(RenderError::new(
                    "Vertex fetch first argument must be a string.",
                ))
            }
        };

        let batch_buffer = match h
            .param(1)
            .ok_or_else(|| {
                RenderError::new(
                    "Vertex fetch must have argument at index 1 for pointing to the batch data.",
                )
            })?
            .relative_path()
        {
            Some(s) => s,
            _ => {
                return Err(RenderError::new(
                    "Vertex fetch second argument must be a string.",
                ));
            }
        };

        let template = self.generate(h, object_buffer, batch_buffer).map_err(|_| {
            RenderError::new("Failed to write vertex buffer template string to template.")
        })?;

        out.write(&r.render_template(&template, ctx.data())?)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{attributes, Sorting};

    use super::*;

    #[test]
    fn test_single_include() {
        let mut preprocessor = ShaderPreprocessor::default();
        preprocessor.add_shader("shader1", "{{include \"shader2\"}} shader1");
        preprocessor.add_shader("shader2", "shader2");
        let config = ShaderCompileConfig::default();
        let output = preprocessor.bake("shader1", &config, None).unwrap();

        assert_eq!(output, "shader2 shader1");
    }

    #[test]
    fn test_invalid_include_no_argument() {
        let mut preprocessor = ShaderPreprocessor::default();
        preprocessor.add_shader("shader1", "{{include}}");
        let config = ShaderCompileConfig::default();
        let output = preprocessor.bake("shader1", &config, None);
        assert!(output.is_err(), "Expected error: {output:?}");
    }

    #[test]
    fn test_invalid_include_no_such_path() {
        let mut preprocessor = ShaderPreprocessor::default();
        preprocessor.add_shader("shader1", "{{include \"shader2\"}}");
        let config = ShaderCompileConfig::default();
        let output = preprocessor.bake("shader1", &config, None);
        assert!(output.is_err(), "Expected error: {output:?}");
    }

    #[test]
    fn test_recursive_include() {
        let mut preprocessor = ShaderPreprocessor::default();
        preprocessor.add_shader("shader1", "{{include \"shader2\"}} shader1");
        preprocessor.add_shader("shader2", "{{include \"shader1\"}} shader2");
        let config = ShaderCompileConfig::default();
        let output = preprocessor.bake("shader1", &config, None).unwrap();
        assert_eq!(output, " shader2 shader1");
    }

    #[test]
    fn test_with_vertex_attribute() {
        struct TestMaterial {}

        impl Material for TestMaterial {
            fn required_attributes() -> Vec<&'static VertexAttributeId> {
                vec![attributes::POSITION.id(), attributes::UVCOORD_0.id(), attributes::NORMAL.id()]
            }

            fn sorting(&self) -> crate::types::Sorting {
                Sorting {
                    order: crate::types::SortOrder::FrontToBack,
                    reason: crate::types::SortRequirement::Required,
                }
            }
        }

        let mut preprocessor = ShaderPreprocessor::default();
        let config = ShaderCompileConfig::default();
        let layout = VertexInputStreamLayout::new::<TestMaterial>();
        preprocessor.add_shader("shader", "{{vertex_fetch object_buffer batch_data position uvcoord_0 normal}}");
        let output = preprocessor.bake("shader", &config, Some(&layout));
        assert!(output.is_ok(), "Expected successful preprocess.");
    }
}
