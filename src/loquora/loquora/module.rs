use crate::loquora::ast::*;
use crate::loquora::environment::{ToolDef, TypeDef};
use crate::loquora::lexer::Lexer;
use crate::loquora::parser::Parser;
use crate::loquora::value::RuntimeError;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Module {
    pub path: PathBuf,
    pub exports: ModuleExports,
    pub initialized: bool,
}

#[derive(Clone, Debug)]
pub struct ModuleExports {
    pub tools: HashMap<String, ToolDef>,
    pub structs: HashMap<String, TypeDef>,
    pub templates: HashMap<String, TypeDef>,
}

impl ModuleExports {
    pub fn new() -> Self {
        ModuleExports {
            tools: HashMap::new(),
            structs: HashMap::new(),
            templates: HashMap::new(),
        }
    }
}

pub struct ModuleCache {
    modules: HashMap<PathBuf, Module>,
    loading_stack: Vec<PathBuf>,
    stdlib: HashMap<String, Module>,
    search_paths: Vec<PathBuf>,
}

impl ModuleCache {
    pub fn new() -> Self {
        let mut cache = ModuleCache {
            modules: HashMap::new(),
            loading_stack: Vec::new(),
            stdlib: HashMap::new(),
            search_paths: vec![
                PathBuf::from("."),
                PathBuf::from("./src"),
                PathBuf::from("./.loq/std"),
            ],
        };

        cache.init_stdlib();
        cache
    }

    fn init_stdlib(&mut self) {}

    #[allow(dead_code)]
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    fn resolve_module_path(&self, module_path: &[String]) -> Result<PathBuf, RuntimeError> {
        let module_name = module_path.join("/");
        if let Some(stdlib_mod) = self.stdlib.get(&module_name) {
            return Ok(stdlib_mod.path.clone());
        }

        let mut file_path = PathBuf::new();
        for (i, part) in module_path.iter().enumerate() {
            if i < module_path.len() - 1 {
                file_path.push(part);
            } else {
                file_path.push(format!("{}.loq", part));
            }
        }

        for search_path in &self.search_paths {
            let full_path = search_path.join(&file_path);
            if full_path.exists() {
                return Ok(full_path.canonicalize().map_err(|e| {
                    RuntimeError::Custom(format!("Failed to canonicalize path: {}", e))
                })?);
            }
        }

        Err(RuntimeError::Custom(format!(
            "Module not found: {} (searched: {:?})",
            module_path.join("/"),
            file_path
        )))
    }

    pub fn load_module(&mut self, module_path: &[String], run: bool) -> Result<Module, RuntimeError> {
        let file_path = self.resolve_module_path(module_path)?;

        if let Some(module) = self.modules.get(&file_path) {
            if !module.initialized {
                return Err(RuntimeError::Custom(format!(
                    "Circular import detected: {} is currently being loaded",
                    file_path.display()
                )));
            }
            return Ok(module.clone());
        }

        if self.loading_stack.contains(&file_path) {
            return Err(RuntimeError::Custom(format!(
                "Circular import detected: {}",
                file_path.display()
            )));
        }

        self.loading_stack.push(file_path.clone());

        let source = fs::read_to_string(&file_path)
            .map_err(|e| RuntimeError::Custom(format!("Failed to read module: {}", e)))?;

        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        if !run {
        let exports = self.extract_exports(&program)?;

        let module = Module {
            path: file_path.clone(),
            exports,
            initialized: true,
        };

        self.modules.insert(file_path.clone(), module.clone());
        self.loading_stack.pop();

        Ok(module)
        } else {
        let exports = self.extract_exports(&program)?;

        let module = Module {
            path: file_path.clone(),
            exports,
            initialized: true,
        };

        self.modules.insert(file_path.clone(), module.clone());
        self.loading_stack.pop();

        Ok(module)
        }
    }

    fn extract_exports(&mut self, program: &Program) -> Result<ModuleExports, RuntimeError> {
        let mut exports = ModuleExports::new();

        for stmt in &program.statements {
            match &stmt.inner {
                StmtKind::ExportDecl { decl } => {
                    self.extract_export(&mut exports, decl)?;
                }
                _ => {}
            }
        }

        Ok(exports)
    }

    fn extract_export(
        &mut self,
        exports: &mut ModuleExports,
        decl: &Stmt,
    ) -> Result<(), RuntimeError> {
        match &decl.inner {
            StmtKind::ToolDecl {
                name,
                params,
                return_type: _,
                body,
            } => {
                exports.tools.insert(
                    name.clone(),
                    ToolDef {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            }

            StmtKind::StructDecl { name, members } => {
                exports.structs.insert(
                    name.clone(),
                    TypeDef::Struct {
                        name: name.clone(),
                        members: members.clone(),
                    },
                );
            }

            StmtKind::TemplateDecl { name, params, body } => {
                exports.templates.insert(
                    name.clone(),
                    TypeDef::Template {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            }

            _ => {
                return Err(RuntimeError::Custom(format!(
                    "Cannot export this declaration type"
                )));
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.modules.clear();
        self.loading_stack.clear();
    }

    #[allow(dead_code)]
    pub fn remove_module(&mut self, path: &[String]) -> bool {
        if let Ok(resolved_path) = self.resolve_module_path(path) {
            self.modules.remove(&resolved_path).is_some()
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn is_cached(&self, path: &[String]) -> bool {
        let module_name = path.join("/");
        if let Ok(resolved_path) = self.resolve_module_path(path) {
            self.modules.contains_key(&resolved_path) || self.stdlib.contains_key(&module_name)
        } else {
            self.stdlib.contains_key(&module_name)
        }
    }

    #[allow(dead_code)]
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_modules: self.modules.len(),
            stdlib_modules: self.stdlib.len(),
            search_paths: self.search_paths.len(),
            total_exports: self
                .modules
                .values()
                .map(|m| {
                    m.exports.tools.len() + m.exports.structs.len() + m.exports.templates.len()
                })
                .sum(),
        }
    }

    #[allow(dead_code)]
    pub fn list_cached_modules(&self) -> Vec<PathBuf> {
        self.modules.keys().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn list_search_paths(&self) -> Vec<PathBuf> {
        self.search_paths.clone()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct CacheStats {
    pub cached_modules: usize,
    pub stdlib_modules: usize,
    pub search_paths: usize,
    pub total_exports: usize,
}
