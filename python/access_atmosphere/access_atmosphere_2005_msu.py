import logging

from dataclasses import dataclass
from datetime import datetime, date, timedelta
from pathlib import Path
from time import perf_counter_ns

import numpy as np
from netCDF4 import Dataset, getlibversion, num2date
from numpy.typing import NDArray

import os

print(os.getcwd())
import access_atmosphere
print(dir(access_atmosphere))
print(access_atmosphere.__file__)

import access_atmosphere.process 
print(access_atmosphere.process.__file__)




output_path = Path('/mnt/m/Obs4MIPs/atmospheric-rtm/tbs')
path_to_rtm = Path('/mnt/flux-write/ERA5/1hr')
start_date = date(2005, 1, 3)
end_date = date(2005, 12, 31)
date_to_do = start_date
time_indices = np.arange(0, 24, 1)
one_pass=True
workers=12
overwrite=False

    # Set up logging
log_level = logging.INFO
log_fmt = "{asctime} {levelname} {message}"

log_datefmt = "%Y-%m-%d %H:%M:%S%z"
logging.basicConfig(style="{", format=log_fmt, datefmt=log_datefmt, level=log_level)


while date_to_do <= end_date:
    print(date_to_do)
    surf_file = path_to_rtm / 'surface'/ f'y{date_to_do.year:04d}' / f'm{date_to_do.month:02d}' / f'era5_surface_{date_to_do}.nc'
    level_file = path_to_rtm / 'profiles' / f'y{date_to_do.year:04d}' / f'm{date_to_do.month:02d}' / f'era5_levels_{date_to_do}.nc'
    output_file = output_path / 'tbs_msu' / f'y{date_to_do.year:04d}' / f'm{date_to_do.month:02d}' / f'era5_tbs_{date_to_do}.msu.ch2.nc'

    output_file.parent.mkdir(parents=True, exist_ok=True)

    if output_file.exists():
        if not overwrite:
            logging.info(f"{output_file} already exists, skipping")
            date_to_do += timedelta(days=1)
            continue
        else:
            logging.info(f"{output_file} already exists, overwriting")

    logging.info(f"ERA5 surface file: {surf_file}")
    logging.info(f"ERA5 levels file: {level_file}")
    logging.info(f"RTM output file: {output_file}")
    
    print(f'Computing Tbs for {date_to_do}')
    print(f'{datetime.now()}')

    one_pass=False
    access_atmosphere.process.convert_all(surf_file,
                                      level_file,
                                      output_file,
                                      one_pass=one_pass,
                                      time_indices=time_indices,
                                      workers=workers,
                                      satellite='MSU'
    )
    
    date_to_do += timedelta(days=1)

