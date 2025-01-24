# Core Async, Streams, and Utility Dependencies

   - tokio
       + Purpose: Tokio is an async runtime for Rust, enabling asynchronous I/O, timers, and task scheduling.
       + Features:
            "full": Includes everything Tokio offers, such as TCP/UDP, timers, file operations, and sync primitives.

   - tokio-util
        + Purpose: Provides utilities for working with tokio, such as codecs for encoding/decoding frames or handling asynchronous streams.
        + Features:
            "codec": Includes utilities like Framed to help work with protocols (e.g., parsing lines from a stream).

   - futures
       + Purpose: A foundational crate for async programming, offering traits like Future, Stream, and utilities for managing them.

   - tokio-stream
       + Purpose: Bridges tokio with futures, making it easier to work with Stream combinators in the Tokio ecosystem.


# Tracing and Logging Dependencies

   - tracing
      + Purpose: A structured logging library for asynchronous applications. It tracks execution contexts across async tasks, making debugging easier.

   - tracing-subscriber
       + Purpose: A set of utilities to collect and manage tracing spans and events.
       + Features:
            "env-filter": Allows filtering logs dynamically via environment variables.
            "parking_lot": Uses a more efficient locking mechanism for performance.
            "fmt": Outputs tracing logs in a human-readable format.
            "tracing-log": Converts logs from the log crate into tracing spans.
            "std": Enables support for the standard library.
            "ansi": Adds color support to logs.

   - tracing-appender
       + Purpose: Helps write tracing logs to files, rolling them over periodically.


# Randomness and Performance Utilities

   - fastrand
       + Purpose: A small, fast random number generator for use in Rust applications. Useful for generating random numbers where cryptographic strength isn't required.

   - dashmap
       + Purpose: A concurrent hash map (thread-safe and lock-free) for high-performance shared data structures in multi-threaded environments.

   - anyhow
       + Purpose: Simplifies error handling by providing an easy way to construct and propagate error messages.


# Terminal UI and Formatting

   - ratatui
       + Purpose: A terminal UI library for building rich, text-based interfaces in the terminal. (ratatui is a fork of the popular tui crate.)

   - crossterm
       + Purpose: A cross-platform library for handling terminal interactions, such as input, output, and events.
       + Features:
            "event-stream": Enables async event handling, such as reading keypresses or terminal resize events.

   - tui-textarea
       + Purpose: A specialized library for adding editable text areas to terminal user interfaces, often used with ratatui.

   - textwrap
       + Purpose: Provides utilities to wrap text within a given width, commonly used in terminal applications.

   - compact_str
       + Purpose: A memory-efficient string type that stores short strings inline (rather than on the heap). Useful for performance-sensitive code.
