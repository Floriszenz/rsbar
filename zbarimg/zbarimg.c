/*------------------------------------------------------------------------
 *  Copyright 2007-2010 (c) Jeff Brown <spadix@users.sourceforge.net>
 *
 *  This file is part of the ZBar Bar Code Reader.
 *
 *  The ZBar Bar Code Reader is free software; you can redistribute it
 *  and/or modify it under the terms of the GNU Lesser Public License as
 *  published by the Free Software Foundation; either version 2.1 of
 *  the License, or (at your option) any later version.
 *
 *  The ZBar Bar Code Reader is distributed in the hope that it will be
 *  useful, but WITHOUT ANY WARRANTY; without even the implied warranty
 *  of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Lesser Public License for more details.
 *
 *  You should have received a copy of the GNU Lesser Public License
 *  along with the ZBar Bar Code Reader; if not, write to the Free
 *  Software Foundation, Inc., 51 Franklin St, Fifth Floor,
 *  Boston, MA  02110-1301  USA
 *
 *  http://sourceforge.net/projects/zbar
 *------------------------------------------------------------------------*/

#include "config.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef HAVE_UNISTD_H
#include <unistd.h>
#endif
#ifdef HAVE_SYS_TIMES_H
#include <sys/times.h>
#endif
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif
#include <assert.h>

#include <zbar.h>

#ifdef ENABLE_NLS
#include <libintl.h>
#include <locale.h>
#define _(string) gettext(string)
#else
#define _(string) string
#endif

#define N_(string) string

#ifdef HAVE_GRAPHICSMAGICK
#include <wand/wand_api.h>
#endif

#ifdef HAVE_IMAGEMAGICK
#ifdef HAVE_IMAGEMAGICK7
#include <MagickWand/MagickWand.h>
#else
#include <wand/MagickWand.h>
#endif

/* ImageMagick frequently changes API names - just use the original
 * (more stable?) names to match GraphicsMagick
 */
#define InitializeMagick(f) MagickWandGenesis()
#define DestroyMagick	    MagickWandTerminus
#define MagickSetImageIndex MagickSetIteratorIndex

/* in 6.4.5.4 MagickGetImagePixels changed to MagickExportImagePixels.
 * (still not sure this check is quite right...
 *  how does MagickGetAuthenticImagePixels fit in?)
 * ref http://bugs.gentoo.org/247292
 */
#if MagickLibVersion > 0x645
#define MagickGetImagePixels MagickExportImagePixels
#endif
#endif

static const char *note_usage = N_(
    "usage: zbarimg [options] <image>...\n"
    "\n"
    "scan and decode bar codes from one or more image files\n"
    "\n"
    "options:\n"
    "    -h, --help      display this help text\n"
    "    --version       display version information and exit\n"
    "    --polygon       output points delimiting code zone with decoded symbol data\n"
    "    -q, --quiet     minimal output, only print decoded symbol data\n"
    "    -v, --verbose   increase debug output level\n"
    "    --verbose=N     set specific debug output level\n"
    "    -d, --display   enable display of following images to the screen\n"
    "    -D, --nodisplay disable display of following images (default)\n"
    "    --xml, --noxml  enable/disable XML output format\n"
    "    --raw           output decoded symbol data without converting charsets\n"
    "    -1, --oneshot   exit after scanning one bar code\n"
    "    -S<CONFIG>[=<VALUE>], --set <CONFIG>[=<VALUE>]\n"
    "                    set decoder/scanner <CONFIG> to <VALUE> (or 1)\n"
    // FIXME overlay level
    "\n");

#ifdef HAVE_DBUS
static const char *note_usage2 =
    N_("    --nodbus        disable dbus message\n");
#endif

static const char *warning_not_found_head = N_(
    "\n"
    "WARNING: barcode data was not detected in some image(s)\n"
    "Things to check:\n"
    "  - is the barcode type supported? Currently supported symbologies are:\n");

static const char *warning_not_found_tail = N_(
    "  - is the barcode large enough in the image?\n"
    "  - is the barcode mostly in focus?\n"
    "  - is there sufficient contrast/illumination?\n"
    "  - If the symbol is split in several barcodes, are they combined in one "
    "image?\n"
    "  - Did you enable the barcode type?\n"
    "    some EAN/UPC codes are disabled by default. To enable all, use:\n"
    "    $ zbarimg -S*.enable <files>\n"
    "    Please also notice that some variants take precedence over others.\n"
    "    Due to that, if you want, for example, ISBN-10, you should do:\n"
    "    $ zbarimg -Sisbn10.enable <files>\n"
    "\n");

static const char *xml_head =
    "<barcodes xmlns='http://zbar.sourceforge.net/2008/barcode'>\n";
static const char *xml_foot = "</barcodes>\n";

static int notfound = 0, exit_code = 0;
static int num_images = 0, num_symbols = 0;
static int xmllvl  = 0;
static int polygon = 0;
static int oneshot = 0;
static int binary  = 0;

char *xmlbuf	   = NULL;
unsigned xmlbuflen = 0;

static zbar_processor_t *processor = NULL;

static inline int dump_error(MagickWand *wand)
{
    char *desc;
    ExceptionType severity;
    desc = MagickGetException(wand, &severity);

    if (severity >= FatalErrorException)
	exit_code = 2;
    else if (severity >= ErrorException)
	exit_code = 1;
    else
	exit_code = 0;

    static const char *sevdesc[] = { "WARNING", "ERROR", "FATAL" };
    fprintf(stderr, "%s: %s\n", sevdesc[exit_code], desc);

    MagickRelinquishMemory(desc);
    return (exit_code);
}

static int scan_image(const char *filename)
{
    if (exit_code == 3)
	return (-1);

    int found	       = 0;
    MagickWand *images = NewMagickWand();
    if (!MagickReadImage(images, filename) && dump_error(images))
	return (-1);

    unsigned seq, n = MagickGetNumberImages(images);
    for (seq = 0; seq < n; seq++) {
	if (exit_code == 3)
	    return (-1);

	if (!MagickSetImageIndex(images, seq) && dump_error(images))
	    return (-1);

	zbar_image_t *zimage = zbar_image_create();
	assert(zimage);
	zbar_image_set_format(zimage, zbar_fourcc('Y', '8', '0', '0'));

	int width  = MagickGetImageWidth(images);
	int height = MagickGetImageHeight(images);
	zbar_image_set_size(zimage, width, height);

	// extract grayscale image pixels
	// FIXME color!! ...preserve most color w/422P
	// (but only if it's a color image)
	size_t bloblen	    = width * height;
	unsigned char *blob = malloc(bloblen);
	zbar_image_set_data(zimage, blob, bloblen, zbar_image_free_data);

	if (!MagickGetImagePixels(images, 0, 0, width, height, "I", CharPixel,
				  blob))
	    return (-1);

	if (xmllvl == 1) {
	    xmllvl++;
	    printf("<source href='%s'>\n", filename);
	}

	zbar_process_image(processor, zimage);

	// output result data
	const zbar_symbol_t *sym = zbar_image_first_symbol(zimage);
	for (; sym; sym = zbar_symbol_next(sym)) {
	    zbar_symbol_type_t typ = zbar_symbol_get_type(sym);
	    unsigned len	   = zbar_symbol_get_data_length(sym);
	    if (typ == ZBAR_PARTIAL)
		continue;
	    else if (xmllvl <= 0) {
		if (!xmllvl)
		    printf("%s:", zbar_get_symbol_name(typ));
		if (polygon) {
		    int p;
		    if (zbar_symbol_get_loc_size(sym) > 0)
			printf("%+d,%+d", zbar_symbol_get_loc_x(sym,0), zbar_symbol_get_loc_y(sym,0));
		    for (p = 1; p < zbar_symbol_get_loc_size(sym); p++)
			printf(" %+d,%+d", zbar_symbol_get_loc_x(sym,p), zbar_symbol_get_loc_y(sym,p));
		    printf(":");
		}
		if (len &&
		    fwrite(zbar_symbol_get_data(sym), len, 1, stdout) != 1) {
		    exit_code = 1;
		    return (-1);
		}
	    } else {
		if (xmllvl < 3) {
		    xmllvl++;
		    printf("<index num='%u'>\n", seq);
		}
		zbar_symbol_xml(sym, &xmlbuf, &xmlbuflen);
		if (fwrite(xmlbuf, xmlbuflen, 1, stdout) != 1) {
		    exit_code = 1;
		    return (-1);
		}
	    }
	    found++;
	    num_symbols++;

	    if (!binary) {
		if (oneshot) {
		    if (xmllvl >= 0)
			printf("\n");
		    break;
		} else
		    printf("\n");
	    }
	}
	if (xmllvl > 2) {
	    xmllvl--;
	    printf("</index>\n");
	}
	fflush(stdout);

	zbar_image_destroy(zimage);

	num_images++;
	if (zbar_processor_is_visible(processor)) {
	    int rc = zbar_processor_user_wait(processor, -1);
	    if (rc < 0 || rc == 'q' || rc == 'Q')
		exit_code = 3;
	}
    }

    if (xmllvl > 1) {
	xmllvl--;
	printf("</source>\n");
    }

    if (!found)
	notfound++;

    DestroyMagickWand(images);
    return (0);
}

int usage(int rc, const char *msg, const char *arg)
{
    FILE *out = (rc) ? stderr : stdout;
    if (msg) {
	fprintf(out, "%s", msg);
	if (arg)
	    fprintf(out, "%s", arg);
	fprintf(out, "\n\n");
    }
    fprintf(out, "%s", _(note_usage));
#ifdef HAVE_DBUS
    fprintf(out, "%s", _(note_usage2));
#endif
    return (rc);
}

static inline int parse_config(const char *cfgstr, const char *arg)
{
    if (!cfgstr || !cfgstr[0])
	return (usage(1, "ERROR: need argument for option: ", arg));

    if (zbar_processor_parse_config(processor, cfgstr))
	return (usage(1, "ERROR: invalid configuration setting: ", cfgstr));

    if (!strcmp(cfgstr, "binary"))
	binary = 1;

    return (0);
}

int main(int argc, const char *argv[])
{
    // option pre-scan
    int quiet = 0;
#ifdef HAVE_DBUS
    int dbus = 1;
#endif
    int display = 0;
    int i, j;

#ifdef ENABLE_NLS
    setlocale(LC_ALL, "");
    bindtextdomain(PACKAGE, LOCALEDIR);
    textdomain(PACKAGE);
#endif

    for (i = 1; i < argc; i++) {
	const char *arg = argv[i];
	if (arg[0] != '-' || !arg[1])
	    // first pass, skip images
	    num_images++;
	else if (arg[1] != '-')
	    for (j = 1; arg[j]; j++) {
		if (arg[j] == 'S') {
		    if (!arg[++j] && ++i >= argc)
			/* FIXME parse check */
			return (parse_config("", "-S"));
		    break;
		}
		switch (arg[j]) {
		case 'h':
		    return (usage(0, NULL, NULL));
		case 'q':
		    quiet = 1;
		    break;
		case '1':
		    oneshot = 1;
		    break;
		case 'v':
		    zbar_increase_verbosity();
		    break;
		case 'd':
		    display = 1;
		    break;
		case 'D':
		    break;
		default:
		    return (
			usage(1, "ERROR: unknown bundled option: -", arg + j));
		}
	    }
	else if (!strcmp(arg, "--help"))
	    return (usage(0, NULL, NULL));
	else if (!strcmp(arg, "--version")) {
	    printf("%s\n", PACKAGE_VERSION);
	    return (0);
	} else if (!strcmp(arg, "--quiet")) {
	    quiet   = 1;
	    argv[i] = NULL;
	} else if (!strcmp(arg, "--oneshot"))
	    oneshot = 1;
	else if (!strcmp(arg, "--verbose"))
	    zbar_increase_verbosity();
	else if (!strncmp(arg, "--verbose=", 10))
	    zbar_set_verbosity(strtol(argv[i] + 10, NULL, 0));
	else if (!strcmp(arg, "--nodbus"))
#ifdef HAVE_DBUS
	    dbus = 0;
#else
	    ; /* silently ignore the option */
#endif
	else if (!strcmp(arg, "--display"))
	    display++;
	else if (!strcmp(arg, "--xml")) {
	    if (xmllvl >= 0)
		xmllvl = 1;
	} else if (!strcmp(arg, "--noxml")) {
	    if (xmllvl > 0)
		xmllvl = 0;
	} else if (!strcmp(arg, "--raw")) {
	    // RAW mode takes precedence
	    xmllvl = -1;
	} else if (!strcmp(arg, "--polygon")) {
	    polygon = 1;
	} else if (!strcmp(arg, "--nodisplay") || !strcmp(arg, "--set") ||
		   !strncmp(arg, "--set=", 6))
	    continue;
	else if (!strcmp(arg, "--")) {
	    num_images += argc - i - 1;
	    break;
	} else
	    return (usage(1, "ERROR: unknown option: ", arg));
    }

    if (!num_images)
	return (usage(1, "ERROR: specify image file(s) to scan", NULL));
    num_images = 0;

    InitializeMagick("zbarimg");

    processor = zbar_processor_create(0);
    assert(processor);

#ifdef HAVE_DBUS
    zbar_processor_request_dbus(processor, dbus);
#endif

    if (zbar_processor_init(processor, NULL, display)) {
	zbar_processor_error_spew(processor, 0);
	return (1);
    }

    if (xmllvl > 0) {
	printf("%s", xml_head);
    }

    for (i = 1; i < argc; i++) {
	const char *arg = argv[i];
	if (!arg)
	    continue;

	if (binary)
	    xmllvl = -1;

#ifdef _WIN32
	if (xmllvl == -1) {
	    _setmode(_fileno(stdout), _O_BINARY);
	} else {
	    _setmode(_fileno(stdout), _O_TEXT);
	}
#endif

	if (arg[0] != '-' || !arg[1]) {
	    if (scan_image(arg))
		return (exit_code);
	} else if (arg[1] != '-')
	    for (j = 1; arg[j]; j++) {
		if (arg[j] == 'S') {
		    if ((arg[++j]) ? parse_config(arg + j, "-S") :
					   parse_config(argv[++i], "-S"))
			return (1);
		    break;
		}
		switch (arg[j]) {
		case 'd':
		    zbar_processor_set_visible(processor, 1);
		    break;
		case 'D':
		    zbar_processor_set_visible(processor, 0);
		    break;
		}
	    }
	else if (!strcmp(arg, "--display"))
	    zbar_processor_set_visible(processor, 1);
	else if (!strcmp(arg, "--nodisplay"))
	    zbar_processor_set_visible(processor, 0);

	else if (!strcmp(arg, "--set")) {
	    if (parse_config(argv[++i], "--set"))
		return (1);
	} else if (!strncmp(arg, "--set=", 6)) {
	    if (parse_config(arg + 6, "--set="))
		return (1);
	} else if (!strcmp(arg, "--"))
	    break;
    }
    for (i++; i < argc; i++)
	if (scan_image(argv[i]))
	    return (exit_code);

    /* ignore quit during last image */
    if (exit_code == 3)
	exit_code = 0;

    if (xmllvl > 0) {
	printf("%s", xml_foot);
	fflush(stdout);
    }

    if (xmlbuf)
	free(xmlbuf);

    if (num_images && !quiet && xmllvl <= 0) {
	fprintf(stderr, "scanned %d barcode symbols from %d images",
		num_symbols, num_images);
#ifdef HAVE_SYS_TIMES_H
#ifdef HAVE_UNISTD_H
	long clk_tck = sysconf(_SC_CLK_TCK);
	struct tms tms;
	if (clk_tck > 0 && times(&tms) >= 0) {
	    double secs = tms.tms_utime + tms.tms_stime;
	    secs /= clk_tck;
	    fprintf(stderr, " in %.2g seconds\n", secs);
	}
#endif
#endif
	fprintf(stderr, "\n");
	if (notfound) {
	    fprintf(stderr, "%s", _(warning_not_found_head));
#if ENABLE_EAN == 1
	    fprintf(
		stderr,
		_("\t. EAN/UPC (EAN-13, EAN-8, EAN-2, EAN-5, UPC-A, UPC-E, ISBN-10, ISBN-13)\n"));
#endif
#if ENABLE_DATABAR == 1
	    fprintf(stderr, _("\t. DataBar, DataBar Expanded\n"));
#endif
#if ENABLE_CODE128 == 1
	    fprintf(stderr, _("\t. Code 128\n"));
#endif
#if ENABLE_CODE93 == 1
	    fprintf(stderr, _("\t. Code 93\n"));
#endif
#if ENABLE_CODE39 == 1
	    fprintf(stderr, _("\t. Code 39\n"));
#endif
#if ENABLE_CODABAR == 1
	    fprintf(stderr, _("\t. Codabar\n"));
#endif
#if ENABLE_I25 == 1
	    fprintf(stderr, _("\t. Interleaved 2 of 5\n"));
#endif
#if ENABLE_QRCODE == 1
	    fprintf(stderr, _("\t. QR code\n"));
#endif
#if ENABLE_SQCODE == 1
	    fprintf(stderr, _("\t. SQ code\n"));
#endif
#if ENABLE_PDF417 == 1
	    fprintf(stderr, _("\t. PDF 417\n"));
#endif
	    fprintf(stderr, "%s", _(warning_not_found_tail));
	}
    }
    if (num_images && notfound && !exit_code)
	exit_code = 4;

    zbar_processor_destroy(processor);
    DestroyMagick();
    return (exit_code);
}
