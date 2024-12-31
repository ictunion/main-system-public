use handlebars::Handlebars;
use mrml;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::str;

pub const EMAIL_VERIFICATION: Template = Template {
    name: "email_verification",
};

pub const NEW_APPLICATION_NOTICE: Template = Template {
    name: "new_application_notice",
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Template {
    name: &'static str,
}

#[derive(Debug, Clone)]
pub struct Templates<'a> {
    handlebars: Handlebars<'a>,
    loaded: HashSet<Template>,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    TemplateLoading(Box<handlebars::TemplateError>),
    Infallible(std::convert::Infallible),
    Parser(Box<mrml::prelude::parser::Error>),
    Renderer(Box<mrml::prelude::render::Error>),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<handlebars::TemplateError> for Error {
    fn from(value: handlebars::TemplateError) -> Self {
        Self::TemplateLoading(Box::new(value))
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(value: std::convert::Infallible) -> Self {
        Self::Infallible(value)
    }
}

impl From<mrml::prelude::parser::Error> for Error {
    fn from(value: mrml::prelude::parser::Error) -> Self {
        Self::Parser(Box::new(value))
    }
}

impl From<mrml::prelude::render::Error> for Error {
    fn from(value: mrml::prelude::render::Error) -> Self {
        Self::Renderer(Box::new(value))
    }
}

fn get_key_for(template: &Template, lang: &str) -> String {
    format!("{}/{}.mjml", template.name, lang)
}

fn get_default_key(template: &Template) -> String {
    format!("{}/default.mjml", template.name)
}

impl<'a> Templates<'a> {
    pub fn new() -> Self {
        Templates {
            handlebars: Handlebars::new(),
            loaded: HashSet::new(),
        }
    }

    pub fn preload_templates(&mut self, path: &str) -> Result<(), Error> {
        self.load_template(path, &EMAIL_VERIFICATION)?;
        self.load_template(path, &NEW_APPLICATION_NOTICE)?;
        Ok(())
    }

    fn load_template(&mut self, path: &str, template: &Template) -> Result<(), Error> {
        // Template can be directory containing different translations
        // default.html is translation which gets used when no other translation is found.
        let opts = mrml::prelude::render::RenderOptions::default();
        match fs::read_dir(format!("{}/{}", path, template.name)) {
            Ok(paths) => {
                for path in paths {
                    let file_path = path?;
                    let mrml_template: String = fs::read_to_string(file_path.path())?.parse()?;
                    let rendered = mrml::parse(mrml_template)?.render(&opts)?;
                    self.handlebars.register_template_string(
                        &format!(
                            "{}/{}",
                            template.name,
                            file_path.file_name().to_str().unwrap()
                        ),
                        rendered,
                    )?;
                }
            }
            Err(_err) => {
                let mrml_template: String =
                    fs::read_to_string(format!("{}/{}.mjml", path, template.name))?.parse()?;
                let rendered = mrml::parse(mrml_template)?.render(&opts)?;
                self.handlebars
                    .register_template_string(&get_default_key(template), rendered)?;
            }
        };

        self.loaded.insert(template.clone());
        Ok(())
    }

    pub fn renderer(&self, template: &Template, lang: &str) -> Renderer<'a> {
        let template_name = if self.handlebars.has_template(&get_key_for(template, lang)) {
            // If has template for specific language
            get_key_for(template, lang)
        } else {
            // Or use default template
            get_default_key(template)
        };

        Renderer::new(template_name)
    }

    pub fn render(&self, renderer: &Renderer) -> Result<String, handlebars::RenderError> {
        self.handlebars.render(&renderer.template, &renderer.data)
    }
}
impl Default for Templates<'_> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Renderer<'a> {
    template: String,
    // At the moment we don't really need more things than string
    // otherwise this would become some serde base type
    data: HashMap<&'a str, &'a str>,
}

impl<'a> Renderer<'a> {
    pub fn new(template: String) -> Self {
        Self {
            template,
            data: HashMap::new(),
        }
    }

    pub fn bind(&mut self, name: &'a str, value: &'a str) -> &mut Self {
        self.data.insert(name, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_load_templates() {
        let mut templates = Templates::new();
        assert!(templates.preload_templates("templates/emails").is_ok());
    }
}
