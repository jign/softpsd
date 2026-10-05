"""Use the PSB side limit for psd-tools pixel decoding of version 2 files."""

from contextlib import contextmanager
from psd_tools.api import utils


@contextmanager
def pixel_limits(psd):
    previous = utils.MAX_DIMENSION_PSD
    try:
        if psd.version == 2:
            utils.MAX_DIMENSION_PSD = 300_000
        yield
    finally:
        utils.MAX_DIMENSION_PSD = previous
