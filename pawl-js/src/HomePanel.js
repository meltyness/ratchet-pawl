import { useState } from 'react';

import SideBar from "./SideBar"
import UserList from "./UserList"
import DeviceList from "./DeviceList"
import WelcomeLanding from "./WelcomeLanding"

import { Home, Menu2, X } from 'tabler-icons-react';

export default function HomePanel(){
    const [isSideBarVisible, setIsSideBarVisible] = useState(false); 
    const [selectedPage, setSelectedPage] = useState("welcome-page");

    const goHome = () => {
        setSelectedPage("welcome-page");
    };

    const toggleSideBar = () => { 
        setIsSideBarVisible(!isSideBarVisible); 
    };

    const pageSelector = (requestedPage) => {
        toggleSideBar();
        setSelectedPage(requestedPage);
    };

    return (
        <div>
        <button onClick={goHome}>
            <Home />
        </button>
        <button onClick={toggleSideBar}> 
            {isSideBarVisible ? <X /> : <Menu2 />} 
        </button>
        {isSideBarVisible && <SideBar pageSetter={pageSelector}/>}
        {selectedPage === "user-list" ? <UserList /> :
         selectedPage === "device-list" ? <DeviceList /> : 
         selectedPage === "welcome-page" ? <WelcomeLanding /> :
         <div>fatalError</div>
        }
        </div>
    );
}