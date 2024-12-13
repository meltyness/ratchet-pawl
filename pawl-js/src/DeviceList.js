import React, { useState, useEffect } from 'react';
import { X, EditCircle, Network, PencilPlus, Trash, Router } from 'tabler-icons-react';

import DeviceEditor from './DeviceEditor'; // Adjust the path as necessary

const defaultDevices = [
    { id: 1, network_id: '127.0.0.0/24' },
    { id: 2, network_id: '127.0.0.0/28' },
    { id: 3, network_id: '127.0.0.0/32' },
];

export default function DeviceList () {
    const [devices, setDevices] = useState(defaultDevices);
    const [editingDeviceId, setEditingDeviceId] = useState(null);
    const [addingDevice, setAddingDevice] = useState(false);

    function useEffect() {
        var xhr = new XMLHttpRequest();
        xhr.open('GET', 'getdevices', true);
        xhr.onload = function () {
            // do something to response
            console.log(this.responseText);
        };
        
        xhr.send(data);
    }

    const handleDelete = (id) => {
        setDevices(devices.filter(dev => dev.id !== id));
        console.log(`Deleted device with ID: ${id}`);
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

    return (
        <div>
            <h1><Network /> Add or Edit Trusted Systems!</h1>
            <p> These are network machines that can talk to ratchet, and check passwords. </p>
            {devices.map(dev => (
                <div key={dev.id}>
                    {editingDeviceId === dev.id ? (
                        <div>
                            <DeviceEditor initialNetworkId={dev.network_id} editingNetworkId={true} addComplete={handleCancelEdit}/>
                            <button onClick={() => handleCancelEdit()}><X /></button>
                        </div>
                    ) : (
                        <div>
                            <Router />
                            <span>{dev.network_id}</span>
                            <button onClick={() => handleEdit(dev.id)}><EditCircle size={16}/></button>
                            <button onClick={() => handleDelete(dev.id)}><Trash size={16}/></button>
                        </div>
                    )}
                </div>
            ))}
        <hr />
        {addingDevice ? (
                <div>
                    <DeviceEditor addComplete={createdDevice}/>
                    <button onClick={toggleAddingDevice}><X /></button>
                </div>
            ) : (
                <div>
                    <button onClick={toggleAddingDevice}><PencilPlus /></button>
                </div>
            )
        }
        
        </div>
    );
}