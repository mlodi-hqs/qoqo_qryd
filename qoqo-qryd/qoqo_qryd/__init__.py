# Copyright © 2021-2022 HQS Quantum Simulations GmbH.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
# in compliance with the License. You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software distributed under the License
# is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
# or implied. See the License for the specific language governing permissions and limitations under
# the License.

""" `QRydDemo <https://thequantumlaend.de/qryddemo/>`_ support for the `qoqo <https://github.com/HQSquantumsimulations/qoqo>`_ quantum toolkit.

.. autosummary::
    :toctree: generated/

    qryd_devices
    api_devices
    pragma_operations
    SimulatorBackend
    APIBackend
    tweezer_devices
    emulator_devices

"""

from .qoqo_qryd import *  # type: ignore
from .api_devices import *  # type: ignore
from .pragma_operations import *  # type: ignore
from .qryd_devices import *  # type: ignore
from .tweezer_devices import *  # type: ignore

devices = qryd_devices  # type: ignore
