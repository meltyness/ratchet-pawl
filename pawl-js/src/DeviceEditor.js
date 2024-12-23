import { useState } from 'react';

export default function DeviceEditor({ initialNetworkId = '', editingNetworkId = false, addComplete = () => {}}) {
    const [networkid, setNetworkid] = useState(initialNetworkId);
    const [key, setKey] = useState('');
    const [confirmKey, setConfirmKey] = useState('');
    const [error, setError] = useState('');

    const handleSubmit = async (event) => {
        event.preventDefault();
        if (key !== confirmKey) {
            setError('Passwords do not match');
            return;
        }
        var data = new FormData();
        data.append('network_id', networkid);
        // TODO: Actually hash the password on the client.
        data.append('key', confirmKey);
        
        var updateType = !editingNetworkId ? 'adddev' : 'editdev';
        const response = await fetch(updateType, {
            method: "POST",
            body: data
        });

        if (response.status == 200) {
            addComplete(networkid);
        } else {
            if (editingNetworkId) {
                setError('System no longer exists!');
            } else {
                setError('System already exists!');
            }
        }
    };

    let actionName;
    if(editingNetworkId) {
        actionName = "Save Changes";
    } else {
        actionName = "Add Device";
    }

    return (
        <div>
            {editingNetworkId ? ( <h2>Edit System</h2>) : (<h2>Add System</h2>)}
            <form onSubmit={handleSubmit}>
                <div>
                    <label>Network ID:</label>
                    <input
                        type="text"
                        autocomplete="networkid"
                        value={networkid}
                        onChange={(e) => setNetworkid(e.target.value)}
                        disabled={editingNetworkId}
                        required
                    />
                </div>
                <div>
                    <label>TACACS+ Key:</label>
                    <input
                        type="password"
                        autocomplete="new-password"
                        value={key}
                        onChange={(e) => setKey(e.target.value)}
                        required
                    />
                </div>
                <div>
                    <label>Confirm Key:</label>
                    <input
                        type="password"
                        autocomplete="new-password"
                        value={confirmKey}
                        onChange={(e) => setConfirmKey(e.target.value)}
                        required
                    />
                </div>
                {error && <p style={{ color: 'red' }}>{error}</p>}
                <button type="submit">{actionName}</button>
            </form>
        </div>
    );
}