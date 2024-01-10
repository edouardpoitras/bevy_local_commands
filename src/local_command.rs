use std::{
    ffi::OsStr,
    fmt::Debug,
    path::Path,
    process::{Command, CommandArgs, CommandEnvs},
};

use bevy::prelude::*;

#[derive(Component)]
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

    /// Adds an argument to pass to the program.
    ///
    /// Only one argument can be passed per use. So instead of:
    ///
    /// ```no_run
    /// # bevy_local_commands::LocalCommand::new("sh")
    /// .arg("-C /path/to/repo")
    /// # ;
    /// ```
    ///
    /// usage would be:
    ///
    /// ```no_run
    /// # bevy_local_commands::LocalCommand::new("sh")
    /// .arg("-C")
    /// .arg("/path/to/repo")
    /// # ;
    /// ```
    ///
    /// To pass multiple arguments see [`args`].
    ///
    /// [`args`]: LocalCommand::args
    ///
    /// Note that the argument is not passed through a shell, but given
    /// literally to the program. This means that shell syntax like quotes,
    /// escaped characters, word splitting, glob patterns, variable substitution, etc.
    /// have no effect.
    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.command.arg(arg);
        self
    }

    /// Adds multiple arguments to pass to the program.
    ///
    /// To pass a single argument see [`arg`].
    ///
    /// [`arg`]: LocalCommand::arg
    ///
    /// Note that the arguments are not passed through a shell, but given
    /// literally to the program. This means that shell syntax like quotes,
    /// escaped characters, word splitting, glob patterns, variable substitution, etc.
    /// have no effect.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    /// Inserts or updates an explicit environment variable mapping.
    ///
    /// This method allows you to add an environment variable mapping to the spawned process or
    /// overwrite a previously set value. You can use [`LocalCommand::envs`] to set multiple environment
    /// variables simultaneously.
    ///
    /// Child processes will inherit environment variables from their parent process by default.
    /// Environment variables explicitly set using [`LocalCommand::env`] take precedence over inherited
    /// variables. You can disable environment variable inheritance entirely using
    /// [`LocalCommand::env_clear`] or for a single key using [`LocalCommand::env_remove`].
    ///
    /// Note that environment variable names are case-insensitive (but
    /// case-preserving) on Windows and case-sensitive on all other platforms.
    pub fn env<K, V>(mut self, key: K, val: V) -> Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.env(key, val);
        self
    }

    /// Inserts or updates multiple explicit environment variable mappings.
    ///
    /// This method allows you to add multiple environment variable mappings to the spawned process
    /// or overwrite previously set values. You can use [`LocalCommand::env`] to set a single environment
    /// variable.
    ///
    /// Child processes will inherit environment variables from their parent process by default.
    /// Environment variables explicitly set using [`LocalCommand::envs`] take precedence over inherited
    /// variables. You can disable environment variable inheritance entirely using
    /// [`LocalCommand::env_clear`] or for a single key using [`LocalCommand::env_remove`].
    ///
    /// Note that environment variable names are case-insensitive (but case-preserving) on Windows
    /// and case-sensitive on all other platforms.
    pub fn envs<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.envs(vars);
        self
    }

    /// Removes an explicitly set environment variable and prevents inheriting it from a parent
    /// process.
    ///
    /// This method will remove the explicit value of an environment variable set via
    /// [`LocalCommand::env`] or [`LocalCommand::envs`]. In addition, it will prevent the spawned child
    /// process from inheriting that environment variable from its parent process.
    ///
    /// After calling [`LocalCommand::env_remove`], the value associated with its key from
    /// [`LocalCommand::get_envs`] will be [`None`].
    ///
    /// To clear all explicitly set environment variables and disable all environment variable
    /// inheritance, you can use [`LocalCommand::env_clear`].
    pub fn env_remove<K: AsRef<OsStr>>(mut self, key: K) -> Self {
        self.command.env_remove(key);
        self
    }

    /// Clears all explicitly set environment variables and prevents inheriting any parent process
    /// environment variables.
    ///
    /// This method will remove all explicitly added environment variables set via [`LocalCommand::env`]
    /// or [`LocalCommand::envs`]. In addition, it will prevent the spawned child process from inheriting
    /// any environment variable from its parent process.
    ///
    /// After calling [`LocalCommand::env_clear`], the iterator from [`LocalCommand::get_envs`] will be
    /// empty.
    ///
    /// You can use [`LocalCommand::env_remove`] to clear a single mapping.
    pub fn env_clear(mut self) -> Self {
        self.command.env_clear();
        self
    }

    /// Sets the working directory for the child process.
    ///
    /// # Platform-specific behavior
    ///
    /// If the program path is relative (e.g., `"./script.sh"`), it's ambiguous
    /// whether it should be interpreted relative to the parent's working
    /// directory or relative to `current_dir`. The behavior in this case is
    /// platform specific and unstable, and it's recommended to use
    /// [`canonicalize`] to get an absolute program path instead.
    ///
    /// [`canonicalize`]: std::fs::canonicalize
    pub fn current_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.command.current_dir(dir);
        self
    }

    /// Returns the path to the program that was given to [`LocalCommand::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_local_commands::LocalCommand;
    ///
    /// let cmd = LocalCommand::new("echo");
    /// assert_eq!(cmd.get_program(), "echo");
    /// ```
    pub fn get_program(&self) -> &OsStr {
        self.command.get_program()
    }

    /// Returns an iterator of the arguments that will be passed to the program.
    ///
    /// This does not include the path to the program as the first argument;
    /// it only includes the arguments specified with [`LocalCommand::arg`] and
    /// [`LocalCommand::args`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    /// use bevy_local_commands::LocalCommand;
    ///
    /// let mut cmd = LocalCommand::new("echo").arg("first").arg("second");
    /// let args: Vec<&OsStr> = cmd.get_args().collect();
    /// assert_eq!(args, &["first", "second"]);
    /// ```
    pub fn get_args(&self) -> CommandArgs<'_> {
        self.command.get_args()
    }

    /// Returns an iterator of the environment variables explicitly set for the child process.
    ///
    /// Environment variables explicitly set using [`LocalCommand::env`], [`LocalCommand::envs`], and
    /// [`LocalCommand::env_remove`] can be retrieved with this method.
    ///
    /// Note that this output does not include environment variables inherited from the parent
    /// process.
    ///
    /// Each element is a tuple key/value pair `(&OsStr, Option<&OsStr>)`. A [`None`] value
    /// indicates its key was explicitly removed via [`LocalCommand::env_remove`]. The associated key for
    /// the [`None`] value will no longer inherit from its parent process.
    ///
    /// An empty iterator can indicate that no explicit mappings were added or that
    /// [`LocalCommand::env_clear`] was called. After calling [`LocalCommand::env_clear`], the child process
    /// will not inherit any environment variables from its parent process.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    /// use bevy_local_commands::LocalCommand;
    ///
    /// let mut cmd = LocalCommand::new("ls").env("TERM", "dumb").env_remove("TZ");
    /// let envs: Vec<(&OsStr, Option<&OsStr>)> = cmd.get_envs().collect();
    /// assert_eq!(envs, &[
    ///     (OsStr::new("TERM"), Some(OsStr::new("dumb"))),
    ///     (OsStr::new("TZ"), None)
    /// ]);
    /// ```
    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.command.get_envs()
    }

    /// Returns the working directory for the child process.
    ///
    /// This returns [`None`] if the working directory will not be changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use bevy_local_commands::LocalCommand;
    ///
    /// let mut cmd = LocalCommand::new("ls");
    /// assert_eq!(cmd.get_current_dir(), None);
    /// cmd = cmd.current_dir("/bin");
    /// assert_eq!(cmd.get_current_dir(), Some(Path::new("/bin")));
    /// ```
    pub fn get_current_dir(&self) -> Option<&Path> {
        self.command.get_current_dir()
    }
}

impl From<Command> for LocalCommand {
    fn from(command: Command) -> Self {
        Self { command }
    }
}

impl From<LocalCommand> for Command {
    fn from(value: LocalCommand) -> Self {
        value.command
    }
}

impl Debug for LocalCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.command.fmt(f)
    }
}
