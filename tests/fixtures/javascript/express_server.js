const express = require('express');
const app = express();

function handleGetUsers(req, res) {
    res.json([{ id: 1, name: 'John' }]);
}

function handleCreateUser(req, res) {
    validateUser(req.body);
    res.status(201).json({ id: 2 });
}

function validateUser(user) {
    if (!user.name) throw new Error('Name required');
}

function errorHandler(err, req, res, next) {
    res.status(500).json({ error: err.message });
}

app.get('/users', handleGetUsers);
app.post('/users', handleCreateUser);
app.use(errorHandler);

const startApp = () => {
    app.listen(3000, () => {
        console.log('Server started');
    });
};

module.exports = [app, startApp];
