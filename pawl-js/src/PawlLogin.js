import { Tool } from 'tabler-icons-react';
import { useState } from 'react';

export default function PawlLogin({loginComplete}) {
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState('');

    const handleSubmit = async (event) => {
        event.preventDefault();
        var data = new FormData();
        data.append('username', username);
        // TODO: Actually hash the password on the client.
        data.append('password', password);
        
        const response = await fetch('trylogin', {
            method: "POST",
            body: data
        });

        if (response.status == 200) {
            loginComplete();
        } else {
            setError("Please try again...");
        }
    };

    return (
        <div>
            <h1> <Tool /> Please login to Ratchet.  </h1>
            <form onSubmit={handleSubmit}>
                <div>
                    <label>Username:</label>
                    <input
                        type="text"
                        autocomplete="username"
                        value={username}
                        onChange={(e) => setUsername(e.target.value)}
                        required
                    />
                </div>
                <div>
                    <label>Password:</label>
                    <input
                        type="password"
                        autocomplete="password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        required
                    />
                </div>
                {error && <p style={{ color: 'red' }}>{error}</p>}
                <button type="submit">Login</button>
            </form>
        </div>
    );
}