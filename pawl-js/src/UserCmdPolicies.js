import React, { useState, useEffect } from 'react';
import { IconBracesOff, IconStackMiddle, IconLoader } from '@tabler/icons-react';

export default function UserCmdPolicies ({authorizedRedirect}) {
    const [fullPolicy, setFullPolicy] = useState("$\n\(\n\)\n");
    const [policyLoaded, setPolicyLoaded] = useState(false);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');

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

        var data = new FormData();
        data.append('0', fullPolicy);
        
        var updateType = 'pushpolicy';
        const response = await fetch(updateType, {
            method: "POST",
            body: data
        });

        if (response.status == 200) {
            setSuccess('Successfully updated!');
            setError('');
        } else if (response.status == 503) {
            alert('Caution: ratchet not responding, update may not take effect.');
        } else if (response.status == 401) {
            authorizedRedirect();
        } else {
            setError('Syntax Error, no changes made.');
            setSuccess('')
        }
    };

    const maintainValidPolicy = async(event) => {
        // nah just ram it in there
        setFullPolicy(event);
    }

    return (
        <div className="ratchet-editable-items-list">
            <h1><IconBracesOff /> Block User Commands!</h1>
            <p>
            Write a{" "}
            <a
                href="https://github.com/meltyness/ratchet?tab=readme-ov-file#command-authorization-policies"
                rel="noreferrer"
                target="_blank"
            >
                policy
            </a>{" "}
            to restrict what commands users can run.
            </p>
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
                    <div>
                        <label className="dev-editor-fields">Submit</label>
                        <button type="submit"><IconStackMiddle /></button>
                    </div>
                    <div>
                        <label className="dev-editor-fields">Response</label>
                        {error && <p style={{ color: 'red' }}>{error}</p>}
                        {success && <p style={{ color: 'green' }}>{success}</p>}
                    </div>
                </form>
            }
        </div>
    );
}
