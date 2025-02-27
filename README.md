# Saddle

**Saddle** is an asynchronous Rust library designed to simplify the management
of user seats in a Linux environment. It provides a Rust-native, future-based
API for obtaining and releasing user sessions, opening and closing devices
securely, and monitoring session state changes.

## Features

- **Async-first API**: Built with Tokio and designed for asynchronous
  applications
- **Session Management**: Obtain, release, and monitor user sessions
- **Device Access**: Open and close devices securely without requiring root
  privileges
- **VT Switching**: Support for switching between virtual terminals
- **Event Streams**: Monitor session state changes with async streams

## Supported Backends

Currently, Saddle uses the freedesktop `login1` interface as its backend, which
is compatible with both `systemd-logind` and `elogind` systems.

Future releases will include support for `seatd` IPC interface.

## Examples

For a comprehensive example demonstrating how to use Saddle with libinput for
handling input devices and implementing VT switching, see the
[VT switcher example](examples/complex.rs).

## License

This project is licensed under the
[Apache-2.0 License](http://www.apache.org/licenses/LICENSE-2.0). For more
information, please see the [LICENSE](LICENSE) file.
