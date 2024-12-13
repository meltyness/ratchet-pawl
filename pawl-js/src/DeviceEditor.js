import { useState } from 'react';

export default function DeviceEditor({ initialNetworkId = '', editingNetworkId = false, addComplete = () => {}}) {
    const [networkid, setNetworkid] = useState(initialNetworkId);
    const [key, setKey] = useState('');
    const [confirmKey, setConfirmKey] = useState('');
    const [error, setError] = useState('');

    const handleSubmit = (event) => {
        event.preventDefault();
        if (key !== confirmKey) {
            setError('TACACS+ Keys do not match');
            return;
        }
        var data = new FormData();
        data.append('networkid', networkid);
        data.append('passhash', confirmKey);
        
        var xhr = new XMLHttpRequest();
        xhr.open('POST', 'adddevice', true);
        xhr.onload = function () {
            // do something to response
            console.log(this.responseText);
        };
        xhr.send(data);

        // Signal up to collection that we're done
        // or maybe nothing.
        addComplete(networkid);

        setError('');
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