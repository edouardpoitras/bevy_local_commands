use std::{
    io::{self, prelude::*, BufWriter},
    process::{Child, ChildStdin},
};

use bevy::{prelude::*, tasks::Task};

use crate::{Pid, ProcessOutputBuffer};

#[derive(Debug, Component)]
pub struct Process {
    pub(crate) process: Child,
    pub(crate) reader_task: Task<()>,
    pub(crate) output_buffer: ProcessOutputBuffer,
    pub(crate) stdin_writer: BufWriter<ChildStdin>,
    pub(crate) state: ProcessState,
}

/// Keep track of the state of the running process.
///
/// Running - Process is running.
/// Error - Process errored out. Allows for retry logic to kick in. Otherwise state moves to Done
/// Done - Process has completed. Final state, allows for cleanup logic.
///
/// This gives more room for addons to interact with the process without interfering with each other.
#[derive(Debug, PartialEq)]
pub enum ProcessState {
    Running,
    Error,
    Done(ProcessDone),
}

/// Keeps track of the final state of the process.
///
/// Killed - Process was killed. Allows for cleanup logic.
/// Failed - Process failed permanently. Assumes retries exhausted. Allows for cleanup logic.
/// Succeeded - Process succeeded. Allows for cleanup logic.
#[derive(Debug, PartialEq)]
pub enum ProcessDone {
    Killed,
    Failed,
    Succeeded,
}

impl Process {
    pub fn id(&self) -> Pid {
        self.process.id()
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.process.kill()
    }

    /// Write a string to the process stdin.
    ///
    /// See [`Process::println`] for a version which adds a newline (`\n`) to the end of the string.
    ///
    /// Here's how you can write "Hello world!" to a process that has just been started:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_local_commands::Process;
    /// fn provide_input(mut query: Query<&mut Process, Added<Process>>) {
    ///     for mut process in query.iter_mut() {
    ///         process.print("Hello world!").unwrap();
    ///     }
    /// }
    /// ```
    ///
    /// If you want more control, you can also use the `write!` macro.
    /// Just keep in mind that this might not flush the input buffer directly,
    /// so your process might receive the output later.
    /// You can also manually flush the buffer when you have written all your input.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_local_commands::Process;
    /// // To use the macro, we need to import `Write`
    /// use std::io::Write;
    ///
    /// fn provide_input(mut query: Query<&mut Process, Added<Process>>) {
    ///     for mut process in query.iter_mut() {
    ///         let name = "world";
    ///         write!(&mut process, "Hello {name}!").unwrap();
    ///         // Make sure that the process receives the input
    ///         process.flush().unwrap();
    ///     }
    /// }
    /// ```
    pub fn print(&mut self, input: &str) -> Result<(), io::Error> {
        self.write_all(input.as_bytes())?;
        self.flush()
    }

    /// Write a string, terminated by a newline (`\n`) to the process stdin.
    ///
    /// See [`Process::print`] for a version without the newline.
    ///
    /// Here's how you can write "Hello world!" to a process that has just been started:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_local_commands::Process;
    /// fn provide_input(mut query: Query<&mut Process, Added<Process>>) {
    ///     for mut process in query.iter_mut() {
    ///         process.println("Hello world!").unwrap();
    ///     }
    /// }
    /// ```
    ///
    /// If you want more control, you can also use the `writeln!` macro.
    /// Just keep in mind that this might not flush the input buffer directly,
    /// so your process might receive the output later.
    /// You can also manually flush the buffer when you have written all your input.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_local_commands::Process;
    /// // To use the macro, we need to import `Write`
    /// use std::io::Write;
    ///
    /// fn provide_input(mut query: Query<&mut Process, Added<Process>>) {
    ///     for mut process in query.iter_mut() {
    ///         let name = "world";
    ///         writeln!(&mut process, "Hello {name}!").unwrap();
    ///         // Make sure that the process receives the input
    ///         process.flush().unwrap();
    ///     }
    /// }
    /// ```
    pub fn println(&mut self, input: &str) -> Result<(), io::Error> {
        self.write_all(input.as_bytes())?;
        self.write_all(b"\n")?;
        self.flush()
    }
}

impl Write for Process {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdin_writer.write(buf)
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.stdin_writer.write_all(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdin_writer.flush()
    }
}
