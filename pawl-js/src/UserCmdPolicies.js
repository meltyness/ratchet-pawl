import React, { useState, useEffect } from 'react';
import { IconBracesOff, IconStackMiddle, IconLoader } from '@tabler/icons-react';

export default function UserCmdPolicies ({authorizedRedirect}) {
    const [fullPolicy, setFullPolicy] = useState("$\n\(\n\)\n");
    const [policyLoaded, setPolicyLoaded] = useState(false);

    const init = async() => {
        setPolicyLoaded(false);
        const response = await fetch('getpolicy');
        if (response.status === 200) {
            const initFullPolicy = await response.text();
            setFullPolicy( 
                initFullPolicy
            );
            setPolicyLoaded(true);
        } else {
            await authorizedRedirect();
        }
    };

    useEffect( () => { init() }, []);

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
        } else if (response.status == 503) {
            alert('Caution: ratchet not responding, update may not take effect.');
            addComplete(networkid);
        } else if (response.status == 401) {
            authorizedRedirect();
        } else {
            if (editingNetworkId) {
                setError('System no longer exists!');
            } else {
                setError('System already exists!');
            }
        }
    };

    const maintainValidPolicy = async(event) => {
        // nah just ram it in there
        setFullPolicy(event);
    }

    return (
        <div className="ratchet-editable-items-list">
            <h1><IconBracesOff /> Block User Commands!</h1>
            <p> Write a policy to restrict what commands users can run. </p>
            {!policyLoaded ? <IconLoader /> :
                <form onSubmit={handleSubmit}>
                    <div>
                        <label className="dev-editor-fields">Full Policy</label>
                        <textarea
                            className="ratchet-policy-definition"
                            type="text"
                            value={fullPolicy}
                            onChange={(e) => maintainValidPolicy(e.target.value)}
                            required
                        />
                    </div>
                    <button type="submit"><IconStackMiddle /></button>
                </form>
            }
        </div>
    );
}
