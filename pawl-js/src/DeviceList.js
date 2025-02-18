import React, { useState, useEffect } from 'react';
import { IconLoader, IconX, IconEditCircle, IconNetwork, IconPencilPlus, IconTrash, IconRouter } from '@tabler/icons-react';

import DeviceEditor from './DeviceEditor'; // Adjust the path as necessary

export default function DeviceList ({authorizedRedirect}) {
    const [devices, setDevices] = useState([]);
    const [devsLoaded, setDevsLoaded] = useState(false);
    const [editingDeviceId, setEditingDeviceId] = useState(null);
    const [addingDevice, setAddingDevice] = useState(false);

    const init = async() => {
        setDevsLoaded(false);
        const response = await fetch('getdevs');
        setDevsLoaded(true);
        if (response.status === 200) {
            const initDevs = await response.json();
            initDevs.forEach((obj, index) => {
                obj['id'] = index;
            });
            setDevices( devices.concat(
                initDevs
            ));
        } else {
            authorizedRedirect();
        }
      };

    useEffect( () => { init() }, []);

    const sendRemoveDevRequest = async(id) => {
        var data = new FormData();
        data.append('network_id', devices.find(dev => dev.id === id).network_id);
        data.append('key', "");
        const response = await fetch('rmdev', {
            method: "POST",
            body: data,
        });

        return response.status;
    };

    const handleDelete = async(id) => {
        var res = await sendRemoveDevRequest(id);
        if (res == 200 || res == 410) {
            setDevices(devices.filter(dev => dev.id !== id));
        } else if (res == 503) {
            alert('Caution: ratchet not responding to pawl, update may not take effect.');
            setDevices(devices.filter(dev => dev.id !== id));
        } else {
            alert("Error!"); // TODO: Better feedback.
        }
    };

    const handleFakeDelete = async(id) => {
        setDevices(devices.map(dev => {if (dev.id === id) {dev.deleting = true} return dev;}));
    };

    const handleEdit = (id) => {
        setEditingDeviceId(id);
        if (addingDevice) {      // Adding and editing are mutually exclusive
            toggleAddingDevice();
        }
    };

    const handleCancelEdit = () => {
        setEditingDeviceId(null);
    };

    function createdDevice(new_network) {
        if(new_network) {
            setDevices(addDevice(devices, { network_id: new_network}));
            toggleAddingDevice();
        } else {
            // Cancellation from within edit workflow...?
            toggleAddingDevice();
        }
    }

    function addDevice(devices, newDev) {
        const maxId = devices.reduce((max, dev) => (dev.id > max ? dev.id : max), 0);
        newDev.id = maxId + 1;
        return [...devices, newDev];
    }

    function toggleAddingDevice() {
        if (!addingDevice) {      // Adding and editing are mutually exclusive
            handleCancelEdit();
        }
        setAddingDevice(!addingDevice);
    }
    // TODO: Make this a routing table.
    return (
        <div className="ratchet-editable-items-list">
            <h1><IconNetwork /> Add or Edit Trusted Systems!</h1>
            <p> These are network machines that can talk to ratchet, and check passwords. </p>
            {devices.length == 0  && devsLoaded ? 
               <h3>Define some systems to get started!</h3> : !devsLoaded &&  <IconLoader />
            }
            {devices.map(dev => (
                <div key={dev.id}>
                    {editingDeviceId === dev.id ? (
                        <div>
                            <DeviceEditor initialNetworkId={dev.network_id} editingNetworkId={true} addComplete={handleCancelEdit}/>
                            <IconRouter />
                            <span className="ratchet-listed-object">{dev.network_id}</span>
                        </div>
                    ) : (
                        <div>
                            <IconRouter />
                            <span className="ratchet-listed-object">{dev.network_id}</span>
                            <button onClick={() => handleEdit(dev.id)}><IconEditCircle size={16}/></button>
                            { dev.deleting ? (<button onClick={() => handleDelete(dev.id)}>⚡<IconTrash size={16}/></button>) :
                                             (<button onClick={() => handleFakeDelete(dev.id)}><IconTrash size={16}/></button>)
                            }
                        </div>
                    )}
                </div>
            ))}
        <hr />
        {addingDevice ? (
                <div>
                    <DeviceEditor addComplete={createdDevice}/>
                    <button onClick={toggleAddingDevice}><IconX /></button>
                </div>
            ) : (
                <div>
                    <button onClick={toggleAddingDevice}><IconPencilPlus /></button>
                </div>
            )
        }
        
        </div>
    );
}