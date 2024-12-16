import React, { useState, useEffect } from 'react';
import { X, EditCircle, Network, PencilPlus, Trash, Router } from 'tabler-icons-react';

import DeviceEditor from './DeviceEditor'; // Adjust the path as necessary

export default function DeviceList () {
    const [devices, setDevices] = useState([]);
    const [editingDeviceId, setEditingDeviceId] = useState(null);
    const [addingDevice, setAddingDevice] = useState(false);

    const init = async() => {
        const response = await fetch('getdevs');
        const initDevs = await response.json();
        initDevs.forEach((obj, index) => {
            obj['id'] = index;
        });
        setDevices( devices.concat(
            initDevs
        ));
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
            <h1><Network /> Add or Edit Trusted Systems!</h1>
            <p> These are network machines that can talk to ratchet, and check passwords. </p>
            <p> ⚡⚡ There's no confirmation dialog on deletion, it just deletes them! ⚡⚡</p>
            {devices.length == 0 ? 
               <h3>Define some systems to get started!</h3> : <nbsp></nbsp>
            }
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