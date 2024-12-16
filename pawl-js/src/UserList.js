import React, { useState, useEffect } from 'react';
import { X, EditCircle, User, UserX, UserPlus, Users } from 'tabler-icons-react';
import UserEditor from './UserEditor'; // Adjust the path as necessary

export default function UserList () {
    const [users, setUsers] = useState([]);
    const [editingUserId, setEditingUserId] = useState(null);
    const [addingUser, setAddingUser] = useState(false);

    const init = async() => {
        const response = await fetch('getusers');
        const initUsers = await response.json();
        initUsers.forEach((obj, index) => {
            obj['id'] = index;
        });
        setUsers( users.concat(
            initUsers
        ));
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
            <h1><Users /> Add or Edit Users!</h1>
            <p>These are users authorized to access network system consoles.</p>
            <p> ⚡⚡ There's no confirmation dialog on removal, it just removes them! ⚡⚡</p>
            {users.length == 0 ? 
               <h3>Define some users to get started!</h3> : <nbsp></nbsp>
            }
            {users.map(user => (
                <div key={user.id}>
                    {editingUserId === user.id ? (
                        <div>
                            <UserEditor initialUsername={user.username} lockUsername={true} addComplete={handleCancelEdit}/>
                            <button onClick={() => handleCancelEdit()}><X /></button>
                        </div>
                    ) : (
                        <div>
                            <User />
                            <span>{user.username}</span>
                            <button onClick={() => handleEdit(user.id)}><EditCircle size={16} /></button>
                            <button onClick={() => handleDelete(user.id)}><UserX size={16}/></button>
                        </div>
                    )}
                </div>
            ))}
        <hr />
        {addingUser ? (
                <div>
                    <UserEditor addComplete={createdUser}/>
                    <button onClick={toggleAddingUser} ><X /></button>
                </div>
            ) : (
                <div>
                    <button onClick={toggleAddingUser}><UserPlus /></button>
                </div>
            )
        }
        
        </div>
    );
}