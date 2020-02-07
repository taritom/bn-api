const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');
const debug = require("debug");
var log=debug('bn-api');
const user = require("../../helpers/user");
const {get} = require("../../helpers/requests");

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/users/{{__del_user_id}}';

var response;
var response2;

const del = async function () {
    return baseUrl
        .del(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{token}}'))
        .send();
};

describe('User disable', function () {
    before(async function () {
        var token = await user.registerAndLogin("xxxx", true);
        var me = await user.me(token);
        pm.environment.set("__del_user_id", me.user.id);
        response = await del();
        log(response.request.header);
        log(response.request.url);
        log(response.request._data);
        log(response.request.method);
        responseBody = JSON.stringify(response.body);

        response2 = await get("/users/me", token);

    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    });

    it("user should not be able to get token", function(){
        expect(response2.status).to.equal(401);
    })
});


