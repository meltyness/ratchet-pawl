import { useState, useEffect } from 'react';

export default function SideBar({pageSetter=() => {}}) {
    const [showLogin, setShowLogin] = useState(true);

    const init = async() => {
        var cookies = await cookieStore.getAll();
        var discoveredCookie = cookies.find( (x) => x.name == "X-Ratchet-Auth-Token" );
        if (discoveredCookie && discoveredCookie.expires > Date.now()) {
            setShowLogin(false);
        } else {
            setShowLogin(true);
        }
    };

    useEffect( () => { init() }, []);

    const setPage = (desiredPage) => {
        pageSetter(desiredPage);
    };

    return (
        <div className="sidebar-container">
            <div className="sidebar-div">
                <label className="sidebar-item" onClick={() => {setPage("device-list")}}>
                    Device Editor
                </label>
            </div>
            <div className="sidebar-div">
                <label className="sidebar-item" onClick={() => {setPage("user-list")}}>
                    User Editor
                </label>
            </div>
            { showLogin &&
                <div className="sidebar-div">
                    <label className="sidebar-item" onClick={() => {setPage("pawl-login")}}>
                        Login
                    </label>
                </div>
            }
        </div>
    );

}