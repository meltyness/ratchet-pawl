import React, { useState, useEffect } from 'react';
import { IconX, IconEditCircle, IconNetwork, IconPencilPlus, IconTrash, IconRouter } from '@tabler/icons-react';

import DeviceEditor from './DeviceEditor'; // Adjust the path as necessary

export default function DeviceList ({authorizedRedirect}) {
    const [devices, setDevices] = useState([]);
    const [editingDeviceId, setEditingDeviceId] = useState(null);
    const [addingDevice, setAddingDevice] = useState(false);

    const init = async() => {
        const response = await fetch('getdevs');
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

        const success = (response.status === 200 || response.status === 410);
        return success;
    };

    const handleDelete = async(id) => {
        if (await sendRemoveDevRequest(id)) {
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
    };

    const handleCancelEdit = () => {
        setEditingDeviceId(null);
    };

    function createdDevice(new_network) {
        setDevices(addDevice(devices, { network_id: new_network}));
        toggleAddingDevice();
    }

    function addDevice(devices, newDev) {
        const maxId = devices.reduce((max, dev) => (dev.id > max ? dev.id : max), 0);
        newDev.id = maxId + 1;
        return [...devices, newDev];
    }

    function toggleAddingDevice() {
        setAddingDevice(!addingDevice);
    }
    // TODO: Make this a routing table.
    return (
        <div>
            <h1><IconNetwork /> Add or Edit Trusted Systems!</h1>
            <p> These are network machines that can talk to ratchet, and check passwords. </p>
            {devices.length == 0 ? 
               <h3>Define some systems to get started!</h3> : <nbsp></nbsp>
            }
            {devices.map(dev => (
                <div key={dev.id}>
                    {editingDeviceId === dev.id ? (
                        <div>
                            <DeviceEditor initialNetworkId={dev.network_id} editingNetworkId={true} addComplete={handleCancelEdit}/>
                            <button onClick={() => handleCancelEdit()}><IconX /></button>
                        </div>
                    ) : (
                        <div>
                            <IconRouter />
                            <span>{dev.network_id}</span>
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