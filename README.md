# Saddle

**Saddle** is an asynchronous library designed to simplify the management of
user seats in a Linux environment. It provides a easy to use interface for
obtaining and releasing user sessions, which can be utilized to open and close
devices securely.

## Current Features

- **Session Management:** Obtain, release and pause user sessions
- **Device Management:** Open and close devices tied to a user session
- **VT Switching:** Support for switching between virtual terminals

## Supported Backend

At present, **Saddle** leverages the freedesktop `login1` interface as its
backend, which is compatible with both `logind` and `elogind` systems.

In the future, we plan to extend support to the `seatd` IPC interface,

## License

This project is licensed under the
[Apache-2.0 License](http://www.apache.org/licenses/LICENSE-2.0). For more
information, please see the [LICENSE](LICENSE) file.
