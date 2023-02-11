bin_PROGRAMS += zbarcam/zbarcam
zbarcam_zbarcam_SOURCES = zbarcam/zbarcam.c
zbarcam_zbarcam_LDADD = zbar/libzbar.la
zbarcam_zbarcam_CPPFLAGS = $(AM_CPPFLAGS)
# automake bug in "monolithic mode"?
CLEANFILES += zbarcam/.libs/zbarcam

if HAVE_GTK
if !WIN32
bin_PROGRAMS += zbarcam/zbarcam-gtk
zbarcam_zbarcam_gtk_SOURCES = zbarcam/zbarcam-gtk.c zbarcam/scan_video.c
zbarcam_zbarcam_gtk_CPPFLAGS = $(GTK_CFLAGS) $(AM_CPPFLAGS)
zbarcam_zbarcam_gtk_LDADD = $(GTK_LIBS) gtk/libzbargtk.la zbar/libzbar.la \
	$(AM_LDADD)

endif
endif

if WIN32
zbarcam_zbarcam_SOURCES += zbarcam/zbarcam.rc
zbarcam_zbarcam_LDADD += zbarcam/zbarcam-rc.o @LTLIBINTL@

if WITH_DIRECTSHOW
zbarcam_zbarcam_CPPFLAGS += -DDIRECTSHOW
endif

endif
