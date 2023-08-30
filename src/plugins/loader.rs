use std::{
    io,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use fs_err::{read, read_dir};
use wasmtime::Engine;

use super::plugin::{Plugin, WasmPluginBuilder};
use crate::errors::Result;
use crate::state::ShellState;

/// Searches paths for files ending in .wasm and loads them as plugins.
pub struct RecursivePluginLoader {
    paths: Vec<io::Result<PathBuf>>,
    state: Arc<RwLock<ShellState>>,
    engine: Engine,
}

impl RecursivePluginLoader {
    pub fn new(engine: Engine, state: Arc<RwLock<ShellState>>) -> Self {
        let plugin_paths = state.read().unwrap().config.plugin_paths.clone();
        Self {
            paths: plugin_paths.clone().into_iter().rev().map(Ok).collect(),
            state,
            engine,
        }
    }
}

impl Iterator for RecursivePluginLoader {
    type Item = Result<Box<dyn Plugin>>;
    fn next(&mut self) -> Option<Self::Item> {
        let path = match self.paths.pop()? {
            Ok(path) => path,
            Err(e) => return Some(Err(e.into())),
        };

        if path.is_dir() {
            match read_dir(&path) {
                Ok(dir) => {
                    // We can only return 1 plugin per iteration so we need to
                    // store the rest of the paths for later.
                    self.paths
                        .extend(dir.map(|result| result.map(|entry| entry.path())).filter(
                            |path| {
                                // filter out non-wasm files as they aren't plugins
                                path.as_ref()
                                    .map(|path| path.extension() == Some("wasm".as_ref()))
                                    .unwrap_or(true)
                            },
                        ));
                    return self.next();
                }
                Err(e) => return Some(Err(e.into())),
            }
        }

        let bytes = match read(&path) {
            Ok(bytes) => bytes,
            Err(e) => return Some(Err(e.into())),
        };

        let plugin = WasmPluginBuilder::new(&self.engine, &bytes)
            .name(path.file_name().unwrap().to_str().unwrap().to_string())
            .state(self.state.clone())
            .wasi(true)
            .build();
        Some(match plugin {
            Ok(plugin) => Ok(Box::new(plugin)),
            Err(e) => Err(e),
        })
    }
}
