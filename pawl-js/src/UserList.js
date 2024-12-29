import React, { useState, useEffect } from 'react';
import { IconX, IconEditCircle, IconUser, IconUserX, IconUserPlus, IconUsers } from '@tabler/icons-react';
import UserEditor from './UserEditor'; // Adjust the path as necessary

export default function UserList ({authorizedRedirect}) {
    const [users, setUsers] = useState([]);
    const [editingUserId, setEditingUserId] = useState(null);
    const [addingUser, setAddingUser] = useState(false);

    const init = async() => {
        const response = await fetch('getusers');
        if (response.status === 200) {
            const initUsers = await response.json();
            initUsers.forEach((obj, index) => {
                obj['id'] = index;
            });
            setUsers( users.concat(
                initUsers
            ));
        } else {
            authorizedRedirect();
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

        const success = (response.status === 200 || response.status === 410);
        return success;
    };

    const handleDelete = async(id) => {
        if (await sendRemoveUserRequest(id)) {
            setUsers(users.filter(user => user.id !== id));
        } else {
            alert("Error!"); // TODO: Better feedback.
        }
    };

    const handleFakeDelete = async(id) => {
        setUsers(users.map(usr => {if (usr.id === id) {usr.deleting = true} return usr;}));
    };

    const handleEdit = (id) => {
        setEditingUserId(id);
    };

    const handleCancelEdit = () => {
        setEditingUserId(null);
    };

    function createdUser(new_name) {
        setUsers(addUser(users, { username: new_name}));
        toggleAddingUser();
    }

    function addUser(users, newUser) {
        // TODO: I think this is a bug, when, uhhh.. something
        const maxId = users.reduce((max, user) => (user.id > max ? user.id : max), 0);
        newUser.id = maxId + 1;
        return [...users, newUser];
    }

    function toggleAddingUser() {
        setAddingUser(!addingUser);
    }

    return (
        <div>
            <h1><IconUsers /> Add or Edit Users!</h1>
            <p>These are users authorized to access network system consoles.</p>
            {users.length == 0 ? 
               <h3>Define some users to get started!</h3> : <nbsp></nbsp>
            }
            {users.map(user => (
                <div key={user.id}>
                    {editingUserId === user.id ? (
                        <div>
                            <UserEditor initialUsername={user.username} lockUsername={true} addComplete={handleCancelEdit}/>
                            <button onClick={() => handleCancelEdit()}><IconX /></button>
                        </div>
                    ) : (
                        <div>
                            <IconUser />
                            <span>{user.username}</span>
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
                    <UserEditor addComplete={createdUser}/>
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