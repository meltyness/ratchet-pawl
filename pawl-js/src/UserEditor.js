import { useState } from 'react';
import { IconX } from '@tabler/icons-react';

export default function UserEditor({ initialUsername = '', 
                                     lockUsername = false,
                                     authorizedRedirect,
                                     addComplete = () => {}}) {
    const [username, setUsername] = useState(initialUsername);
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [error, setError] = useState('');

    const handleSubmit = async (event) => {
        event.preventDefault();
        if (password !== confirmPassword) {
            setError('Passwords do not match');
            return;
        }
        var data = new FormData();
        data.append('username', username);
        // TODO: Actually hash the password on the client.
        data.append('passhash', confirmPassword);
        
        var updateType = !lockUsername ? 'adduser' : 'edituser';
        const response = await fetch(updateType, {
            method: "POST",
            body: data
        });

        if (response.status == 200) {
            addComplete(username);
        } else if (response.status == 503) {
            alert('Caution: ratchet not responding, update may not take effect.'); 
            addComplete(username);
        } else if (response.status == 401) {
            authorizedRedirect();
        } else {
            if (lockUsername) {
                setError('User no longer exists!');
            } else {
                setError('User already exists!');
            }
        }
    };

    const handleCancel = () => {
        addComplete();
    };

    let actionName;
    if(lockUsername) {
        actionName = "Save Changes";
    } else {
        actionName = "Add User";
    }

    return (
        <div className="ratchet-editor-popover">
            {lockUsername ? ( <h2>Edit User</h2>) : (<h2>Add User</h2>)}
            <form onSubmit={handleSubmit}>
                <div>
                    <label className="editor-fields">Username:</label>
                    <input
                        type="text"
                        autocomplete="username"
                        value={username}
                        onChange={(e) => setUsername(e.target.value)}
                        disabled={lockUsername}
                        required
                    />
                </div>
                <div>
                    <label className="editor-fields">Password:</label>
                    <input
                        type="password"
                        autocomplete="new-password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        required
                    />
                </div>
                <div>
                    <label className="editor-fields">Confirm:</label>
                    <input
                        type="password"
                        autocomplete="new-password"
                        value={confirmPassword}
                        onChange={(e) => setConfirmPassword(e.target.value)}
                        required
                    />
                </div>
                {error && <p style={{ color: 'red' }}>{error}</p>}
                <button type="submit">{actionName}</button>
            </form>
            <button onClick={() => handleCancel()}><IconX /></button>
        </div>
    );
}