import { useState, useEffect } from 'react';

import SideBar from "./SideBar"
import UserList from "./UserList"
import DeviceList from "./DeviceList"
import PawlLogin from "./PawlLogin"
import WelcomeLanding from "./WelcomeLanding"
import UserCmdPolicies from './UserCmdPolicies';

import { IconHome, IconMenu2, IconX } from '@tabler/icons-react';

export default function HomePanel(){
    const [isSideBarVisible, setIsSideBarVisible] = useState(false); 
    const [selectedPage, setSelectedPage] = useState("welcome-page");

    const init = async() => {
        const response = await fetch('logged');
        if (response.status != 200) {
            await goLogin();
        }
      };

    useEffect( () => { init() }, []);

    const goHome = () => {
        setSelectedPage("welcome-page");
    };

    const goLogin = async() => {
        // In this case, we're not authorized, wipe cached cookie.
        await cookieStore.delete("X-Ratchet-Auth-Token")
        setSelectedPage("pawl-login");
    };

    const forceLogout = async() => {
        await fetch('hangup');
    };

    const toggleSideBar = () => { 
        setIsSideBarVisible(!isSideBarVisible); 
    };

    const pageSelector = (requestedPage) => {
        toggleSideBar();
        if (requestedPage != "pawl-logout") {
            setSelectedPage(requestedPage);
        } else {
            forceLogout();
            goLogin();
        }
    };

    return (
        <div>
            
        <button onClick={goHome}>
            <IconHome />
        </button>

        <button onClick={toggleSideBar}> 
            {isSideBarVisible ? <IconX /> : <IconMenu2 />} 
        </button>

        {isSideBarVisible && <SideBar pageSetter={pageSelector}/>}

        {selectedPage === "user-cmd-policies" ? <UserCmdPolicies authorizedRedirect={goLogin}/> :
         selectedPage === "user-list" ? <UserList authorizedRedirect={goLogin}/> :
         selectedPage === "device-list" ? <DeviceList authorizedRedirect={goLogin}/> : 
         selectedPage === "pawl-login" ? <PawlLogin loginComplete={goHome}/> : 
         selectedPage === "welcome-page" ? <WelcomeLanding /> :
         <div>fatalError</div>
        }
        </div>
    );
}