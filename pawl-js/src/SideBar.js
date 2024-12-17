import { useState } from 'react';

export default function SideBar({pageSetter=() => {}}) {

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
            <div className="sidebar-div">
                <label className="sidebar-item" onClick={() => {setPage("pawl-login")}}>
                    Login
                </label>
            </div>
        </div>
    );

}