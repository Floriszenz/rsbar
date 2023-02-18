# `rsbar` - bar code reader

**DISCLAIMER**: This fork is a personal side project that has the goal of translating this C-based library to Rust.
It is only a hobby project and has mainly the goal to experiment, get familiar with,
and practice concepts of the Rust programming language, work with Function Foreign Interfaces (FFI),
and refactoring in general. It is therefore not planned to (re-)distribute this library in any way.

RSBar is a fork and migration of the ZBar Bar Code Reader library to Rust. ZBar Bar Code Reader is an open source software suite for reading bar codes from various sources, such as video streams, image files and raw intensity sensors. It supports EAN-13/UPC-A, UPC-E, EAN-8, Code 128, Code 93, Code 39, Codabar, Interleaved 2 of 5, QR Code and SQ Code.

Included with the library are basic applications for decoding captured bar code images and using a video device (e.g. webcam) as a bar code scanner. For application developers, language bindings are included for C, C++, and Python 2 as well as GUI widgets for GTK and PyGTK 2.0.

Zbar also supports sending the scanned codes via dbus, allowing its integration with other applications.

License information can be found in `COPYING`.

## Building

See `INSTALL.md` for generic configuration and build instructions.

Please notice that at least autotools related packages and a C compiler are needed, in order to generate the configure script.

So, on Debian, at least those packages are needed: autoconf autopoint pkg-config libtool gcc make

If you have installed all needed dependencies, all you need to do to build the original C-based library and executables is to run:

```
autoreconf -vfi
./configure
make
```

After building the original library, you can build the `rsbar` library and executables using:

```
cargo build
```

The scanner/decoder library itself only requires a few standard library functions which should be available almost anywhere.

The zbarcam program uses the video4linux API (v4l1 or v4l2) to access the video device. This interface is part of the linux kernel, a 3.16 kernel or upper is recommended for full support. More information is available at:

-   <http://www.linuxtv.org/wiki/>

`pkg-config` is used to locate installed libraries. You should have installed `pkg-config` if you need any of the remaining components. pkg-config may be obtained from:

-   <http://pkg-config.freedesktop.org/>

## Gtk Widget

The GTK+ widget requires GTK+-2.x or GTK+3.x. You will need GTK+ if you
would like to use or develop a GTK+ GUI application with an integrated bar
code scanning widget. GTK+ may be obtained from:

-   <http://www.gtk.org/>

## Python widgets

**Python 2 legacy Gtk widget**

The PyGTK 2.0/pygobject 2.0 wrapper for the GTK+ 2.x widget requires Python 2,
PyGTK. You will need to enable both pygtk2 and gtk2 if you would like to use
or develop a Python 2 GUI application with an integrated bar code scanning
widget. PyGTK may be obtained from:

-   <http://www.pygtk.org/>

**Python 2 or 3 GIR Gtk widget**

The GObject Introspection (GIR) wrapper for GTK+ widget is compatible with
PyGObject, with works with either Python version 2 or 3. You will need to
enable both Gtk and Python in order to use or develop a Python application
with an integrated bar code scanning and webcam support. In order to build
it, you need the required dependencies for GIR development. The actual
package depends on the distribution. On Fedora, it is `pygobject3-devel`.
On Debian/Ubuntu, it is `libgirepository1.0-dev` and `gir1.2-gtk-3.0`.
While GIR builds with Gtk2, It is strongly recommended to use GTK+
version 3.x, as there are known issues with version 2.x and GIR, with
will likely make it to fail. A test script can be built and run with:
`make check-gi`. Instructions about how to use are GIR on Python are
available at:

-   <https://pygobject.readthedocs.io/en/latest/>

**Python bindings**

The Python bindings require Python 2 or 3 and provide only non-GUI functions.
You will need Python and PIL or Pillow if you would like to scan images or
video directly using Python. Python is available from:

-   <http://python.org/>

## Running

`make install` will install the library and application programs. Run `zbarcam` to start the video scanner. Use `cargo run --bin rsbar-img <file>` to decode a saved image file.

Check the manual to find specific options for each program.

## dbus Testing

In order to test if dbus is working, you could use:

```shell
$ dbus-monitor --system interface=org.linuxtv.Zbar1.Code
```

or build the test programs with:

```shell
$ make test_progs
```

And run:

```shell
$ ./test/test_dbus
```

With that, running this command on a separate shell:

```shell
$ cargo run --features dbus --bin rsbar-img examples/code-128.png
CODE-128:https://github.com/mchehab/zbar
scanned 1 barcode symbols from 1 images in 0.01 seconds
```

Will produce this output at test_dbus shell window:

```shell
Waiting for Zbar events
Type = CODE-128
Value = https://github.com/mchehab/zbar
```

Note for WSL users: You may need to start dbus manually for this to work. All you have to do is running `service dbus start`.

-   If it's complaining about missing permissions, run it with `sudo`.
-   If you run `rsbar-img` and it throws a `Name Error(Connection ":1.1" is not allowed to own the service "org.linuxtv.zbar" due to security policies in the configuration file)` warning, make sure you have added the file `dbus/org.linuxtv.Zbar.conf` to `/etc/dbus-1/system.d/`). After copying the file to this directory it should work.
