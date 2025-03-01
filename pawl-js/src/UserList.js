import React, { useState, useEffect } from 'react';
import { IconLoader, IconX, IconEditCircle, IconUser, IconUserX, IconUserPlus, IconUsers } from '@tabler/icons-react';
import UserEditor from './UserEditor'; // Adjust the path as necessary

export default function UserList ({authorizedRedirect}) {
    const [users, setUsers] = useState([]);
    const [usersLoaded, setUsersLoaded] = useState(false);
    const [editingUserId, setEditingUserId] = useState(null);
    const [addingUser, setAddingUser] = useState(false);

    const init = async() => {
        setUsersLoaded(false);
        const response = await fetch('getusers');
        if (response.status === 200) {
            const initUsers = await response.json();
            initUsers.forEach((obj, index) => {
                obj['id'] = index;
            });
            setUsers( users.concat(
                initUsers
            ));
            setUsersLoaded(true);
        } else {
            await authorizedRedirect();
        }
      };

    useEffect( () => { init() }, []);

    const sendRemoveUserRequest = async(id) => {
        var data = new FormData();
        data.append('username', users.find(user => user.id === id).username);
        const response = await fetch('rmuser', {
            method: "POST",
            body: data,
        });

        return response.status;
    };

    const checkLoggedIn = async() => {
        const response = await fetch('logged');
        if (response.status != 200) {
            await authorizedRedirect();
        }
    }

    const handleDelete = async(id) => {
        var res = await sendRemoveUserRequest(id);
        if (res == 200 || res == 410) {
            setUsers(users.filter(user => user.id !== id));
            await checkLoggedIn();
        } else if (res == 401) {
            await authorizedRedirect();
        } else if (res == 503) {
            alert('Caution: ratchet not responding to pawl, update may not take effect.');
            setUsers(users.filter(user => user.id !== id));
        } else {
            alert("Error!"); // TODO: Better feedback.
        }
    };

    const handleFakeDelete = async(id) => {
        await checkLoggedIn();
        setUsers(users.map(usr => {if (usr.id === id) {usr.deleting = true} return usr;}));
    };

    const handleEdit = (id) => {
        checkLoggedIn();
        setEditingUserId(id);
        if (addingUser) {       // Adding and editing are mutually exclusive
            toggleAddingUser();
        }
    };

    const handleCancelEdit = () => {
        checkLoggedIn();
        setEditingUserId(null);
    };

    function createdUser(new_name) {
        checkLoggedIn();
        if(new_name) {
            setUsers(addUser(users, { username: new_name}));
            toggleAddingUser();
        } else {
            // Cancellation from within edit workflow...?
            toggleAddingUser();
        }
    }

    function addUser(users, newUser) {
        // TODO: I think this is a bug, when, uhhh.. something
        const maxId = users.reduce((max, user) => (user.id > max ? user.id : max), 0);
        newUser.id = maxId + 1;
        return [...users, newUser];
    }

    function toggleAddingUser() {
        checkLoggedIn();
        if (!addingUser) {
            handleCancelEdit();
        }
        setAddingUser(!addingUser);
    }

    return (
        <div className="ratchet-editable-items-list">
            <h1><IconUsers /> Add or Edit Users!</h1>
            <p>These are users authorized to access network system consoles.</p>
            {users.length == 0 && usersLoaded ? 
               <h3>Define some users to get started!</h3> : !usersLoaded && <IconLoader />
            }
            {users.map(user => (
                <div key={user.id}>
                    {editingUserId === user.id ? (
                        <div>
                            <UserEditor initialUsername={user.username} lockUsername={true} addComplete={handleCancelEdit} authorizedRedirect={authorizedRedirect}/>
                            <IconUser />
                            <span className="ratchet-listed-object">{user.username}</span>
                        </div>
                    ) : (
                        <div>
                            <IconUser />
                            <span className="ratchet-listed-object">{user.username}</span>
                            <button onClick={() => handleEdit(user.id)}><IconEditCircle size={16} /></button>
                            { user.deleting ? (<button disabled={users.length <= 1} onClick={() => handleDelete(user.id)}>⚡<IconUserX size={16}/></button>) :
                                              (<button disabled={users.length <= 1} onClick={() => handleFakeDelete(user.id)}><IconUserX size={16}/></button>)
                            }
                        </div>
                    )}
                </div>
            ))}
        <hr />
        {addingUser ? (
                <div>
                    <UserEditor addComplete={createdUser} authorizedRedirect={authorizedRedirect}/>
                    <button onClick={toggleAddingUser} ><IconX /></button>
                </div>
            ) : (
                <div>
                    <button onClick={toggleAddingUser}><IconUserPlus /></button>
                </div>
            )
        }
        
        </div>
    );
}