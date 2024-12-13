import React, { useState, useEffect } from 'react';
import { X, EditCircle, User, UserX, UserPlus, Users } from 'tabler-icons-react';
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
            <h1><Users /> Add or Edit Users!</h1>
            <p>These are users authorized to access network system consoles.</p>
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