const supertest = require('supertest');
const pm = require("../test/pm");
const baseUrl = supertest(pm.environment.get('server'));
const post = async function (apiEndPoint, request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')

        .send(pm.substitute(request_body));
};

module.exports = {
    post: post
}