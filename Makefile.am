ACLOCAL_AMFLAGS = -I config
AM_CFLAGS += -DLOCALEDIR=\"$(localedir)\"
AM_CXXFLAGS += -DLOCALEDIR=\"$(localedir)\"
bin_PROGRAMS =
check_PROGRAMS =
EXTRA_PROGRAMS =
lib_LTLIBRARIES =
pyexec_LTLIBRARIES =
CLEANFILES =
DISTCLEANFILES =
MAINTAINERCLEANFILES =
BUILT_SOURCES =
EXTRA_DIST =
PHONY = $(SUBDIRS)

pkgconfigdir = $(libdir)/pkgconfig
pkgconfig_DATA = zbar.pc

dist_doc_DATA = COPYING HACKING.md INSTALL.md LICENSE.md NEWS.md README.md TODO.md

include $(srcdir)/include/Makefile.am.inc

SUBDIRS = zbar

zbar/libzbar.la:
	$(MAKE) -C @builddir@/zbar libzbar.la

if HAVE_MAGICK
include $(srcdir)/zbarimg/Makefile.am.inc
endif
if HAVE_VIDEO
include $(srcdir)/zbarcam/Makefile.am.inc
endif
if HAVE_PYTHON
include $(srcdir)/python/Makefile.am.inc
endif
if HAVE_GTK
SUBDIRS += gtk
pkgconfig_DATA += zbar-gtk.pc

gtk/libzbargtk.la:
	$(MAKE) -C @builddir@/gtk libzbargtk.la

gtk/zbarmarshal.h:
	$(MAKE) -C @builddir@/gtk zbarmarshal.h

gtk/ZBar-1.0.typelib:
	$(MAKE) -C $(srcdir)/gtk ZBar-1.0.typelib

if HAVE_PYGTK2
include $(srcdir)/pygtk/Makefile.am.inc
endif
endif
include $(srcdir)/test/Makefile.am.inc
if HAVE_DOC
include $(srcdir)/doc/Makefile.am.inc
endif

if HAVE_DBUS
dbusconfdir = @DBUS_CONFDIR@
dbusconf_DATA = $(srcdir)/dbus/org.linuxtv.Zbar.conf
endif

EXTRA_DIST += zbar.ico zbar.nsi dbus/org.linuxtv.Zbar.conf

EXTRA_DIST += examples/*.png examples/sha1sum \
    examples/upcrpc.py examples/upcrpc.pl \
    examples/scan_image.c examples/scan_image.cpp examples/scan_image.vcproj

if WIN32
dist_doc_DATA += README-windows.md
pkgdata_DATA = $(srcdir)/python/test/barcode.png \
    $(srcdir)/examples/scan_image.cpp $(srcdir)/examples/scan_image.vcproj

%-rc.o: %.rc
	$(RC) $(DEFS) $(DEFAULT_INCLUDES) $(INCLUDES) \
	$(AM_CPPFLAGS) $(CPPFLAGS) -o $@ $<
%-rc.lo: %.rc
	$(LIBTOOL) --tag=RC $(AM_LIBTOOLFLAGS) $(LIBTOOLFLAGS) \
	--mode=compile $(RC) $(DEFS) $(DEFAULT_INCLUDES) $(INCLUDES) \
	$(AM_CPPFLAGS) $(CPPFLAGS) -o $@ $<

# install to tmp dest and run NSIS to generate installer
dist-nsis: html-local
	test ! -e _nsis || test -d _nsis && rm -rf _nsis
	mkdir _nsis
	tmpinst=`cd _nsis && pwd | sed -e 's,^[^:\\/]:[\\/],/,'` \
	  && $(MAKE) $(AM_MAKEFLAGS) DESTDIR=$$tmpinst prefix=/ install
	cp zbar/.libs/libzbar-0.dll.def _nsis/lib/libzbar-0.def
	cp -r doc/html _nsis/share/doc/zbar/
	$(WINEXEC) lib.exe /machine:x86 /def:_nsis/lib/libzbar-0.def /out:_nsis/lib/libzbar-0.lib
	cd _nsis && \
	  makensis -NOCD -V2 -DVERSION=$(VERSION) $(builddir)/zbar.nsi
	@ls -l _nsis/zbar-$(VERSION)-setup.exe

PHONY += dist-nsis
endif

SUBDIRS += .

archive:
	git archive --format=tar --prefix=zbar-$(VERSION)/ -o zbar-$(VERSION).tar $(VERSION)
	bzip2 zbar-$(VERSION).tar

.PHONY : $(PHONY) archive

dist-hook:
	rm -f $(distdir)/debian $(distdir)/travis
