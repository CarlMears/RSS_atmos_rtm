from typing import Optional, final

import numpy as np
from numpy.typing import NDArray

@final
class AtmoParameters:
    """Atmospheric radiative parameters.

    This is the output of the RTM.
    """

    @property
    def tran(self) -> NDArray[np.float32]:
        """Transmissivity, from 0 to 1.

        Dimensioned as (`num_points`, `num_freq`).
        """
    @property
    def tb_up(self) -> NDArray[np.float32]:
        """Upwelling TB, in K.

        Dimensioned as (`num_points`, `num_freq`).
        """
    @property
    def tb_down(self) -> NDArray[np.float32]:
        """Downwelling TB, in K.

        Dimensioned as (`num_points`, `num_freq`).
        """

def compute_rtm(
    pressure: NDArray[np.float32],
    temperature: NDArray[np.float32],
    height: NDArray[np.float32],
    specific_humidity: NDArray[np.float32],
    liquid_content: NDArray[np.float32],
    surface_temperature: NDArray[np.float32],
    surface_height: NDArray[np.float32],
    surface_dewpoint: NDArray[np.float32],
    surface_pressure: NDArray[np.float32],
    incidence_angle: NDArray[np.float32],
    frequency: NDArray[np.float32],
    num_threads: Optional[int],
) -> AtmoParameters:
    """Compute the radiative transfer model for the atmosphere.

    Most of the inputs are numpy arrays and are either 1d or 2d. The
    `pressure` parameter is the pressure levels in hPa and has shape
    (`num_levels`, ). It is treated as a constant (i.e., not a function of
    `num_points`).

    `pressure`: pressure levels, in hPa

    The following are input profiles and have shape (`num_points`,
    `num_levels`):

    `temperature`: physical temperature in K

    `height`: geometric height above the geoid in m

    `specific_humidity`: specific humidity in kg/kg

    `liquid_content`: liquid water content (from clouds) in kg/kg

    The following are surface parameters and have shape (`num_points`, ):

    `surface_temperature`: 2 meter air temperature in K

    `surface_height`: geopotential height at the surface in m

    `surface_dewpoint`: 2 meter dewpoint in K

    `surface_pressure`: surface pressure in hPa

    The following are RTM parameters and have shape (`num_freq`, ):

    `incidence_angle`: Earth incidence angle in degrees

    `frequency`: microwave frequency in GHz

    The returned atmospheric parameters are each dimensioned as (`num_points`,
    `num_freq`).

    The number of worker threads is controlled by `num_threads`. It must be a
    positive integer, or `None` to automatically choose the number of threads.
    """
