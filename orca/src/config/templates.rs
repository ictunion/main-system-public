use handlebars::Handlebars;
use std::collections::{HashMap, HashSet};
use std::fs;

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

fn get_key_for(template: &Template, lang: &str) -> String {
    format!("{}/{}.html", template.name, lang)
}

fn get_default_key(template: &Template) -> String {
    format!("{}/default.html", template.name)
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
        // Tempalte can be directory containing different translations
        // default.html is translation which gets used when no other translation is found.
        match fs::read_dir(format!("{}/{}", path, template.name)) {
            Ok(paths) => {
                for path in paths {
                    let file_path = path?;
                    self.handlebars.register_template_file(
                        &format!(
                            "{}/{}",
                            template.name,
                            file_path.file_name().to_str().unwrap()
                        ),
                        file_path.path(),
                    )?;
                }
            }
            Err(_err) => {
                self.handlebars.register_template_file(
                    &get_default_key(template),
                    format!("{}/{}.html", path, template.name),
                )?;
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

pub struct Renderer<'a> {
    template: String,
    // At the moment we don't really need more things thatn string
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
        assert!(templates.preload_templates("templates").is_ok());
    }
}
