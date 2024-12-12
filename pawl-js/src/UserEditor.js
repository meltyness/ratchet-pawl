import { useState } from 'react';

export default function UserEditor({ initialUsername = '', lockUsername = false, addComplete = () => {}}) {
    const [username, setUsername] = useState(initialUsername);
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [error, setError] = useState('');

    const handleSubmit = (event) => {
        event.preventDefault();
        if (password !== confirmPassword) {
            setError('Passwords do not match');
            return;
        }
        var data = new FormData();
        data.append('username', username);
        data.append('passhash', confirmPassword);
        
        var xhr = new XMLHttpRequest();
        xhr.open('POST', 'adduser', true);
        xhr.onload = function () {
            // do something to response
            console.log(this.responseText);
        };
        xhr.send(data);

        // Signal up to collection that we're done
        // or maybe nothing.
        addComplete(username);

        setError('');
    };

    let actionName;
    if(lockUsername) {
        actionName = "Edit User";
    } else {
        actionName = "Add User";
    }

    return (
        <div>
            {lockUsername ? ( <h2>Edit User</h2>) : (<h2>Add User</h2>)}
            <form onSubmit={handleSubmit}>
                <div>
                    <label>Username:</label>
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
                    <label>Password:</label>
                    <input
                        type="password"
                        autocomplete="new-password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        required
                    />
                </div>
                <div>
                    <label>Confirm Password:</label>
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
        </div>
    );
}