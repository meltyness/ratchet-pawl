import React, { useState, useEffect } from 'react';
import UserEditor from './UserEditor'; // Adjust the path as necessary

const defaultUsers = [
    { id: 1, username: 'Albert' },
    { id: 2, username: 'Feynman' },
    { id: 3, username: 'Heisenberg' },
];

export default function UserList () {
    const [users, setUsers] = useState(defaultUsers);
    const [editingUserId, setEditingUserId] = useState(null);
    const [addingUser, setAddingUser] = useState(false);

    function useEffect() {
        var xhr = new XMLHttpRequest();
        xhr.open('GET', 'getusers', true);
        xhr.onload = function () {
            // do something to response
            console.log(this.responseText);
        };
        
        xhr.send(data);
    }

    const handleDelete = (id) => {
        setUsers(users.filter(user => user.id !== id));
        console.log(`Deleted user with ID: ${id}`);
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
        const maxId = users.reduce((max, user) => (user.id > max ? user.id : max), 0);
        newUser.id = maxId + 1;
        return [...users, newUser];
    }

    function toggleAddingUser() {
        setAddingUser(!addingUser);
    }

    return (
        <div>
            {users.map(user => (
                <div key={user.id}>
                    {editingUserId === user.id ? (
                        <div>
                            <UserEditor initialUsername={user.username} lockUsername={true} addComplete={handleCancelEdit}/>
                            <button onClick={() => handleCancelEdit()}>Cancel</button>
                        </div>
                    ) : (
                        <div>
                            <span>{user.username}</span>
                            <button onClick={() => handleEdit(user.id)}>Edit</button>
                            <button onClick={() => handleDelete(user.id)}>Delete</button>
                        </div>
                    )}
                </div>
            ))}
        
        {addingUser ? (
                <div>
                    <UserEditor addComplete={createdUser}/>
                    <button onClick={toggleAddingUser} >Cancel</button>
                </div>
            ) : (
                <div>
                    <button onClick={toggleAddingUser}>Add...</button>
                </div>
            )
        }
        
        </div>
    );
}