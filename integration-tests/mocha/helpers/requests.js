const supertest = require('supertest');
const pm = require("../test/pm");
const baseUrl = supertest(pm.environment.get('server'));
const post = async function (apiEndPoint, request_body, token) {
     let req =  baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json');

     if (token) {
         req = req  .set('Authorization', pm.substitute('Bearer ' + token));

     }
        return req.send(pm.substitute(request_body));
};

const put = async function (apiEndPoint, request_body, token) {
    let req =  baseUrl
        .put(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json');

    if (token) {
        req = req  .set('Authorization', pm.substitute('Bearer ' + token));

    }
    return req.send(pm.substitute(request_body));
};

const get = async function (apiEndPoint, token) {
    let req =  baseUrl
        .get(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json');

    if (token) {
        req = req  .set('Authorization', pm.substitute('Bearer ' + token));

    }
    return req.send();
};

module.exports = {
    get,
    post,
    put
}
