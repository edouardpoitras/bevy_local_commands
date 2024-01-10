use std::{ffi::OsStr, process::Command};

use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct LocalCommand {
    pub command: Command,
}

impl LocalCommand {
    pub fn new<S>(program: S) -> Self
    where
        S: AsRef<OsStr>,
    {
        Self {
            command: Command::new(program),
        }
    }

    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.command.arg(arg);
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn env<K, V>(mut self, key: K, val: V) -> Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.env(key, val);
        self
    }

    pub fn envs<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.envs(vars);
        self
    }

    pub fn env_remove<K: AsRef<OsStr>>(mut self, key: K) -> Self {
        self.command.env_remove(key);
        self
    }

    pub fn env_clear(mut self) -> Self {
        self.command.env_clear();
        self
    }

    pub fn current_dir<P: AsRef<std::path::Path>>(mut self, dir: P) -> Self {
        self.command.current_dir(dir);
        self
    }
}
